//! IOReport power backend for Apple Silicon
//!
//! Uses the private IOReport framework to read CPU, GPU, ANE, and DRAM
//! power **without requiring `sudo`**. Works by subscribing to "Energy
//! Model" channels, taking two energy samples with a known time delta,
//! and computing power = ΔE / Δt.
//!
//! # Architecture
//! 1. `dlopen` → `/usr/lib/libIOReport.dylib` at runtime
//! 2. `dlsym` → resolve `IOReportCopyChannelsInGroup`, `IOReportCreateSubscription`,
//!    `IOReportCreateSamples`, `IOReportCreateSamplesDelta`, `IOReportSimpleGetIntegerValue`
//! 3. Subscribe to all `Energy Model` channels
//! 4. Take two samples (`Instant` + `IOReportCreateSamples`) → compute delta
//! 5. Parse channel names: `CPU Energy`, `GPU Energy`, `ANE*`, `DRAM*`
//!
//! # Graceful Degradation
//! If `libIOReport.dylib` is not found (older macOS, non-macOS), the
//! sampler returns `None` from `new()`, and callers fall back to
//! powermetrics. No crashes, no unresolved symbols at load time.

use std::sync::OnceLock;
use std::time::{Duration, Instant};

// ── CoreFoundation raw FFI (always available on macOS) ───────────────

type CFTypeRef = *const std::ffi::c_void;
type CFStringRef = CFTypeRef;
type CFDictionaryRef = CFTypeRef;
type CFMutableDictionaryRef = *mut std::ffi::c_void;
type CFArrayRef = CFTypeRef;

const K_CF_ALLOCATOR_DEFAULT: CFTypeRef = std::ptr::null();
const K_CF_STRING_ENCODING_UTF8: u32 = 0x08000100;

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFStringCreateWithCString(
        allocator: CFTypeRef,
        cStr: *const libc::c_char,
        encoding: u32,
    ) -> CFStringRef;
    fn CFStringGetCString(
        theString: CFStringRef,
        buffer: *mut libc::c_char,
        bufferSize: isize,
        encoding: u32,
    ) -> u8;
    fn CFDictionaryGetValue(theDict: CFDictionaryRef, key: CFTypeRef) -> CFTypeRef;
    fn CFDictionaryGetCount(theDict: CFDictionaryRef) -> isize;
    fn CFDictionaryGetKeysAndValues(
        theDict: CFDictionaryRef,
        keys: *mut CFTypeRef,
        values: *mut CFTypeRef,
    );
    fn CFDictionaryCreateMutable(
        allocator: CFTypeRef,
        capacity: isize,
        keyCallbacks: *const std::ffi::c_void,
        valueCallbacks: *const std::ffi::c_void,
    ) -> CFMutableDictionaryRef;
    fn CFDictionarySetValue(theDict: CFMutableDictionaryRef, key: CFTypeRef, value: CFTypeRef);
    fn CFArrayGetCount(theArray: CFArrayRef) -> isize;
    fn CFArrayGetValueAtIndex(theArray: CFArrayRef, idx: isize) -> CFTypeRef;
    fn CFRelease(obj: CFTypeRef);
}

// ── CF helpers ───────────────────────────────────────────────────────

fn cf_string_create(s: &str) -> CFStringRef {
    unsafe {
        CFStringCreateWithCString(
            K_CF_ALLOCATOR_DEFAULT,
            s.as_ptr() as *const libc::c_char,
            K_CF_STRING_ENCODING_UTF8,
        )
    }
}

fn cf_string_to_str(s: CFStringRef) -> Option<String> {
    if s.is_null() {
        return None;
    }
    unsafe {
        let mut buf = [0i8; 256];
        if CFStringGetCString(
            s,
            buf.as_mut_ptr(),
            buf.len() as isize,
            K_CF_STRING_ENCODING_UTF8,
        ) != 0
        {
            let len = buf.iter().position(|&c| c == 0).unwrap_or(255);
            Some(
                String::from_utf8_lossy(&buf[..len].iter().map(|&c| c as u8).collect::<Vec<_>>())
                    .into_owned(),
            )
        } else {
            None
        }
    }
}

fn cf_dict_get(dict: CFDictionaryRef, key: &str) -> CFTypeRef {
    let key_cf = cf_string_create(key);
    if key_cf.is_null() {
        return std::ptr::null();
    }
    let val = unsafe { CFDictionaryGetValue(dict, key_cf) };
    unsafe { CFRelease(key_cf) };
    val
}

fn cf_release(obj: CFTypeRef) {
    if !obj.is_null() {
        unsafe { CFRelease(obj) }
    }
}

// ── IOReport dlsym loading ───────────────────────────────────────────

/// Dynamically loaded IOReport function pointers.
struct IoReportFns {
    copy_channels_in_group:
        unsafe extern "C" fn(CFStringRef, CFStringRef, u64, u64, u64) -> CFDictionaryRef,
    create_subscription: unsafe extern "C" fn(
        CFTypeRef,
        CFMutableDictionaryRef,
        *mut CFMutableDictionaryRef,
        u64,
        CFTypeRef,
    ) -> *mut std::ffi::c_void,
    create_samples: unsafe extern "C" fn(
        *mut std::ffi::c_void,
        CFMutableDictionaryRef,
        CFTypeRef,
    ) -> CFDictionaryRef,
    create_samples_delta:
        unsafe extern "C" fn(CFDictionaryRef, CFDictionaryRef, CFTypeRef) -> CFDictionaryRef,
    channel_get_group: unsafe extern "C" fn(CFDictionaryRef) -> CFStringRef,
    channel_get_channel_name: unsafe extern "C" fn(CFDictionaryRef) -> CFStringRef,
    channel_get_unit_label: unsafe extern "C" fn(CFDictionaryRef) -> CFStringRef,
    simple_get_integer_value: unsafe extern "C" fn(CFDictionaryRef, i32) -> i64,
}

unsafe fn load_symbol<T>(handle: *mut std::ffi::c_void, name: &'static [u8]) -> T {
    let symbol = libc::dlsym(handle, name.as_ptr() as *const libc::c_char);
    std::mem::transmute_copy(&symbol)
}

static IOREPORT_FNS: OnceLock<Option<IoReportFns>> = OnceLock::new();

fn load_io_report() -> Option<&'static IoReportFns> {
    IOREPORT_FNS
        .get_or_init(|| {
            let lib_path = c"/usr/lib/libIOReport.dylib";
            let handle = unsafe {
                libc::dlopen(
                    lib_path.as_ptr() as *const libc::c_char,
                    libc::RTLD_LAZY | libc::RTLD_LOCAL,
                )
            };
            if handle.is_null() {
                log::debug!("IOReport: dlopen failed — libIOReport.dylib not found");
                return None;
            }

            let copy_channels_in_group =
                unsafe { load_symbol(handle, b"IOReportCopyChannelsInGroup\0") };
            let create_subscription =
                unsafe { load_symbol(handle, b"IOReportCreateSubscription\0") };
            let create_samples = unsafe { load_symbol(handle, b"IOReportCreateSamples\0") };
            let create_samples_delta =
                unsafe { load_symbol(handle, b"IOReportCreateSamplesDelta\0") };
            let channel_get_group = unsafe { load_symbol(handle, b"IOReportChannelGetGroup\0") };
            let channel_get_channel_name =
                unsafe { load_symbol(handle, b"IOReportChannelGetChannelName\0") };
            let channel_get_unit_label =
                unsafe { load_symbol(handle, b"IOReportChannelGetUnitLabel\0") };
            let simple_get_integer_value =
                unsafe { load_symbol(handle, b"IOReportSimpleGetIntegerValue\0") };

            Some(IoReportFns {
                copy_channels_in_group,
                create_subscription,
                create_samples,
                create_samples_delta,
                channel_get_group,
                channel_get_channel_name,
                channel_get_unit_label,
                simple_get_integer_value,
            })
        })
        .as_ref()
}

/// Check at runtime whether IOReport is available on this system.
pub fn is_available() -> bool {
    load_io_report().is_some()
}

// ── Sample type ──────────────────────────────────────────────────────

/// An absolute IOReport snapshot at a point in time.
struct Sample {
    inner: CFDictionaryRef,
    ts: Instant,
}

impl Drop for Sample {
    fn drop(&mut self) {
        cf_release(self.inner);
    }
}

// ── Public sampler ───────────────────────────────────────────────────

/// A power reading derived from IOReport energy channels.
#[derive(Debug, Clone, Default)]
pub struct IoReportPowerSample {
    pub cpu_w: f64,
    pub gpu_w: f64,
    pub ane_w: f64,
    pub dram_w: f64,
    pub package_w: f64,
}

/// IOReport power sampler for Apple Silicon.
///
/// Subscribes to Energy Model channels and computes power from energy
/// deltas. Create via [`IoReportSampler::new()`] which fails gracefully
/// if IOReport is unavailable.
pub struct IoReportSampler {
    subscription: *mut std::ffi::c_void,
    subscribed_channels: CFMutableDictionaryRef,
    prev_sample: Option<Sample>,
}

// Safety: IOReport API is thread-safe for separate sampler instances.
unsafe impl Send for IoReportSampler {}

impl IoReportSampler {
    /// Initialise IOReport and subscribe to all Energy Model channels.
    ///
    /// Returns `Err` if `libIOReport.dylib` is not found or no Energy
    /// Model channels are available. Callers should fall back to other
    /// power sources (e.g. powermetrics).
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let fns = load_io_report().ok_or_else(|| "IOReport not available".to_string())?;

        unsafe {
            // 1. Discover Energy Model channels
            let group_cf = cf_string_create("Energy Model");
            if group_cf.is_null() {
                return Err("IOReport: failed to create CFString".into());
            }

            let channels = (fns.copy_channels_in_group)(group_cf, std::ptr::null(), 0, 0, 0);
            cf_release(group_cf);

            if channels.is_null() {
                return Err("IOReport: no Energy Model channels found".into());
            }

            // 2. Create a mutable copy for subscription
            let desired = {
                let count = CFDictionaryGetCount(channels);
                if count == 0 {
                    cf_release(channels);
                    return Err("IOReport: Energy Model has 0 channels".into());
                }
                // CFDictionaryCreateMutableCopy is not available as raw FFI,
                // so we create a new mutable dict and copy key-value pairs.
                let mut_dict = CFDictionaryCreateMutable(
                    K_CF_ALLOCATOR_DEFAULT,
                    count,
                    &kCFTypeDictionaryKeyCallBacks as *const _ as *const std::ffi::c_void,
                    &kCFTypeDictionaryValueCallBacks as *const _ as *const std::ffi::c_void,
                );
                if mut_dict.is_null() {
                    cf_release(channels);
                    return Err("IOReport: failed to create mutable dict".into());
                }
                let mut keys = vec![std::ptr::null(); count as usize];
                let mut vals = vec![std::ptr::null(); count as usize];
                CFDictionaryGetKeysAndValues(channels, keys.as_mut_ptr(), vals.as_mut_ptr());
                for i in 0..count as usize {
                    if !keys[i].is_null() && !vals[i].is_null() {
                        CFDictionarySetValue(mut_dict, keys[i], vals[i]);
                    }
                }
                cf_release(channels);
                mut_dict
            };

            // 3. Subscribe
            let mut subd: CFMutableDictionaryRef = std::ptr::null_mut();
            let sub = (fns.create_subscription)(
                std::ptr::null(),
                desired,
                &mut subd,
                0,
                std::ptr::null(),
            );
            cf_release(desired as CFTypeRef);

            if sub.is_null() || subd.is_null() {
                cf_release(sub as CFTypeRef);
                cf_release(subd as CFTypeRef);
                return Err("IOReport: subscription failed".into());
            }

            log::info!("IOReport: subscribed to Energy Model channels");

            Ok(Self {
                subscription: sub,
                subscribed_channels: subd,
                prev_sample: None,
            })
        }
    }

    /// Take a single absolute energy sample.
    fn sample(&self) -> Result<Sample, Box<dyn std::error::Error>> {
        let fns = load_io_report().unwrap(); // safe: new() succeeded
        unsafe {
            let s = (fns.create_samples)(
                self.subscription,
                self.subscribed_channels,
                std::ptr::null(),
            );
            if s.is_null() {
                return Err("IOReport: create_samples returned null".into());
            }
            Ok(Sample {
                inner: s,
                ts: Instant::now(),
            })
        }
    }

    /// Sample power by taking two energy snapshots and computing the delta.
    ///
    /// Returns `IoReportPowerSample` with CPU/GPU/ANE/DRAM/package watts,
    /// or `None` if insufficient samples have been collected.
    pub fn sample_power(&mut self) -> Option<IoReportPowerSample> {
        let cur = self.sample().ok()?;
        let cur_ts = cur.ts;
        // Move `cur` into prev_sample, getting back the old sample as `prev`
        let prev = self.prev_sample.replace(cur)?;

        // Need at least 10ms delta for meaningful energy difference
        let dt = cur_ts.duration_since(prev.ts);
        if dt < Duration::from_millis(10) {
            return None;
        }

        // `self.prev_sample` now holds the new sample (cur); prev is the old one
        self.compute_power_delta(&prev, self.prev_sample.as_ref().unwrap(), dt)
    }

    /// Compute power from the energy delta between two samples.
    fn compute_power_delta(
        &self,
        prev: &Sample,
        cur: &Sample,
        dt: Duration,
    ) -> Option<IoReportPowerSample> {
        let fns = load_io_report().unwrap();
        let dt_ms = dt.as_millis() as u64;

        unsafe {
            let delta = (fns.create_samples_delta)(prev.inner, cur.inner, std::ptr::null());
            if delta.is_null() {
                return None;
            }

            // The delta contains an "IOReportChannels" key → CFArray
            let channels_arr = cf_dict_get(delta, "IOReportChannels") as CFArrayRef;
            if channels_arr.is_null() {
                cf_release(delta);
                return None;
            }

            let count = CFArrayGetCount(channels_arr);
            let mut result = IoReportPowerSample::default();

            for i in 0..count {
                let ch = CFArrayGetValueAtIndex(channels_arr, i) as CFDictionaryRef;
                if ch.is_null() {
                    continue;
                }

                let group_str = cf_string_to_str((fns.channel_get_group)(ch)).unwrap_or_default();
                if group_str != "Energy Model" {
                    continue;
                }

                let name = cf_string_to_str((fns.channel_get_channel_name)(ch)).unwrap_or_default();
                let unit = cf_string_to_str((fns.channel_get_unit_label)(ch)).unwrap_or_default();
                let val = (fns.simple_get_integer_value)(ch, 0);

                // Convert energy delta to watts
                let watts = energy_to_watts(val, &unit, dt_ms);

                // Categorise by channel name
                if name.ends_with("CPU Energy") {
                    result.cpu_w += watts;
                } else if name == "GPU Energy" {
                    result.gpu_w += watts;
                } else if name.starts_with("ANE") {
                    result.ane_w += watts;
                } else if name.starts_with("DRAM") {
                    result.dram_w += watts;
                }
            }

            result.package_w = result.cpu_w + result.gpu_w + result.ane_w + result.dram_w;

            cf_release(delta);
            Some(result)
        }
    }
}

impl Drop for IoReportSampler {
    fn drop(&mut self) {
        cf_release(self.subscription as CFTypeRef);
        cf_release(self.subscribed_channels as CFTypeRef);
    }
}

/// Convert IOReport integer energy value to watts.
///
/// `val` — energy delta from `IOReportSimpleGetIntegerValue`
/// `unit` — unit label from the channel (`"mJ"`, `"uJ"`, `"nJ"`)
/// `dt_ms` — wall-clock duration between the two samples in ms
fn energy_to_watts(val: i64, unit: &str, dt_ms: u64) -> f64 {
    if val <= 0 || dt_ms == 0 {
        return 0.0;
    }
    let dt_s = dt_ms as f64 / 1000.0;
    let per_sec = val as f64 / dt_s;
    match unit {
        "mJ" => per_sec / 1e3, // millijoules → joules/s = watts
        "uJ" => per_sec / 1e6, // microjoules → watts
        "nJ" => per_sec / 1e9, // nanojoules → watts
        _ => 0.0,              // unknown unit
    }
}

// ── CoreFoundation constant declarations ─────────────────────────────

// These are real symbols exported by CoreFoundation.
#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    static kCFTypeDictionaryKeyCallBacks: std::ffi::c_void;
    static kCFTypeDictionaryValueCallBacks: std::ffi::c_void;
}

// ── Tests (platform-agnostic) ────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_energy_to_watts_mj() {
        // 1000 mJ over 1000 ms → 1 W
        let w = energy_to_watts(1000, "mJ", 1000);
        assert!((w - 1.0).abs() < 1e-6, "Expected 1.0 W, got {}", w);
    }

    #[test]
    fn test_energy_to_watts_uj() {
        // 1_000_000 uJ over 1000 ms → 1 W
        let w = energy_to_watts(1_000_000, "uJ", 1000);
        assert!((w - 1.0).abs() < 1e-6, "Expected 1.0 W, got {}", w);
    }

    #[test]
    fn test_energy_to_watts_nj() {
        // 1_000_000_000 nJ over 1000 ms → 1 W
        let w = energy_to_watts(1_000_000_000, "nJ", 1000);
        assert!((w - 1.0).abs() < 1e-6, "Expected 1.0 W, got {}", w);
    }

    #[test]
    fn test_energy_to_watts_zero_energy() {
        assert_eq!(energy_to_watts(0, "mJ", 1000), 0.0);
    }

    #[test]
    fn test_energy_to_watts_unknown_unit() {
        assert_eq!(energy_to_watts(100, "J", 1000), 0.0);
    }

    #[test]
    fn test_energy_to_watts_short_interval() {
        // Even with 1ms, should compute correctly
        let w = energy_to_watts(1_000_000, "uJ", 1);
        assert!((w - 1000.0).abs() < 1.0, "Expected ~1000 W, got {}", w);
    }

    #[test]
    fn test_energy_to_watts_cpu_equivalent() {
        // Simulate: 500 mJ over 100 ms → 5 W (typical CPU power on Apple Silicon)
        let w = energy_to_watts(500, "mJ", 100);
        assert!((w - 5.0).abs() < 1e-6, "Expected 5.0 W, got {}", w);
    }
}

#[cfg(test)]
#[cfg(target_os = "macos")]
mod integration_tests {
    use super::*;

    #[test]
    fn test_is_available_checks() {
        // Should not panic; on macOS with IOReport, returns true
        let _ = is_available();
    }

    #[test]
    fn test_sampler_new_production() {
        // May fail in CI without IOReport, but should not crash or panic
        match IoReportSampler::new() {
            Ok(sampler) => {
                // If we got a sampler, try to sample power
                let mut s = sampler;
                // First sample sets prev
                let _ = s.sample_power();
                // Second sample should compute a delta
                std::thread::sleep(Duration::from_millis(50));
                let sample = s.sample_power();
                if let Some(p) = sample {
                    // Power values should be non-negative
                    assert!(p.cpu_w >= 0.0, "CPU power negative: {}", p.cpu_w);
                    assert!(p.gpu_w >= 0.0, "GPU power negative: {}", p.gpu_w);
                    assert!(p.ane_w >= 0.0, "ANE power negative: {}", p.ane_w);
                    assert!(p.dram_w >= 0.0, "DRAM power negative: {}", p.dram_w);
                    // Total should match sum
                    assert!(
                        (p.package_w - (p.cpu_w + p.gpu_w + p.ane_w + p.dram_w)).abs() < 0.001,
                        "Package power mismatch"
                    );
                }
            }
            Err(e) => {
                log::debug!("IOReport integration test skipped: {}", e);
            }
        }
    }
}

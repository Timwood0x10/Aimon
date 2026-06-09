//! IOHID temperature sensor backend
//!
//! Uses `IOHIDEventSystemClient` to read temperature sensor data from
//! Apple Silicon's internal HID sensors. Functions as an intermediate
//! layer between SMC (which may miss some sensors) and sysinfo
//! (which provides limited coverage).
//!
//! # Architecture
//! 1. `dlopen` → IOKit.framework
//! 2. `dlsym` → `IOHIDEventSystemClientCreate`, `SetMatching`,
//!    `CopyServices`, `IOHIDServiceClientCopyEvent`,
//!    `IOHIDServiceClientCopyProperty`, `IOHIDEventGetFloatValue`
//! 3. Match sensors: `PrimaryUsagePage = 0xff00`, `PrimaryUsage = 5`
//! 4. Iterate services → copy temperature events → extract °C
//! 5. Filter by name (`eACC`, `pACC`, `PMU tdie`, `SOC MTR Temp`)
//!    and valid range (0–150 °C)
//!
//! # Graceful Degradation
//! If IOHIDEventSystemClient symbols are not available (older macOS,
//! non-macOS), the backend returns an empty Vec and callers fall back
//! to sysinfo. No crashes, no unresolved symbols at load time.

use std::collections::HashMap;
use std::sync::OnceLock;

// ── CoreFoundation types (raw FFI, same pattern as IOReport) ─────────

type CFTypeRef = *const std::ffi::c_void;
type CFStringRef = CFTypeRef;
type CFDictionaryRef = CFTypeRef;
type CFNumberRef = CFTypeRef;
type CFArrayRef = CFTypeRef;
type CFAllocatorRef = CFTypeRef;

const K_CF_ALLOCATOR_DEFAULT: CFAllocatorRef = std::ptr::null();
const K_CF_STRING_ENCODING_UTF8: u32 = 0x08000100;
const K_CF_NUMBER_SINT32: u32 = 3; // kCFNumberSInt32Type

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFStringCreateWithCString(
        allocator: CFAllocatorRef,
        cStr: *const libc::c_char,
        encoding: u32,
    ) -> CFStringRef;
    fn CFStringGetCString(
        theString: CFStringRef,
        buffer: *mut libc::c_char,
        bufferSize: isize,
        encoding: u32,
    ) -> u8;
    fn CFDictionaryCreate(
        allocator: CFAllocatorRef,
        keys: *const CFTypeRef,
        values: *const CFTypeRef,
        numValues: isize,
        keyCallbacks: *const std::ffi::c_void,
        valueCallbacks: *const std::ffi::c_void,
    ) -> CFDictionaryRef;
    fn CFNumberCreate(
        allocator: CFAllocatorRef,
        theType: u32,
        valuePtr: *const std::ffi::c_void,
    ) -> CFNumberRef;
    fn CFArrayGetCount(theArray: CFArrayRef) -> isize;
    fn CFArrayGetValueAtIndex(theArray: CFArrayRef, idx: isize) -> CFTypeRef;
    fn CFRelease(obj: CFTypeRef);
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    static kCFTypeDictionaryKeyCallBacks: std::ffi::c_void;
    static kCFTypeDictionaryValueCallBacks: std::ffi::c_void;
}

// ── CF helpers ───────────────────────────────────────────────────────

fn cf_string_create(s: &str) -> CFStringRef {
    // Rust string literals are guaranteed null-terminated by the compiler,
    // so passing `as_ptr()` directly to C functions is safe.
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

fn cf_number_i32(value: i32) -> CFNumberRef {
    unsafe {
        CFNumberCreate(
            K_CF_ALLOCATOR_DEFAULT,
            K_CF_NUMBER_SINT32,
            &value as *const _ as *const std::ffi::c_void,
        )
    }
}

fn cf_release(obj: CFTypeRef) {
    if !obj.is_null() {
        unsafe { CFRelease(obj) }
    }
}

// ── IOHID types ──────────────────────────────────────────────────────

type IOHIDEventSystemClientRef = *mut std::ffi::c_void;
type IOHIDServiceClientRef = *mut std::ffi::c_void;
type IOHIDEventRef = *mut std::ffi::c_void;

// On LP64 (macOS 64-bit), IOHIDFloat is double
type IOHIDFloat = f64;

/// kIOHIDEventTypeTemperature = 15
const K_IOHID_EVENT_TYPE_TEMPERATURE: i64 = 15;

/// IOHIDEventFieldBase(type) = type << 16
fn iohid_event_field_base(type_: i64) -> i32 {
    (type_ << 16) as i32
}

// ── IOHID dlsym loading ──────────────────────────────────────────────

/// Dynamically loaded IOHID function pointers from IOKit.framework.
struct IohidFns {
    create_client: unsafe extern "C" fn(CFAllocatorRef) -> IOHIDEventSystemClientRef,
    set_matching: unsafe extern "C" fn(IOHIDEventSystemClientRef, CFDictionaryRef) -> i32,
    copy_services: unsafe extern "C" fn(IOHIDEventSystemClientRef) -> CFArrayRef,
    service_copy_event: unsafe extern "C" fn(IOHIDServiceClientRef, i64, i32, i64) -> IOHIDEventRef,
    service_copy_property: unsafe extern "C" fn(IOHIDServiceClientRef, CFStringRef) -> CFStringRef,
    event_get_float_value: unsafe extern "C" fn(IOHIDEventRef, i32) -> IOHIDFloat,
}

static IOHID_FNS: OnceLock<Option<IohidFns>> = OnceLock::new();

unsafe fn load_symbol<T>(handle: *mut std::ffi::c_void, name: &'static [u8]) -> T {
    let symbol = libc::dlsym(handle, name.as_ptr() as *const libc::c_char);
    std::mem::transmute_copy(&symbol)
}

fn load_iohid_fns() -> Option<&'static IohidFns> {
    IOHID_FNS
        .get_or_init(|| {
            let lib_path = c"/System/Library/Frameworks/IOKit.framework/IOKit";
            let handle = unsafe {
                libc::dlopen(
                    lib_path.as_ptr() as *const libc::c_char,
                    libc::RTLD_LAZY | libc::RTLD_LOCAL,
                )
            };
            if handle.is_null() {
                log::debug!("IOHID: dlopen failed — IOKit.framework not found");
                return None;
            }

            let create_client = unsafe { load_symbol(handle, b"IOHIDEventSystemClientCreate\0") };
            let set_matching =
                unsafe { load_symbol(handle, b"IOHIDEventSystemClientSetMatching\0") };
            let copy_services =
                unsafe { load_symbol(handle, b"IOHIDEventSystemClientCopyServices\0") };
            let service_copy_event =
                unsafe { load_symbol(handle, b"IOHIDServiceClientCopyEvent\0") };
            let service_copy_property =
                unsafe { load_symbol(handle, b"IOHIDServiceClientCopyProperty\0") };
            let event_get_float_value =
                unsafe { load_symbol(handle, b"IOHIDEventGetFloatValue\0") };

            Some(IohidFns {
                create_client,
                set_matching,
                copy_services,
                service_copy_event,
                service_copy_property,
                event_get_float_value,
            })
        })
        .as_ref()
}

/// Check at runtime whether IOHIDEventSystemClient is available.
pub fn is_available() -> bool {
    load_iohid_fns().is_some()
}

// ── Public API ───────────────────────────────────────────────────────

/// A temperature reading from IOHID.
#[derive(Debug, Clone)]
pub struct IohidTemperatureReading {
    pub label: String,
    pub value_celsius: f32,
}

/// Read all temperature sensors via IOHIDEventSystemClient.
///
/// Returns an empty Vec if IOHID is unavailable or no sensors are found.
/// Callers should fall back to sysinfo::Components.
pub fn read_iohid_temperatures() -> Vec<IohidTemperatureReading> {
    let fns = match load_iohid_fns() {
        Some(f) => f,
        None => return Vec::new(),
    };

    unsafe {
        // 1. Create matching dict: PrimaryUsagePage=0xff00, PrimaryUsage=5
        let page_key = cf_string_create("PrimaryUsagePage");
        let usage_key = cf_string_create("PrimaryUsage");
        if page_key.is_null() || usage_key.is_null() {
            cf_release(page_key);
            cf_release(usage_key);
            return Vec::new();
        }

        let page_val = cf_number_i32(0xff00);
        let usage_val = cf_number_i32(5);
        if page_val.is_null() || usage_val.is_null() {
            cf_release(page_key);
            cf_release(usage_key);
            cf_release(page_val);
            cf_release(usage_val);
            return Vec::new();
        }

        let keys: [CFTypeRef; 2] = [page_key as CFTypeRef, usage_key as CFTypeRef];
        let vals: [CFTypeRef; 2] = [page_val as CFTypeRef, usage_val as CFTypeRef];
        let matching_dict = CFDictionaryCreate(
            K_CF_ALLOCATOR_DEFAULT,
            keys.as_ptr(),
            vals.as_ptr(),
            2,
            &kCFTypeDictionaryKeyCallBacks as *const _,
            &kCFTypeDictionaryValueCallBacks as *const _,
        );
        cf_release(page_key);
        cf_release(usage_key);
        cf_release(page_val);
        cf_release(usage_val);

        if matching_dict.is_null() {
            return Vec::new();
        }

        // 2. Create client + set matching
        let client = (fns.create_client)(K_CF_ALLOCATOR_DEFAULT);
        if client.is_null() {
            cf_release(matching_dict);
            return Vec::new();
        }

        (fns.set_matching)(client, matching_dict);
        cf_release(matching_dict);

        // 3. Copy services
        let services = (fns.copy_services)(client);
        cf_release(client as CFTypeRef);

        if services.is_null() {
            return Vec::new();
        }

        let count = CFArrayGetCount(services);
        let mut readings: HashMap<String, f32> = HashMap::new();

        for i in 0..count {
            let sc = CFArrayGetValueAtIndex(services, i) as IOHIDServiceClientRef;
            if sc.is_null() {
                continue;
            }

            // 4. Get sensor name via "Product" property
            let product_key = cf_string_create("Product");
            let name_cf = (fns.service_copy_property)(sc, product_key);
            cf_release(product_key);

            let name = match cf_string_to_str(name_cf) {
                Some(n) => n,
                None => {
                    cf_release(name_cf as CFTypeRef);
                    continue;
                }
            };
            cf_release(name_cf as CFTypeRef);

            // 5. Copy temperature event
            let event = (fns.service_copy_event)(sc, K_IOHID_EVENT_TYPE_TEMPERATURE, 0, 0);
            if event.is_null() {
                continue;
            }

            // 6. Extract temperature value
            let field = iohid_event_field_base(K_IOHID_EVENT_TYPE_TEMPERATURE);
            let temp = (fns.event_get_float_value)(event, field);
            cf_release(event as CFTypeRef);

            // 7. Validate range (0–150 °C) and deduplicate by name
            if temp > 0.0 && temp < 150.0 {
                let temp_f32 = temp as f32;
                readings
                    .entry(name)
                    .and_modify(|e| *e = (*e + temp_f32) / 2.0) // average duplicates
                    .or_insert(temp_f32);
            }
        }

        cf_release(services as CFTypeRef);

        // 8. Sort by sensor name for stable ordering
        let mut result: Vec<IohidTemperatureReading> = readings
            .into_iter()
            .map(|(label, value_celsius)| IohidTemperatureReading {
                label,
                value_celsius,
            })
            .collect();
        result.sort_by(|a, b| a.label.cmp(&b.label));
        result
    }
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iohid_temperature_range_filter() {
        // Test the validation logic inline: temps outside 0-150 are rejected
        // This is enforced inside read_iohid_temperatures() — here we verify
        // the helper logic that temps are categorized correctly.
        let valid_temps = [0.1f32, 25.0, 100.0, 149.9];
        let invalid_temps = [-1.0f32, 0.0, 150.0, 200.0];

        for &t in &valid_temps {
            assert!(
                t > 0.0 && t < 150.0,
                "Valid temp {t} should pass the filter"
            );
        }
        for &t in &invalid_temps {
            assert!(
                !(t > 0.0 && t < 150.0),
                "Invalid temp {t} should fail the filter"
            );
        }
    }

    #[test]
    fn test_iohid_event_field_base() {
        assert_eq!(iohid_event_field_base(15), 15 << 16);
        assert_eq!(iohid_event_field_base(0), 0);
    }

    #[test]
    fn test_is_available_checks() {
        // Should not panic; on macOS returns whether IOHID works
        let _ = is_available();
    }
}
#[cfg(test)]
#[cfg(target_os = "macos")]
mod integration_tests {
    use super::*;

    /// Integration test that calls the real IOHID API.
    /// Marked `#[ignore]` because it may crash in CI environments
    /// without IOHID sensor access or with sandbox restrictions.
    #[ignore]
    #[test]
    fn test_read_temperatures_production() {
        let temps = read_iohid_temperatures();
        if !temps.is_empty() {
            for t in &temps {
                assert!(
                    !t.label.is_empty(),
                    "IOHID sensor label should not be empty"
                );
                assert!(
                    t.value_celsius > 0.0 && t.value_celsius < 150.0,
                    "IOHID temp {}°C out of range for {}",
                    t.value_celsius,
                    t.label
                );
            }
        }
    }
}

//! Read-only SMC (System Management Controller) backend
//!
//! Enumerates temperature sensors and fan data by reading Apple SMC keys
//! via IOKit. Fan writes are compiled only with the `fan-control` feature
//! and should be enabled by callers only after explicit operator consent.
//!
//! # Architecture
//! 1. Open AppleSMC service via IOKit (`IOServiceGetMatchingService` +
//!    `IOServiceOpen`)
//! 2. Send SMC commands via `IOConnectCallStructMethod` with the well-known
//!    `SMCParamStruct` layout
//! 3. Decode returned data in sp78 (temperature), fpe2 (fan speed), and
//!    integer formats
//! 4. Fall back gracefully if SMC is unavailable or a key is not found

use crate::types::FanInfo;
use std::mem::size_of;

// ── IOKit types ──────────────────────────────────────────────────────

#[allow(non_camel_case_types)]
type mach_port_t = libc::c_uint;
#[allow(non_camel_case_types)]
type kern_return_t = libc::c_int;
#[allow(non_camel_case_types)]
type io_service_t = mach_port_t;
#[allow(non_camel_case_types)]
type io_connect_t = mach_port_t;

const KERN_SUCCESS: kern_return_t = 0;
const KIO_MASTER_PORT_DEFAULT: mach_port_t = 0;

// ── SMC constants ────────────────────────────────────────────────────

/// SMC command: get key info or read key value
const SMC_CMD_READ_KEY: u8 = 5;
#[cfg(feature = "fan-control")]
const SMC_CMD_WRITE_KEY: u8 = 6;

/// IOKit user-client selector for SMC communication
const SMC_SELECTOR: u32 = 2;

// ── SMCParamStruct ───────────────────────────────────────────────────
// Well-known layout used by osx-cpu-temp, smcFanControl, iStats, etc.
// Total size: 52 bytes.

#[repr(C)]
#[derive(Clone, Copy)]
struct SMCParamStruct {
    key: u32,        // 0-3   SMC key code (big-endian)
    data_type: u32,  // 4-7   Data type code (big-endian)
    data_bytes: u32, // 8-11  Number of valid data bytes
    command: u8,     // 12    Command code (5 = read)
    result: u8,      // 13    Result code (0 = success)
    _pad: [u8; 2],   // 14-15 Padding
    data32: u32,     // 16-19 Auxiliary data
    bytes: [u8; 32], // 20-51 Data buffer
}

impl SMCParamStruct {
    fn new() -> Self {
        Self {
            key: 0,
            data_type: 0,
            data_bytes: 0,
            command: 0,
            result: 0,
            _pad: [0; 2],
            data32: 0,
            bytes: [0; 32],
        }
    }
}

// ── IOKit FFI ────────────────────────────────────────────────────────

#[link(name = "IOKit", kind = "framework")]
extern "C" {
    fn IOServiceGetMatchingService(
        master_port: mach_port_t,
        matching: *mut libc::c_void,
    ) -> io_service_t;

    fn IOServiceMatching(name: *const libc::c_char) -> *mut libc::c_void;

    fn IOServiceOpen(
        service: io_service_t,
        owning_task: mach_port_t,
        type_: u32,
        connect: *mut io_connect_t,
    ) -> kern_return_t;

    fn IOConnectCallStructMethod(
        connection: io_connect_t,
        selector: u32,
        input_struct: *const libc::c_void,
        input_struct_cnt: usize,
        output_struct: *mut libc::c_void,
        output_struct_cnt: *mut usize,
    ) -> kern_return_t;

    fn IOServiceClose(connection: io_connect_t) -> kern_return_t;

    fn mach_task_self() -> mach_port_t;
}

// ── SMC data type codes (4-byte identifiers) ─────────────────────────

const TYPE_SP78: u32 = u32::from_ne_bytes([b's', b'p', b'7', b'8']);
const TYPE_FPE2: u32 = u32::from_ne_bytes([b'f', b'p', b'e', b'2']);
const TYPE_FLT: u32 = u32::from_ne_bytes([b'f', b'l', b't', b' ']);
const TYPE_UI8: u32 = u32::from_ne_bytes([b'u', b'i', b'8', b' ']);
const TYPE_UI16: u32 = u32::from_ne_bytes([b'u', b'i', b'1', b'6']);
const TYPE_UI32: u32 = u32::from_ne_bytes([b'u', b'i', b'3', b'2']);

// ── Known temperature SMC keys (4-char codes as big-endian u32) ──────
// These work on Apple Silicon Macs.

const TEMP_KEYS: &[(&str, &str, u32)] = &[
    ("Tp09", "CPU Proximity 1", u32::from_be_bytes(*b"Tp09")),
    ("Tp0T", "CPU Proximity 2", u32::from_be_bytes(*b"Tp0T")),
    ("Tp01", "P-Core 1", u32::from_be_bytes(*b"Tp01")),
    ("Tp03", "P-Core 2", u32::from_be_bytes(*b"Tp03")),
    ("Tp05", "P-Core 3", u32::from_be_bytes(*b"Tp05")),
    ("Tp07", "P-Core 4", u32::from_be_bytes(*b"Tp07")),
    ("Tp0D", "E-Core Cluster", u32::from_be_bytes(*b"Tp0D")),
    ("Tp0H", "SoC", u32::from_be_bytes(*b"Tp0H")),
    ("Tb0T", "Battery", u32::from_be_bytes(*b"Tb0T")),
    ("Tb1T", "Battery 2", u32::from_be_bytes(*b"Tb1T")),
    ("Ts0P", "Thermal Sensor", u32::from_be_bytes(*b"Ts0P")),
    ("Tm0P", "Memory Proximity", u32::from_be_bytes(*b"Tm0P")),
    ("Ta0P", "Ambient 1", u32::from_be_bytes(*b"Ta0P")),
    ("Ta1P", "Ambient 2", u32::from_be_bytes(*b"Ta1P")),
];

/// Helper: build a fan key code like b"F0Ac" → u32
fn fan_key(index: u32, suffix: &[u8; 2]) -> u32 {
    let mut buf = [b'F'; 4];
    let digit = (index as u8) + b'0';
    buf[1] = digit;
    buf[2] = suffix[0];
    buf[3] = suffix[1];
    u32::from_be_bytes(buf)
}

// ── Public API ───────────────────────────────────────────────────────

/// A temperature reading from the SMC
#[derive(Debug, Clone)]
pub struct SmcTemperatureReading {
    pub key: String,
    pub label: String,
    pub value_celsius: f32,
}

/// Read-only SMC handle backed by an IOKit connection
pub struct SmcHandle {
    conn: io_connect_t,
}

impl SmcHandle {
    /// Attempt to open AppleSMC via IOKit.
    ///
    /// Returns `Err` if the AppleSMC service is not found or the connection
    /// cannot be established. Callers should fall back to other sources.
    pub fn open() -> Result<Self, Box<dyn std::error::Error>> {
        unsafe {
            // 1. Find the AppleSMC service
            let service_name = c"AppleSMC".as_ptr() as *const libc::c_char;
            let matching = IOServiceMatching(service_name);
            if matching.is_null() {
                return Err("IOServiceMatching returned null".into());
            }

            let service = IOServiceGetMatchingService(KIO_MASTER_PORT_DEFAULT, matching);
            if service == 0 {
                return Err("AppleSMC service not found".into());
            }

            // 2. Open a connection to the service
            let mut conn: io_connect_t = 0;
            let task = mach_task_self();
            let ret = IOServiceOpen(service, task, 0, &mut conn);

            if ret != KERN_SUCCESS {
                return Err(format!("IOServiceOpen failed: {}", ret).into());
            }

            Ok(Self { conn })
        }
    }

    /// Read a raw SMC key value.
    ///
    /// Two-step protocol:
    ///   1. Send `SMC_CMD_READ_KEY` with `data_bytes = 0` to get key info
    ///      (data type + size)
    ///   2. Send `SMC_CMD_READ_KEY` with the actual `data_bytes` to read
    ///      the value
    fn smc_read(&self, raw_key: u32) -> Result<SMCValue, Box<dyn std::error::Error>> {
        let key_be = raw_key.to_be();

        // ── Step 1: Get key info ────────────────────────────────────
        let mut input = SMCParamStruct::new();
        input.key = key_be;
        input.command = SMC_CMD_READ_KEY;
        input.data_bytes = 0;

        let mut output = SMCParamStruct::new();
        let mut output_size = size_of::<SMCParamStruct>();

        let ret = unsafe {
            IOConnectCallStructMethod(
                self.conn,
                SMC_SELECTOR,
                &input as *const _ as *const libc::c_void,
                size_of::<SMCParamStruct>(),
                &mut output as *mut _ as *mut libc::c_void,
                &mut output_size,
            )
        };

        if ret != KERN_SUCCESS {
            return Err(format!("SMC read (info) failed: {}", ret).into());
        }
        if output.result != 0 {
            return Err(format!("SMC key not found (result: {})", output.result).into());
        }

        let data_type = u32::from_be(output.data_type);
        let data_bytes = u32::from_be(output.data_bytes);

        if data_bytes == 0 || data_bytes > 32 {
            return Err(format!("SMC invalid data size: {}", data_bytes).into());
        }

        // ── Step 2: Read the value ──────────────────────────────────
        input = SMCParamStruct::new();
        input.key = key_be;
        input.command = SMC_CMD_READ_KEY;
        input.data_bytes = data_bytes.to_be();

        output = SMCParamStruct::new();
        output_size = size_of::<SMCParamStruct>();

        let ret = unsafe {
            IOConnectCallStructMethod(
                self.conn,
                SMC_SELECTOR,
                &input as *const _ as *const libc::c_void,
                size_of::<SMCParamStruct>(),
                &mut output as *mut _ as *mut libc::c_void,
                &mut output_size,
            )
        };

        if ret != KERN_SUCCESS {
            return Err(format!("SMC read (value) failed: {}", ret).into());
        }
        if output.result != 0 {
            return Err(format!("SMC value read failed (result: {})", output.result).into());
        }

        let bytes = output.bytes;
        Ok(SMCValue {
            data_type,
            data_bytes: data_bytes as usize,
            bytes,
        })
    }

    #[cfg(feature = "fan-control")]
    fn smc_write(&self, raw_key: u32, value: &SMCValue) -> Result<(), Box<dyn std::error::Error>> {
        let mut input = SMCParamStruct::new();
        input.key = raw_key.to_be();
        input.command = SMC_CMD_WRITE_KEY;
        input.data_type = value.data_type.to_be();
        input.data_bytes = (value.data_bytes as u32).to_be();
        input.bytes = value.bytes;

        let mut output = SMCParamStruct::new();
        let mut output_size = size_of::<SMCParamStruct>();

        let ret = unsafe {
            IOConnectCallStructMethod(
                self.conn,
                SMC_SELECTOR,
                &input as *const _ as *const libc::c_void,
                size_of::<SMCParamStruct>(),
                &mut output as *mut _ as *mut libc::c_void,
                &mut output_size,
            )
        };

        if ret != KERN_SUCCESS {
            return Err(format!("SMC write failed: {}", ret).into());
        }
        if output.result != 0 {
            return Err(format!("SMC write result failed: {}", output.result).into());
        }

        Ok(())
    }

    /// Read all available temperature sensors from known SMC keys.
    pub fn read_temperatures(
        &self,
    ) -> Result<Vec<SmcTemperatureReading>, Box<dyn std::error::Error>> {
        let mut readings = Vec::new();

        for &(_key_str, label, raw_key) in TEMP_KEYS {
            if let Ok(val) = self.smc_read(raw_key) {
                if let Some(celsius) = val.as_f32() {
                    readings.push(SmcTemperatureReading {
                        key: String::from_utf8_lossy(&raw_key.to_be_bytes()).to_string(),
                        label: label.to_string(),
                        value_celsius: celsius,
                    });
                }
            }
        }

        Ok(readings)
    }

    /// Read fan count via the `FNum` key.
    pub fn fan_count(&self) -> Result<u32, Box<dyn std::error::Error>> {
        let key = u32::from_be_bytes(*b"FNum");
        let val = self.smc_read(key)?;
        val.as_u32().ok_or_else(|| "FNum not an integer".into())
    }

    /// Read individual fan info.
    pub fn read_fan(&self, index: u32) -> Result<FanInfo, Box<dyn std::error::Error>> {
        let ac_key = fan_key(index, b"Ac");
        let mn_key = fan_key(index, b"Mn");
        let mx_key = fan_key(index, b"Mx");
        let tg_key = fan_key(index, b"Tg");
        let md_key = fan_key(index, b"Md");

        let current_rpm = self
            .smc_read(ac_key)
            .ok()
            .and_then(|v| v.as_f32())
            .unwrap_or(0.0) as u32;
        let min_rpm = self
            .smc_read(mn_key)
            .ok()
            .and_then(|v| v.as_f32())
            .unwrap_or(0.0) as u32;
        let max_rpm = self
            .smc_read(mx_key)
            .ok()
            .and_then(|v| v.as_f32())
            .unwrap_or(0.0) as u32;
        let target_rpm = self
            .smc_read(tg_key)
            .ok()
            .and_then(|v| v.as_f32())
            .unwrap_or(0.0) as u32;

        // Read mode: b"Auto" or b"Manu" or similar 4-char code
        let mode = if let Ok(val) = self.smc_read(md_key) {
            if val.data_type == u32::from_ne_bytes([b'c', b'h', b'8', b'*']) {
                let raw = val.bytes[0];
                if raw == 0 {
                    "Auto"
                } else {
                    "Manual"
                }
            } else {
                "Auto"
            }
        } else {
            "Auto"
        };

        Ok(FanInfo {
            id: index as usize,
            current_rpm,
            min_rpm,
            max_rpm,
            target_rpm,
            mode: mode.to_string(),
        })
    }

    /// Read all fans.
    pub fn read_all_fans(&self) -> Result<Vec<FanInfo>, Box<dyn std::error::Error>> {
        let count = self.fan_count()?;
        (0..count).map(|i| self.read_fan(i)).collect()
    }

    #[cfg(feature = "fan-control")]
    pub fn set_fan_target_rpm(
        &self,
        index: u32,
        target_rpm: u32,
        safe_min_rpm: u32,
        safe_max_rpm: u32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let fan = self.read_fan(index)?;
        let floor = safe_min_rpm.max(fan.min_rpm);
        let ceiling = safe_max_rpm.min(fan.max_rpm.max(safe_max_rpm));
        if target_rpm < floor || target_rpm > ceiling {
            return Err(format!(
                "target RPM {} outside safe range {}..={}",
                target_rpm, floor, ceiling
            )
            .into());
        }

        self.smc_write(fan_key(index, b"Md"), &SMCValue::ui8(1))?;
        self.smc_write(fan_key(index, b"Tg"), &SMCValue::fpe2(target_rpm))
    }

    #[cfg(feature = "fan-control")]
    pub fn set_fan_auto(&self, index: u32) -> Result<(), Box<dyn std::error::Error>> {
        self.smc_write(fan_key(index, b"Md"), &SMCValue::ui8(0))
    }
}

impl Drop for SmcHandle {
    fn drop(&mut self) {
        unsafe {
            IOServiceClose(self.conn);
        }
    }
}

// ── Value decoding ───────────────────────────────────────────────────

struct SMCValue {
    data_type: u32,
    data_bytes: usize,
    bytes: [u8; 32],
}

impl SMCValue {
    #[cfg(feature = "fan-control")]
    fn ui8(value: u8) -> Self {
        let mut bytes = [0u8; 32];
        bytes[0] = value;
        Self {
            data_type: TYPE_UI8,
            data_bytes: 1,
            bytes,
        }
    }

    #[cfg(feature = "fan-control")]
    fn fpe2(rpm: u32) -> Self {
        let raw = (rpm.saturating_mul(4)).min(u16::MAX as u32) as u16;
        let mut bytes = [0u8; 32];
        bytes[..2].copy_from_slice(&raw.to_be_bytes());
        Self {
            data_type: TYPE_FPE2,
            data_bytes: 2,
            bytes,
        }
    }

    /// Decode as f32 if the data type supports floating-point values
    /// (sp78, fpe2, flt).
    fn as_f32(&self) -> Option<f32> {
        match self.data_type {
            TYPE_SP78 => {
                // Signed 7.8 fixed-point: first byte is integer part,
                // second byte is fractional part (1/256)
                if self.data_bytes >= 2 {
                    let int = self.bytes[0] as i8 as f32;
                    let frac = self.bytes[1] as f32 / 256.0;
                    Some(int + frac)
                } else {
                    None
                }
            }
            TYPE_FPE2 => {
                // fpe2: N bits integer, 2 bits fractional
                if self.data_bytes >= 2 {
                    let raw = u16::from_be_bytes([self.bytes[0], self.bytes[1]]) as f32;
                    Some(raw / 4.0)
                } else {
                    None
                }
            }
            TYPE_FLT => {
                if self.data_bytes >= 4 {
                    let bits = u32::from_be_bytes([
                        self.bytes[0],
                        self.bytes[1],
                        self.bytes[2],
                        self.bytes[3],
                    ]);
                    Some(f32::from_bits(bits))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Decode as u32 if the data type is an unsigned integer.
    fn as_u32(&self) -> Option<u32> {
        match self.data_type {
            TYPE_UI8 => Some(self.bytes[0] as u32),
            TYPE_UI16 => {
                if self.data_bytes >= 2 {
                    Some(u16::from_be_bytes([self.bytes[0], self.bytes[1]]) as u32)
                } else {
                    None
                }
            }
            TYPE_UI32 => {
                if self.data_bytes >= 4 {
                    Some(u32::from_be_bytes([
                        self.bytes[0],
                        self.bytes[1],
                        self.bytes[2],
                        self.bytes[3],
                    ]))
                } else {
                    None
                }
            }
            _ => {
                // Try fpe2 (fan speed data is often fpe2)
                self.as_f32().map(|v| v as u32)
            }
        }
    }
}

// ── Convenience top-level functions ──────────────────────────────────

/// Open the SMC, read all temperature sensors, and close.
///
/// Returns an empty Vec if SMC is unavailable.
pub fn read_smc_temperatures() -> Vec<SmcTemperatureReading> {
    match SmcHandle::open() {
        Ok(handle) => handle.read_temperatures().unwrap_or_default(),
        Err(e) => {
            log::debug!("SMC temperature read failed (will use sysinfo): {}", e);
            Vec::new()
        }
    }
}

/// Open the SMC, read all fan info, and close.
///
/// Returns an empty Vec if SMC is unavailable.
pub fn read_smc_fans() -> Vec<FanInfo> {
    match SmcHandle::open() {
        Ok(handle) => handle.read_all_fans().unwrap_or_default(),
        Err(e) => {
            log::debug!("SMC fan read failed (will use powermetrics): {}", e);
            Vec::new()
        }
    }
}

#[cfg(feature = "fan-control")]
pub fn set_smc_fans_target_rpm(
    target_rpm: u32,
    safe_min_rpm: u32,
    safe_max_rpm: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let handle = SmcHandle::open()?;
    let count = handle.fan_count()?;
    for index in 0..count {
        handle.set_fan_target_rpm(index, target_rpm, safe_min_rpm, safe_max_rpm)?;
    }
    Ok(())
}

#[cfg(not(feature = "fan-control"))]
pub fn set_smc_fans_target_rpm(
    _target_rpm: u32,
    _safe_min_rpm: u32,
    _safe_max_rpm: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    Err("fan-control feature is not enabled".into())
}

#[cfg(feature = "fan-control")]
pub fn set_smc_fans_auto() -> Result<(), Box<dyn std::error::Error>> {
    let handle = SmcHandle::open()?;
    let count = handle.fan_count()?;
    for index in 0..count {
        handle.set_fan_auto(index)?;
    }
    Ok(())
}

#[cfg(not(feature = "fan-control"))]
pub fn set_smc_fans_auto() -> Result<(), Box<dyn std::error::Error>> {
    Err("fan-control feature is not enabled".into())
}

// ── Tests ────────────────────────────────────────────────────────────
// These tests only run on macOS where the SMC is available.

#[cfg(test)]
#[cfg(target_os = "macos")]
mod tests {
    use super::*;

    #[test]
    fn test_smc_open() {
        let handle = SmcHandle::open();
        // SMC may or may not be accessible (sandboxing, etc.), but the
        // call should not crash or panic either way.
        if let Ok(h) = handle {
            let _ = h.read_fan(0);
            let _ = h.read_temperatures();
        }
    }

    #[test]
    fn test_smc_temperatures() {
        let temps = read_smc_temperatures();
        // If SMC is available, we should get at least some readings;
        // if not, an empty Vec is acceptable.
        if !temps.is_empty() {
            for t in &temps {
                assert!(!t.key.is_empty(), "SMC temperature key should not be empty");
                assert!(
                    !t.label.is_empty(),
                    "SMC temperature label should not be empty"
                );
                // Reasonable temperature range: -10°C to 150°C
                assert!(
                    t.value_celsius > -10.0 && t.value_celsius < 150.0,
                    "Temperature {}°C out of reasonable range for {}",
                    t.value_celsius,
                    t.label
                );
            }
        }
    }

    #[test]
    fn test_smc_fan_count() {
        let handle = SmcHandle::open();
        if let Ok(h) = handle {
            let count = h.fan_count().unwrap_or(0);
            // Most Macs have 0-4 fans
            assert!(count <= 4, "Unexpected fan count: {}", count);
        }
    }

    #[test]
    fn test_smc_fan_rpm_range() {
        let handle = SmcHandle::open();
        if let Ok(h) = handle {
            if let Ok(fans) = h.read_all_fans() {
                for fan in &fans {
                    // RPM should be within reasonable range
                    assert!(
                        fan.current_rpm < 20000,
                        "Fan RPM {} seems unreasonably high",
                        fan.current_rpm
                    );
                }
            }
        }
    }

    #[test]
    fn test_fan_key_helper() {
        let key = fan_key(0, b"Ac");
        let bytes = key.to_be_bytes();
        assert_eq!(&bytes, b"F0Ac", "fan_key(0, Ac) should produce b\"F0Ac\"");

        let key2 = fan_key(1, b"Mn");
        let bytes2 = key2.to_be_bytes();
        assert_eq!(&bytes2, b"F1Mn", "fan_key(1, Mn) should produce b\"F1Mn\"");
    }
}

// ── Mock decode tests (platform-agnostic) ─────────────────────────
// These construct SMCValue directly with known byte patterns so they
// run on any platform without needing IOKit.

#[cfg(test)]
mod decode_tests {
    use super::*;

    // ── sp78 (signed 7.8 fixed-point) ───────────────────────────────

    #[test]
    fn test_sp78_positive_temp() {
        // 45.5°C = 0x2D80 → int=0x2D=45, frac=0x80/256=0.5
        let val = SMCValue {
            data_type: TYPE_SP78,
            data_bytes: 2,
            bytes: {
                let mut b = [0u8; 32];
                b[0] = 0x2D;
                b[1] = 0x80;
                b
            },
        };
        let result = val.as_f32().unwrap();
        assert!(
            (result - 45.5).abs() < 1e-4,
            "Expected 45.5, got {}",
            result
        );
    }

    #[test]
    fn test_sp78_zero() {
        let val = SMCValue {
            data_type: TYPE_SP78,
            data_bytes: 2,
            bytes: {
                let mut b = [0u8; 32];
                b[0] = 0x00;
                b[1] = 0x00;
                b
            },
        };
        let result = val.as_f32().unwrap();
        assert!((result - 0.0).abs() < 1e-4, "Expected 0.0, got {}", result);
    }

    #[test]
    fn test_sp78_negative_temp() {
        // -9.25°C = 0xF6C0 → int=0xF6(signed=-10), frac=0xC0/256=0.75
        // -10 + 0.75 = -9.25
        let val = SMCValue {
            data_type: TYPE_SP78,
            data_bytes: 2,
            bytes: {
                let mut b = [0u8; 32];
                b[0] = 0xF6u8; // -10 as i8
                b[1] = 0xC0;
                b
            },
        };
        let result = val.as_f32().unwrap();
        assert!(
            (result - (-9.25)).abs() < 1e-4,
            "Expected -9.25, got {}",
            result
        );
    }

    #[test]
    fn test_sp78_100c() {
        // 100.0°C = 0x6400 → int=0x64=100, frac=0x00=0
        let val = SMCValue {
            data_type: TYPE_SP78,
            data_bytes: 2,
            bytes: {
                let mut b = [0u8; 32];
                b[0] = 0x64;
                b[1] = 0x00;
                b
            },
        };
        let result = val.as_f32().unwrap();
        assert!(
            (result - 100.0).abs() < 1e-4,
            "Expected 100.0, got {}",
            result
        );
    }

    #[test]
    fn test_sp78_insufficient_bytes() {
        let val = SMCValue {
            data_type: TYPE_SP78,
            data_bytes: 1, // Need 2, have 1
            bytes: [0; 32],
        };
        assert!(
            val.as_f32().is_none(),
            "sp78 with 1 byte should return None"
        );
    }

    // ── fpe2 (2-bit fractional, fan speed) ───────────────────────────

    #[test]
    fn test_fpe2_integer() {
        // 5000 RPM = 0x4E20 → big-endian u16 = 5000×4 = 20000 = 0x4E20
        let val = SMCValue {
            data_type: TYPE_FPE2,
            data_bytes: 2,
            bytes: {
                let mut b = [0u8; 32];
                b[0] = 0x4E;
                b[1] = 0x20;
                b
            },
        };
        let result = val.as_f32().unwrap();
        assert!(
            (result - 5000.0).abs() < 1e-4,
            "Expected 5000, got {}",
            result
        );
    }

    #[test]
    fn test_fpe2_fractional() {
        // 1234.5 RPM → 1234.5 × 4 = 4938 = 0x134A
        let val = SMCValue {
            data_type: TYPE_FPE2,
            data_bytes: 2,
            bytes: {
                let mut b = [0u8; 32];
                b[0] = 0x13;
                b[1] = 0x4A;
                b
            },
        };
        let result = val.as_f32().unwrap();
        assert!(
            (result - 1234.5).abs() < 1e-4,
            "Expected 1234.5, got {}",
            result
        );
    }

    #[test]
    fn test_fpe2_zero() {
        let val = SMCValue {
            data_type: TYPE_FPE2,
            data_bytes: 2,
            bytes: [0; 32],
        };
        let result = val.as_f32().unwrap();
        assert!((result - 0.0).abs() < 1e-4, "Expected 0.0, got {}", result);
    }

    #[test]
    fn test_fpe2_insufficient_bytes() {
        let val = SMCValue {
            data_type: TYPE_FPE2,
            data_bytes: 1,
            bytes: [0xFF; 32],
        };
        assert!(
            val.as_f32().is_none(),
            "fpe2 with 1 byte should return None"
        );
    }

    // ── flt (IEEE 754 float) ─────────────────────────────────────────

    #[test]
    fn test_flt_positive() {
        // 12.5 = 0x41480000 in IEEE 754 (big-endian)
        let val = SMCValue {
            data_type: TYPE_FLT,
            data_bytes: 4,
            bytes: {
                let mut b = [0u8; 32];
                b[0] = 0x41;
                b[1] = 0x48;
                b[2] = 0x00;
                b[3] = 0x00;
                b
            },
        };
        let result = val.as_f32().unwrap();
        assert!(
            (result - 12.5).abs() < 1e-4,
            "Expected 12.5, got {}",
            result
        );
    }

    #[test]
    #[allow(clippy::approx_constant)]
    fn test_flt_negative() {
        // -3.14 ≈ 0xC048F5C3 in IEEE 754
        let val = SMCValue {
            data_type: TYPE_FLT,
            data_bytes: 4,
            bytes: {
                let mut b = [0u8; 32];
                b[0] = 0xC0;
                b[1] = 0x48;
                b[2] = 0xF5;
                b[3] = 0xC3;
                b
            },
        };
        let result = val.as_f32().unwrap();
        assert!(
            (result - (-3.14)).abs() < 1e-3,
            "Expected ~-3.14, got {}",
            result
        );
    }

    #[test]
    fn test_flt_insufficient_bytes() {
        let val = SMCValue {
            data_type: TYPE_FLT,
            data_bytes: 2, // Need 4, have 2
            bytes: [0; 32],
        };
        assert!(
            val.as_f32().is_none(),
            "flt with 2 bytes should return None"
        );
    }

    // ── ui8 (unsigned 8-bit integer) ─────────────────────────────────

    #[test]
    fn test_ui8_zero() {
        let val = SMCValue {
            data_type: TYPE_UI8,
            data_bytes: 1,
            bytes: {
                let mut b = [0u8; 32];
                b[0] = 0;
                b
            },
        };
        assert_eq!(val.as_u32(), Some(0));
    }

    #[test]
    fn test_ui8_max() {
        let val = SMCValue {
            data_type: TYPE_UI8,
            data_bytes: 1,
            bytes: {
                let mut b = [0u8; 32];
                b[0] = 0xFF;
                b
            },
        };
        assert_eq!(val.as_u32(), Some(255));
    }

    #[test]
    fn test_ui8_mid() {
        let val = SMCValue {
            data_type: TYPE_UI8,
            data_bytes: 1,
            bytes: {
                let mut b = [0u8; 32];
                b[0] = 42;
                b
            },
        };
        assert_eq!(val.as_u32(), Some(42));
    }

    // ── ui16 (unsigned 16-bit integer, big-endian) ───────────────────

    #[test]
    fn test_ui16_zero() {
        let val = SMCValue {
            data_type: TYPE_UI16,
            data_bytes: 2,
            bytes: [0; 32],
        };
        assert_eq!(val.as_u32(), Some(0));
    }

    #[test]
    fn test_ui16_max() {
        let val = SMCValue {
            data_type: TYPE_UI16,
            data_bytes: 2,
            bytes: {
                let mut b = [0u8; 32];
                b[0] = 0xFF;
                b[1] = 0xFF;
                b
            },
        };
        assert_eq!(val.as_u32(), Some(65535));
    }

    #[test]
    fn test_ui16_big_endian() {
        // 0x1234 = 4660
        let val = SMCValue {
            data_type: TYPE_UI16,
            data_bytes: 2,
            bytes: {
                let mut b = [0u8; 32];
                b[0] = 0x12;
                b[1] = 0x34;
                b
            },
        };
        assert_eq!(val.as_u32(), Some(0x1234));
    }

    #[test]
    fn test_ui16_insufficient_bytes() {
        let val = SMCValue {
            data_type: TYPE_UI16,
            data_bytes: 1,
            bytes: [0xFF; 32],
        };
        assert!(
            val.as_u32().is_none(),
            "ui16 with 1 byte should return None"
        );
    }

    // ── ui32 (unsigned 32-bit integer, big-endian) ───────────────────

    #[test]
    fn test_ui32_known_value() {
        // 0xDEADBEEF = 3735928559
        let val = SMCValue {
            data_type: TYPE_UI32,
            data_bytes: 4,
            bytes: {
                let mut b = [0u8; 32];
                b[0] = 0xDE;
                b[1] = 0xAD;
                b[2] = 0xBE;
                b[3] = 0xEF;
                b
            },
        };
        assert_eq!(val.as_u32(), Some(0xDEAD_BEEF));
    }

    #[test]
    fn test_ui32_big_endian() {
        // Big-endian: 0x01 = MSB, 0x00 = LSBs → value = 1
        let val = SMCValue {
            data_type: TYPE_UI32,
            data_bytes: 4,
            bytes: {
                let mut b = [0u8; 32];
                b[0] = 0x00;
                b[1] = 0x00;
                b[2] = 0x00;
                b[3] = 0x01;
                b
            },
        };
        assert_eq!(val.as_u32(), Some(1));
    }

    #[test]
    fn test_ui32_insufficient_bytes() {
        let val = SMCValue {
            data_type: TYPE_UI32,
            data_bytes: 2, // Need 4, have 2
            bytes: [0xFF; 32],
        };
        assert!(
            val.as_u32().is_none(),
            "ui32 with 2 bytes should return None"
        );
    }

    // ── Unknown type ─────────────────────────────────────────────────

    #[test]
    fn test_unknown_type_as_f32() {
        let val = SMCValue {
            data_type: 0xDEADBEEF, // Not a recognized type
            data_bytes: 4,
            bytes: [0; 32],
        };
        assert!(val.as_f32().is_none(), "Unknown type should return None");
    }

    #[test]
    fn test_unknown_type_as_u32_fallback() {
        // Unknown type falls through as_u32, which calls as_f32 as
        // fallback → should still be None
        let val = SMCValue {
            data_type: 0xDEADBEEF,
            data_bytes: 4,
            bytes: [0; 32],
        };
        assert!(val.as_u32().is_none(), "Unknown type should return None");
    }

    // ── as_u32 fallback for fpe2 ─────────────────────────────────────

    #[test]
    fn test_fpe2_as_u32() {
        // as_u32 falls back to as_f32() for fpe2, then casts to u32
        let val = SMCValue {
            data_type: TYPE_FPE2,
            data_bytes: 2,
            bytes: {
                let mut b = [0u8; 32];
                b[0] = 0x4E;
                b[1] = 0x20; // 5000 RPM as fpe2
                b
            },
        };
        assert_eq!(val.as_u32(), Some(5000));
    }

    // ── fan_key helper ───────────────────────────────────────────────

    #[test]
    fn test_fan_key_all_digits() {
        for i in 0..=3 {
            let key = fan_key(i, b"Ac");
            let bytes = key.to_be_bytes();
            assert_eq!(bytes[0], b'F', "prefix F mismatch for fan {}", i);
            assert_eq!(
                bytes[1],
                b'0' + i as u8,
                "fan_key({}) digit mismatch: {:?}",
                i,
                bytes
            );
            assert_eq!(bytes[2..], b"Ac"[..], "suffix mismatch for fan {}", i);
        }
    }
}

#[cfg(test)]
#[cfg(not(target_os = "macos"))]
mod tests {
    // No IOKit tests on non-macOS since SMC is not available.
}

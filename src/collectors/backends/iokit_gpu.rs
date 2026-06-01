//! IOKit GPU metadata backend.
//!
//! Reads Apple GPU-related IORegistry properties when available. The backend
//! does not invent values: if no known property is found it returns `None`.

use crate::types::GpuInfo;

type CfTypeRef = *const std::ffi::c_void;
type CfStringRef = CfTypeRef;
type CfMutableDictionaryRef = *mut std::ffi::c_void;
type IoIterator = libc::c_uint;
type IoObject = libc::c_uint;
type KernReturn = libc::c_int;

const KERN_SUCCESS: KernReturn = 0;
const K_CF_ALLOCATOR_DEFAULT: CfTypeRef = std::ptr::null();
const K_CF_STRING_ENCODING_UTF8: u32 = 0x08000100;
const K_CF_NUMBER_SINT32: u32 = 3;
const K_CF_NUMBER_SINT64: u32 = 4;

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFStringCreateWithCString(
        allocator: CfTypeRef,
        cStr: *const libc::c_char,
        encoding: u32,
    ) -> CfStringRef;
    fn CFRelease(obj: CfTypeRef);
    fn CFGetTypeID(obj: CfTypeRef) -> usize;
    fn CFNumberGetTypeID() -> usize;
    fn CFArrayGetTypeID() -> usize;
    fn CFNumberGetValue(number: CfTypeRef, theType: u32, valuePtr: *mut std::ffi::c_void) -> u8;
    fn CFArrayGetCount(theArray: CfTypeRef) -> isize;
    fn CFArrayGetValueAtIndex(theArray: CfTypeRef, idx: isize) -> CfTypeRef;
}

#[link(name = "IOKit", kind = "framework")]
extern "C" {
    fn IOServiceMatching(name: *const libc::c_char) -> CfMutableDictionaryRef;
    fn IOServiceGetMatchingServices(
        master_port: libc::c_uint,
        matching: CfMutableDictionaryRef,
        existing: *mut IoIterator,
    ) -> KernReturn;
    fn IOIteratorNext(iterator: IoIterator) -> IoObject;
    fn IOObjectRelease(object: IoObject) -> KernReturn;
    fn IORegistryEntryCreateCFProperty(
        entry: IoObject,
        key: CfStringRef,
        allocator: CfTypeRef,
        options: u32,
    ) -> CfTypeRef;
}

fn cf_release(obj: CfTypeRef) {
    if !obj.is_null() {
        unsafe { CFRelease(obj) }
    }
}

fn cf_string(value: &str) -> Option<CfStringRef> {
    let c_string = std::ffi::CString::new(value).ok()?;
    let cf = unsafe {
        CFStringCreateWithCString(
            K_CF_ALLOCATOR_DEFAULT,
            c_string.as_ptr(),
            K_CF_STRING_ENCODING_UTF8,
        )
    };
    (!cf.is_null()).then_some(cf)
}

fn cf_number_to_u64(value: CfTypeRef) -> Option<u64> {
    if value.is_null() || unsafe { CFGetTypeID(value) != CFNumberGetTypeID() } {
        return None;
    }

    let mut out_i64 = 0i64;
    let ok = unsafe {
        CFNumberGetValue(
            value,
            K_CF_NUMBER_SINT64,
            &mut out_i64 as *mut _ as *mut std::ffi::c_void,
        )
    };
    if ok != 0 && out_i64 >= 0 {
        return Some(out_i64 as u64);
    }

    let mut out_i32 = 0i32;
    let ok = unsafe {
        CFNumberGetValue(
            value,
            K_CF_NUMBER_SINT32,
            &mut out_i32 as *mut _ as *mut std::ffi::c_void,
        )
    };
    (ok != 0 && out_i32 >= 0).then_some(out_i32 as u64)
}

fn read_number_property(entry: IoObject, key: &str) -> Option<u64> {
    let key_ref = cf_string(key)?;
    let value =
        unsafe { IORegistryEntryCreateCFProperty(entry, key_ref, K_CF_ALLOCATOR_DEFAULT, 0) };
    cf_release(key_ref);
    let number = cf_number_to_u64(value);
    cf_release(value);
    number
}

fn read_number_array_property(entry: IoObject, key: &str) -> Vec<u64> {
    let Some(key_ref) = cf_string(key) else {
        return Vec::new();
    };
    let value =
        unsafe { IORegistryEntryCreateCFProperty(entry, key_ref, K_CF_ALLOCATOR_DEFAULT, 0) };
    cf_release(key_ref);

    if value.is_null() || unsafe { CFGetTypeID(value) != CFArrayGetTypeID() } {
        cf_release(value);
        return Vec::new();
    }

    let count = unsafe { CFArrayGetCount(value) };
    let mut numbers = Vec::new();
    for index in 0..count {
        let item = unsafe { CFArrayGetValueAtIndex(value, index) };
        if let Some(number) = cf_number_to_u64(item) {
            numbers.push(number);
        }
    }
    cf_release(value);
    numbers
}

fn normalize_frequency_to_mhz(value: u64) -> i32 {
    if value > 10_000_000 {
        (value / 1_000_000) as i32
    } else if value > 10_000 {
        (value / 1_000) as i32
    } else {
        value as i32
    }
}

fn update_from_entry(entry: IoObject, info: &mut GpuInfo) {
    for key in [
        "gpu-core-count",
        "gpu-core-counts",
        "num-gpu-cores",
        "GPUCores",
    ] {
        if info.core_count == 0 {
            if let Some(value) = read_number_property(entry, key) {
                info.core_count = value as usize;
            }
        }
    }

    for key in [
        "max-gpu-frequency",
        "gpu-max-frequency",
        "max-frequency",
        "GPU Max Frequency",
    ] {
        if info.max_freq_mhz == 0 {
            if let Some(value) = read_number_property(entry, key) {
                info.max_freq_mhz = normalize_frequency_to_mhz(value);
            }
        }
    }

    for key in [
        "gpu-frequency-table",
        "gpu-frequencies",
        "frequency-table",
        "performance-states",
    ] {
        for value in read_number_array_property(entry, key) {
            let mhz = normalize_frequency_to_mhz(value);
            if mhz > 0 && !info.frequency_table_mhz.contains(&mhz) {
                info.frequency_table_mhz.push(mhz);
            }
        }
    }

    info.frequency_table_mhz.sort_unstable();
    if let Some(max_freq) = info.frequency_table_mhz.last().copied() {
        info.max_freq_mhz = info.max_freq_mhz.max(max_freq);
    }
}

fn scan_service(service_name: &'static std::ffi::CStr, info: &mut GpuInfo) {
    let matching = unsafe { IOServiceMatching(service_name.as_ptr()) };
    if matching.is_null() {
        return;
    }

    let mut iterator = 0;
    let result = unsafe { IOServiceGetMatchingServices(0, matching, &mut iterator) };
    if result != KERN_SUCCESS || iterator == 0 {
        return;
    }

    loop {
        let entry = unsafe { IOIteratorNext(iterator) };
        if entry == 0 {
            break;
        }
        update_from_entry(entry, info);
        unsafe { IOObjectRelease(entry) };
    }

    unsafe { IOObjectRelease(iterator) };
}

pub fn read_iokit_gpu_info() -> Option<GpuInfo> {
    let mut info = GpuInfo::default();
    for service in [c"AGXAccelerator", c"IOAccelerator", c"IOGPU"] {
        scan_service(service, &mut info);
    }

    (info.core_count > 0 || info.max_freq_mhz > 0 || !info.frequency_table_mhz.is_empty())
        .then_some(info)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_frequency_to_mhz() {
        assert_eq!(
            normalize_frequency_to_mhz(1_398_000_000),
            1398,
            "Hz values should be converted to MHz"
        );
        assert_eq!(
            normalize_frequency_to_mhz(1_398_000),
            1398,
            "kHz values should be converted to MHz"
        );
        assert_eq!(
            normalize_frequency_to_mhz(1398),
            1398,
            "MHz values should remain unchanged"
        );
    }
}

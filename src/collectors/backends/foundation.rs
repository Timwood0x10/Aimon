//! Foundation/NSProcessInfo thermal state backend
//!
//! Queries `[NSProcessInfo processInfo].thermalState` to get the system
//! thermal state (Nominal/Fair/Serious/Critical) from a public Apple API.
//!
//! Uses `objc2-foundation` for safe Objective-C interop. Falls back to
//! returning `None` if the Objective-C runtime cannot be reached (e.g. on
//! non-macOS targets or if the class/method is unexpectedly unavailable).

use crate::types::ThermalState;

/// Read the current thermal state from NSProcessInfo.
///
/// Returns `None` if the call fails or is not available (non-macOS,
/// runtime error, etc.).
pub fn read_thermal_state() -> Option<ThermalState> {
    // On non-macOS targets, Foundation is always unavailable
    #[cfg(not(target_os = "macos"))]
    {
        return None;
    }

    #[cfg(target_os = "macos")]
    {
        _read_thermal_state_impl()
    }
}

#[cfg(target_os = "macos")]
fn _read_thermal_state_impl() -> Option<ThermalState> {
    use objc2_foundation::NSProcessInfo;

    // NSProcessInfo::processInfo() is a class method that returns
    // a retained reference. It never fails on macOS.
    let info = NSProcessInfo::processInfo();
    let objc_state = info.thermalState();

    map_thermal_state(objc_state)
}

/// Map the objc2 NSProcessInfoThermalState enum to our crate's ThermalState.
///
/// Extracted into a separate function so it can be unit-tested with
/// each variant value.
#[cfg(target_os = "macos")]
fn map_thermal_state(
    objc_state: objc2_foundation::NSProcessInfoThermalState,
) -> Option<ThermalState> {
    use objc2_foundation::NSProcessInfoThermalState;
    match objc_state {
        NSProcessInfoThermalState::Nominal => Some(ThermalState::Nominal),
        NSProcessInfoThermalState::Fair => Some(ThermalState::Fair),
        NSProcessInfoThermalState::Serious => Some(ThermalState::Serious),
        NSProcessInfoThermalState::Critical => Some(ThermalState::Critical),
        _ => {
            // Unknown value — fall back gracefully
            log::warn!("Unknown NSProcessInfoThermalState value: {:?}", objc_state);
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Integration test ────────────────────────────────────────────
    // Only runs on macOS where NSProcessInfo is available.

    #[cfg(target_os = "macos")]
    #[test]
    fn test_read_thermal_state_returns_some_on_macos() {
        let state = read_thermal_state();
        assert!(
            state.is_some(),
            "NSProcessInfo::thermalState() should return a valid state on macOS"
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_read_thermal_state_is_valid_enum() {
        let state = read_thermal_state().unwrap();
        // ThermalState only has 4 variants — any return value must be one of them
        match state {
            ThermalState::Nominal
            | ThermalState::Fair
            | ThermalState::Serious
            | ThermalState::Critical => {}
        }
    }

    // ── Non-macOS path ──────────────────────────────────────────────

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn test_read_thermal_state_returns_none_on_non_macos() {
        assert_eq!(
            read_thermal_state(),
            None,
            "should return None outside macOS"
        );
    }

    // ── Mapping unit tests ──────────────────────────────────────────
    // Construct each objc2 enum variant and verify the mapping.

    #[cfg(target_os = "macos")]
    #[test]
    fn test_map_nominal() {
        let result = map_thermal_state(objc2_foundation::NSProcessInfoThermalState::Nominal);
        assert_eq!(result, Some(ThermalState::Nominal));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_map_fair() {
        let result = map_thermal_state(objc2_foundation::NSProcessInfoThermalState::Fair);
        assert_eq!(result, Some(ThermalState::Fair));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_map_serious() {
        let result = map_thermal_state(objc2_foundation::NSProcessInfoThermalState::Serious);
        assert_eq!(result, Some(ThermalState::Serious));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_map_critical() {
        let result = map_thermal_state(objc2_foundation::NSProcessInfoThermalState::Critical);
        assert_eq!(result, Some(ThermalState::Critical));
    }

    // ── Correctness checks ──────────────────────────────────────────

    #[test]
    fn test_thermal_state_display_fmt() {
        // Verify all 4 thermal states produce non-empty Display output
        assert!(!ThermalState::Nominal.to_string().is_empty());
        assert!(!ThermalState::Fair.to_string().is_empty());
        assert!(!ThermalState::Serious.to_string().is_empty());
        assert!(!ThermalState::Critical.to_string().is_empty());
    }
}

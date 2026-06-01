//! Temperature data collector
//! Handles collection of temperature sensor readings
//!
//! Priority: SMC (via IOKit) → IOHID → sysinfo::Components

use crate::types::*;
use sysinfo::Components;

/// Collect temperature sensor information
///
/// Tries SMC first for richest sensor data on macOS; if SMC returns
/// nothing, tries IOHID HID event system; falls back to
/// sysinfo::Components.
pub fn collect_temperature_info(components: &Components) -> Vec<TemperatureInfo> {
    // ── 1. SMC (preferred, direct IOKit access, per-key enumeration) ───
    #[cfg(target_os = "macos")]
    {
        let smc_temps = crate::collectors::backends::smc::read_smc_temperatures();
        if !smc_temps.is_empty() {
            return smc_temps
                .into_iter()
                .map(|t| TemperatureInfo {
                    label: format!("{} [SMC]", t.label),
                    temperature: t.value_celsius,
                    critical_temperature: 100.0,
                })
                .collect();
        }
    }

    // ── 2. IOHID (HID event system, covers sensors SMC may miss) ────
    #[cfg(target_os = "macos")]
    {
        let iohid_temps = crate::collectors::backends::iohid::read_iohid_temperatures();
        if !iohid_temps.is_empty() {
            return iohid_temps
                .into_iter()
                .map(|t| TemperatureInfo {
                    label: format!("{} [IOHID]", t.label),
                    temperature: t.value_celsius,
                    critical_temperature: 100.0,
                })
                .collect();
        }
    }

    // ── 3. sysinfo (fallback, works on all platforms) ─────────────────
    components
        .iter()
        .map(|component| TemperatureInfo {
            label: component.label().to_string(),
            temperature: component.temperature(),
            critical_temperature: component.critical().unwrap_or(100.0),
        })
        .collect()
}

//! Temperature data collector
//! Handles collection of temperature sensor readings

use crate::types::*;
use sysinfo::Components;

/// Collect temperature sensor information
pub fn collect_temperature_info(components: &Components) -> Vec<TemperatureInfo> {
    components
        .iter()
        .map(|component| {
            TemperatureInfo {
                label: component.label().to_string(),
                temperature: component.temperature(),
                critical_temperature: component.critical().unwrap_or(100.0),
            }
        })
        .collect()
}
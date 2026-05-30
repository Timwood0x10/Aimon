//! Thermal data collector
//! Handles collection of thermal and fan information

use crate::types::*;

/// Collect thermal information from the system
pub async fn collect_thermal_info() -> ThermalInfo {
    let mut thermal_info = ThermalInfo::default();
    
    // Get real thermal pressure from system
    if let Ok(output) = tokio::process::Command::new("sysctl")
        .arg("-n")
        .arg("machdep.xcpm.cpu_thermal_level")
        .output()
        .await
    {
        if let Ok(thermal_str) = String::from_utf8(output.stdout) {
            if let Ok(thermal_level) = thermal_str.trim().parse::<u8>() {
                thermal_info.thermal_pressure = thermal_level * 10; // Convert to percentage
            }
        }
    }
    
    // Check for thermal throttling via CPU frequency scaling
    thermal_info.thermal_throttling = thermal_info.thermal_pressure > 50;
    
    // Get fan speeds from powermetrics if available
    if let Ok(output) = tokio::process::Command::new("powermetrics")
        .arg("--samplers")
        .arg("smc")
        .arg("-n")
        .arg("1")
        .arg("--show-initial-usage")
        .output()
        .await
    {
        if let Ok(power_str) = String::from_utf8(output.stdout) {
            let mut fan_speeds = Vec::new();
            for line in power_str.lines() {
                if line.contains("Fan") && line.contains("RPM") {
                    if let Some(rpm_str) = line.split_whitespace().find(|s| s.ends_with("RPM")) {
                        if let Ok(rpm) = rpm_str.trim_end_matches("RPM").parse::<u32>() {
                            fan_speeds.push(rpm);
                        }
                    }
                }
            }
            if !fan_speeds.is_empty() {
                thermal_info.fan_speeds = fan_speeds;
            }
        }
    }
    
    // Estimate heat dissipation based on thermal pressure
    thermal_info.heat_dissipation_rate = match thermal_info.thermal_pressure {
        0..=20 => 5.0,
        21..=40 => 10.0,
        41..=60 => 15.0,
        61..=80 => 20.0,
        _ => 25.0,
    };
    
    thermal_info
}
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

    if let Ok(Ok(output)) = tokio::time::timeout(
        std::time::Duration::from_millis(900),
        tokio::process::Command::new("powermetrics")
            .args(["--samplers", "smc", "-n", "1", "--show-initial-usage"])
            .output(),
    )
    .await
    {
        if let Ok(power_str) = String::from_utf8(output.stdout) {
            thermal_info.fan_speeds = parse_fan_speeds(&power_str);
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

fn parse_fan_speeds(output: &str) -> Vec<u32> {
    output
        .lines()
        .filter(|line| line.to_ascii_lowercase().contains("fan"))
        .filter_map(|line| {
            line.split_whitespace().find_map(|part| {
                part.trim_end_matches("RPM")
                    .trim_end_matches("rpm")
                    .parse::<u32>()
                    .ok()
            })
        })
        .collect()
}

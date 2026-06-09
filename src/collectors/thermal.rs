//! Thermal data collector
//! Handles collection of thermal and fan information

use crate::collectors::backends::foundation;
use crate::config::FanControlConfig;
use crate::types::*;

/// Collect thermal information from the system
pub async fn collect_thermal_info(fan_control: &FanControlConfig) -> ThermalInfo {
    let mut thermal_info = ThermalInfo {
        fan_control_status: fan_control_status(fan_control),
        ..Default::default()
    };

    // ── Thermal State (via Foundation/NSProcessInfo) ────────────────
    // Try the public Apple API first; falls back gracefully.
    if let Some(state) = foundation::read_thermal_state() {
        thermal_info.thermal_state = state;
        thermal_info.state_meta = MetricMeta::measured(MetricSource::Foundation);
    } else {
        // Fallback: mark as unavailable
        thermal_info.state_meta = MetricMeta::unavailable();
    }

    // ── Thermal Pressure (via sysctl) ───────────────────────────────
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

    // ── Fan Speeds (prefer SMC, fallback powermetrics) ──────────────
    // SMC provides direct fan RPM without spawning a subprocess.
    #[cfg(target_os = "macos")]
    let smc_fans = crate::collectors::backends::smc::read_smc_fans();
    #[cfg(not(target_os = "macos"))]
    let smc_fans: Vec<FanInfo> = Vec::new();

    if !smc_fans.is_empty() {
        thermal_info.fans = smc_fans;
        thermal_info.fan_speeds = thermal_info.fans.iter().map(|f| f.current_rpm).collect();
        thermal_info.fan_meta = MetricMeta::measured(MetricSource::Smc);
        apply_fan_control_if_enabled(&mut thermal_info, fan_control);
    } else if let Ok(Ok(output)) = tokio::time::timeout(
        std::time::Duration::from_millis(900),
        tokio::process::Command::new("powermetrics")
            .args(["--samplers", "smc", "-n", "1", "--show-initial-usage"])
            .output(),
    )
    .await
    {
        if let Ok(power_str) = String::from_utf8(output.stdout) {
            thermal_info.fan_speeds = parse_fan_speeds(&power_str);
            if !thermal_info.fan_speeds.is_empty() {
                thermal_info.fan_meta = MetricMeta::measured(MetricSource::Powermetrics);
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

fn fan_control_status(fan_control: &FanControlConfig) -> String {
    if !fan_control.enabled {
        "Disabled".to_string()
    } else if !fan_control.runtime_allowed {
        "Needs --allow-fan-control".to_string()
    } else {
        "Armed".to_string()
    }
}

fn apply_fan_control_if_enabled(thermal_info: &mut ThermalInfo, fan_control: &FanControlConfig) {
    if !fan_control.enabled || !fan_control.runtime_allowed {
        return;
    }

    #[cfg(target_os = "macos")]
    {
        let result = if let Some(target_rpm) = fan_control.target_rpm {
            crate::collectors::backends::smc::set_smc_fans_target_rpm(
                target_rpm,
                fan_control.safe_min_rpm,
                fan_control.safe_max_rpm,
            )
        } else if fan_control.restore_on_exit {
            crate::collectors::backends::smc::set_smc_fans_auto()
        } else {
            Ok(())
        };
        thermal_info.fan_control_status = match result {
            Ok(()) if fan_control.target_rpm.is_some() => {
                format!("Manual {} RPM", fan_control.target_rpm.unwrap_or_default())
            }
            Ok(()) if fan_control.restore_on_exit => "Auto mode restored".to_string(),
            Ok(()) => "Armed".to_string(),
            Err(error) => format!("SMC write failed: {}", error),
        };
    }

    #[cfg(not(target_os = "macos"))]
    {
        thermal_info.fan_control_status = "Unavailable on this OS".to_string();
    }
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

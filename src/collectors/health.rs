//! System health collector
//! Handles collection of system health and uptime information

use crate::types::*;

/// Collect system health information
pub async fn collect_system_health() -> SystemHealthInfo {
    // Get real system health data
    let mut health_info = SystemHealthInfo::default();
    
    // Get real uptime
    if let Ok(output) = tokio::process::Command::new("sysctl")
        .arg("-n")
        .arg("kern.boottime")
        .output()
        .await
    {
        if let Ok(boottime_str) = String::from_utf8(output.stdout) {
            // Parse boottime and calculate uptime
            if let Some(timestamp_str) = boottime_str.split_whitespace().nth(3) {
                if let Ok(boot_timestamp) = timestamp_str.trim_end_matches(',').parse::<i64>() {
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs() as i64;
                    health_info.uptime_seconds = (now - boot_timestamp) as u64;
                }
            }
        }
    }
    
    // Get real load averages
    if let Ok(output) = tokio::process::Command::new("sysctl")
        .arg("-n")
        .arg("vm.loadavg")
        .output()
        .await
    {
        if let Ok(loadavg_str) = String::from_utf8(output.stdout) {
            // Parse "{ 2.66 2.75 2.97 }" format
            let loads: Vec<f64> = loadavg_str
                .trim()
                .trim_start_matches('{')
                .trim_end_matches('}')
                .split_whitespace()
                .filter_map(|s| s.parse().ok())
                .collect();
            
            if loads.len() >= 3 {
                health_info.system_load_1min = loads[0];
                health_info.system_load_5min = loads[1];
                health_info.system_load_15min = loads[2];
            }
        }
    }
    
    // Calculate power quality score based on load and other factors
    let avg_load = (health_info.system_load_1min + health_info.system_load_5min + health_info.system_load_15min) / 3.0;
    health_info.power_quality_score = if avg_load < 1.0 {
        95
    } else if avg_load < 2.0 {
        85
    } else if avg_load < 3.0 {
        75
    } else {
        65
    };
    
    // Estimate sleep/wake efficiency (simplified)
    health_info.sleep_wake_efficiency = if avg_load < 1.5 { 95.0 } else { 85.0 };
    
    health_info
}
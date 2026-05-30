//! Thunderbolt data collector
//! Collects Thunderbolt bus and device information using system_profiler

use crate::types::{ThunderboltBus, ThunderboltDevice, ThunderboltInfo};

/// Collect Thunderbolt information
pub async fn collect_thunderbolt_info() -> ThunderboltInfo {
    let mut info = ThunderboltInfo::default();

    // Use system_profiler to get Thunderbolt bus information
    if let Ok(output) = get_thunderbolt_info().await {
        parse_thunderbolt_info(&output, &mut info);
    }

    info
}

async fn get_thunderbolt_info() -> Result<String, Box<dyn std::error::Error>> {
    let output = tokio::process::Command::new("system_profiler")
        .args(["SPThunderboltDataType"])
        .output()
        .await?;

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn parse_thunderbolt_info(output: &str, info: &mut ThunderboltInfo) {
    let mut current_bus: Option<ThunderboltBus> = None;

    for line in output.lines() {
        let trimmed = line.trim();

        // Detect new bus entry
        if trimmed.contains("Thunderbolt Bus") || trimmed.contains("USB Bus") {
            // Save previous bus if any
            if let Some(bus) = current_bus.take() {
                info.buses.push(bus);
            }
            current_bus = Some(ThunderboltBus {
                name: trimmed.to_string(),
                status: "Unknown".to_string(),
                speed: "Unknown".to_string(),
                devices: Vec::new(),
                rx_bytes_per_sec: 0.0,
                tx_bytes_per_sec: 0.0,
            });
        }

        // Parse bus details
        if let Some(ref mut bus) = current_bus {
            if trimmed.starts_with("Status:") {
                bus.status = trimmed.trim_start_matches("Status:").trim().to_string();
            }
            if trimmed.starts_with("Speed:") || trimmed.starts_with("Maximum Speed:") {
                bus.speed = trimmed.split(':').nth(1)
                    .map(|s| s.trim().to_string())
                    .unwrap_or_else(|| "Unknown".to_string());
            }
            if trimmed.starts_with("Device:") || trimmed.contains("Vendor ID:") {
                // Simple device detection
                let name = if trimmed.starts_with("Device:") {
                    trimmed.trim_start_matches("Device:").trim().to_string()
                } else {
                    "Unknown Device".to_string()
                };
                bus.devices.push(ThunderboltDevice {
                    name,
                    vendor: String::new(),
                    mode: String::new(),
                    speed: bus.speed.clone(),
                });
            }
        }
    }

    // Don't forget the last bus
    if let Some(bus) = current_bus.take() {
        info.buses.push(bus);
    }
}
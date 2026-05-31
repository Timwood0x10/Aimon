//! Disk I/O data collector
//! Collects disk read/write speeds

use crate::types::DiskIoInfo;
use std::time::Duration;

/// Collect disk I/O information
/// Uses iostat command for accurate per-second rates
pub async fn collect_disk_io_info() -> DiskIoInfo {
    let mut info = DiskIoInfo::default();

    // Try iostat for real-time disk I/O
    let output = tokio::time::timeout(
        Duration::from_secs(2),
        tokio::process::Command::new("iostat")
            .args(["-c", "2", "-w", "1"])
            .output(),
    )
    .await;

    if let Ok(Ok(output)) = output {
        let text = String::from_utf8_lossy(&output.stdout);
        // Parse iostat output for KB/t and tps
        for line in text.lines().skip(2) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                if let Ok(kbs) = parts[3].parse::<f64>() {
                    info.read_bytes_per_sec = kbs * 1024.0;
                }
                if let Ok(kbs) = parts[4].parse::<f64>() {
                    info.write_bytes_per_sec = kbs * 1024.0;
                }
            }
        }
    }

    info
}

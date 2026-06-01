//! Disk usage collector.
//! Collects mounted volume capacity with `sysinfo::Disks` so the UI can show
//! storage pressure independently from real-time disk I/O throughput.

use crate::types::DiskUsageInfo;
use sysinfo::Disks;

/// Collect capacity usage for mounted disks and volumes.
pub fn collect_disk_usage_info() -> Vec<DiskUsageInfo> {
    let disks = Disks::new_with_refreshed_list();
    disks
        .iter()
        .filter_map(|disk| {
            let total_bytes = disk.total_space();
            if total_bytes == 0 {
                return None;
            }

            let available_bytes = disk.available_space();
            let used_bytes = total_bytes.saturating_sub(available_bytes);
            let usage_percentage = used_bytes as f32 / total_bytes as f32 * 100.0;

            Some(DiskUsageInfo {
                name: disk.name().to_string_lossy().to_string(),
                mount_point: disk.mount_point().to_string_lossy().to_string(),
                file_system: disk.file_system().to_string_lossy().to_string(),
                total_bytes,
                available_bytes,
                used_bytes,
                usage_percentage,
                is_removable: disk.is_removable(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Objective: Verify collected disk percentages obey capacity invariants.
    /// Invariants: Used bytes never exceed total bytes and percentage is 0-100.
    #[test]
    fn test_disk_usage_capacity_invariants() {
        for disk in collect_disk_usage_info() {
            assert!(
                disk.used_bytes <= disk.total_bytes,
                "disk used bytes should not exceed total for {}",
                disk.mount_point
            );
            assert!(
                (0.0..=100.0).contains(&disk.usage_percentage),
                "disk usage percentage should stay within 0..=100 for {}",
                disk.mount_point
            );
        }
    }
}

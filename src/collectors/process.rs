//! Process data collector
//! Handles collection of running process information

use crate::types::*;
use sysinfo::System;

/// Collect process information
pub fn collect_process_info(system: &System) -> Vec<ProcessInfo> {
    system
        .processes()
        .iter()
        .map(|(pid, process)| ProcessInfo {
            pid: *pid,
            name: process.name().to_string(),
            cpu_usage: process.cpu_usage(),
            memory_usage: process.memory(),
            disk_read_bytes: process.disk_usage().read_bytes,
            disk_write_bytes: process.disk_usage().written_bytes,
        })
        .collect()
}
//! Memory data collector
//! Handles collection of memory usage and swap information

use crate::types::*;
use sysinfo::System;

/// Collect memory information
pub fn collect_memory_info(system: &System) -> MemoryInfo {
    let total_memory = system.total_memory();
    let used_memory = system.used_memory();
    let available_memory = system.available_memory();
    let total_swap = system.total_swap();
    let used_swap = system.used_swap();

    let usage_percentage = if total_memory > 0 {
        (used_memory as f64 / total_memory as f64 * 100.0) as u16
    } else {
        0
    };

    MemoryInfo {
        total_memory,
        used_memory,
        available_memory,
        total_swap,
        used_swap,
        usage_percentage,
    }
}

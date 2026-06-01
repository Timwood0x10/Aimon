//! Mach CPU usage backend
//!
//! Uses the Mach `host_processor_info` API via FFI to read per-core
//! CPU tick deltas without spawning external commands. Falls back to
//! sysinfo if Mach calls fail.

use libc::{c_int, c_uint, mach_msg_type_number_t, natural_t, processor_info_array_t};
use std::mem;
use std::sync::Mutex;

// Mach constants
const _HOST_PROCESSOR_INFO: c_int = 43;
const PROCESSOR_CPU_LOAD_INFO: c_int = 2;
const CPU_STATE_MAX: usize = 4;
const CPU_STATE_USER: usize = 0;
const CPU_STATE_SYSTEM: usize = 1;
const CPU_STATE_IDLE: usize = 2;
const CPU_STATE_NICE: usize = 3;

type MachPort = c_uint;
type _ProcessorInfoArray = *mut c_int;

extern "C" {
    fn mach_host_self() -> MachPort;
    fn host_processor_info(
        host: MachPort,
        flavor: c_int,
        out_processor_count: *mut natural_t,
        out_processor_info: *mut processor_info_array_t,
        out_processor_info_cnt: *mut mach_msg_type_number_t,
    ) -> c_int;
    fn vm_deallocate(task: MachPort, addr: usize, size: usize) -> c_int;
    fn mach_task_self() -> MachPort;
}

/// Per-core CPU tick snapshot
#[derive(Debug, Clone, Default)]
pub struct CpuTickSnapshot {
    pub user: u64,
    pub system: u64,
    pub idle: u64,
    pub nice: u64,
}

/// Mach CPU sampler that tracks deltas between samples
pub struct MachCpuSampler {
    previous: Mutex<Option<Vec<CpuTickSnapshot>>>,
    processor_count: usize,
}

impl MachCpuSampler {
    pub fn new() -> Self {
        let count = Self::detect_processor_count().unwrap_or(0);
        Self {
            previous: Mutex::new(None),
            processor_count: count,
        }
    }

    /// Returns per-core CPU usage percentages [0.0, 100.0] using tick deltas
    pub fn sample_usage(&self) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        let current = Self::read_ticks()?;
        let mut prev = self.previous.lock().map_err(|e| e.to_string())?;

        let usage = if let Some(ref prev_ticks) = *prev {
            current
                .iter()
                .zip(prev_ticks.iter())
                .map(|(cur, prv)| {
                    let d_total = ((cur.user + cur.system + cur.idle + cur.nice) as i64
                        - (prv.user + prv.system + prv.idle + prv.nice) as i64)
                        .max(1) as f64;
                    let d_busy = ((cur.user + cur.system + cur.nice) as i64
                        - (prv.user + prv.system + prv.nice) as i64)
                        .max(0) as f64;
                    ((d_busy / d_total) * 100.0) as f32
                })
                .collect()
        } else {
            vec![0.0f32; current.len()]
        };

        *prev = Some(current);
        Ok(usage)
    }

    pub fn processor_count(&self) -> usize {
        self.processor_count
    }

    fn read_ticks() -> Result<Vec<CpuTickSnapshot>, Box<dyn std::error::Error>> {
        let mut processor_count: natural_t = 0;
        let mut info: processor_info_array_t = std::ptr::null_mut();
        let mut info_count: mach_msg_type_number_t = 0;

        let host = unsafe { mach_host_self() };
        let ret = unsafe {
            host_processor_info(
                host,
                PROCESSOR_CPU_LOAD_INFO,
                &mut processor_count,
                &mut info,
                &mut info_count,
            )
        };

        if ret != 0 {
            return Err(format!("host_processor_info returned {}", ret).into());
        }

        let ticks_per_proc = CPU_STATE_MAX;
        let total_ticks = processor_count as usize * ticks_per_proc;

        let snapshots: Vec<CpuTickSnapshot> = unsafe {
            let slice = std::slice::from_raw_parts(info as *const c_int, total_ticks);
            (0..processor_count as usize)
                .map(|i| {
                    let base = i * ticks_per_proc;
                    CpuTickSnapshot {
                        user: slice[base + CPU_STATE_USER] as u64,
                        system: slice[base + CPU_STATE_SYSTEM] as u64,
                        idle: slice[base + CPU_STATE_IDLE] as u64,
                        nice: slice[base + CPU_STATE_NICE] as u64,
                    }
                })
                .collect()
        };

        // Deallocate the Mach-allocated array
        let task = unsafe { mach_task_self() };
        unsafe {
            vm_deallocate(
                task,
                info as usize,
                info_count as usize * mem::size_of::<c_int>(),
            );
        }

        Ok(snapshots)
    }

    fn detect_processor_count() -> Result<usize, Box<dyn std::error::Error>> {
        let mut processor_count: natural_t = 0;
        let mut info: processor_info_array_t = std::ptr::null_mut();
        let mut info_count: mach_msg_type_number_t = 0;

        let host = unsafe { mach_host_self() };
        let ret = unsafe {
            host_processor_info(
                host,
                PROCESSOR_CPU_LOAD_INFO,
                &mut processor_count,
                &mut info,
                &mut info_count,
            )
        };

        if ret != 0 {
            return Err(format!("host_processor_info returned {}", ret).into());
        }

        let task = unsafe { mach_task_self() };
        unsafe {
            vm_deallocate(
                task,
                info as usize,
                info_count as usize * mem::size_of::<c_int>(),
            );
        }

        Ok(processor_count as usize)
    }
}

impl Default for MachCpuSampler {
    fn default() -> Self {
        Self::new()
    }
}

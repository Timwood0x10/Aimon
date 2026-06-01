//! Backend data source modules for system metrics
//!
//! Each backend encapsulates a specific data source (powermetrics, Mach, SMC, etc.)
//! with a consistent interface for collecting metrics. Backends that depend on
//! Apple private APIs are gated behind `#[cfg(target_os = "macos")]` and fall
//! back gracefully to public alternatives.

pub mod powermetrics;
pub mod sysinfo_backend;

#[cfg(target_os = "macos")]
pub mod mach;

#[cfg(target_os = "macos")]
pub mod foundation;

#[cfg(target_os = "macos")]
pub mod smc;

#[cfg(target_os = "macos")]
pub mod ioreport;

#[cfg(target_os = "macos")]
pub mod iohid;

#[cfg(target_os = "macos")]
pub mod iokit_gpu;

// Re-export common helpers
pub use powermetrics::PowermetricsSampler;
pub use sysinfo_backend::SysinfoCollector;

#[cfg(target_os = "macos")]
pub use mach::MachCpuSampler;

#[cfg(target_os = "macos")]
pub use smc::{read_smc_fans, read_smc_temperatures, SmcHandle, SmcTemperatureReading};

#[cfg(target_os = "macos")]
pub use ioreport::{is_available as is_ioreport_available, IoReportPowerSample, IoReportSampler};

#[cfg(target_os = "macos")]
pub use iohid::{
    is_available as is_iohid_available, read_iohid_temperatures, IohidTemperatureReading,
};

#[cfg(target_os = "macos")]
pub use iokit_gpu::read_iokit_gpu_info;

use serde::Serialize;
use std::time::Instant;
use sysinfo::Pid;

#[derive(Default, Debug, Clone, Serialize)]
pub struct CPUMetrics {
    pub e_cluster_active: i32,
    pub p_cluster_active: i32,
    pub e_cluster_freq_mhz: i32,
    pub p_cluster_freq_mhz: i32,
    pub cpu_w: f64,
    pub gpu_w: f64,
    pub ane_w: f64,
    pub dram_w: f64,
    pub package_w: f64,
}

/// GPU information including usage, frequency, and core count
#[derive(Debug, Clone, Serialize)]
pub struct GpuInfo {
    pub usage_percentage: f32,
    pub freq_mhz: i32,
    pub max_freq_mhz: i32,
    pub core_count: usize,
    pub sram_power_w: f64,
    pub tflops: f64,
}

impl Default for GpuInfo {
    fn default() -> Self {
        Self {
            usage_percentage: 0.0,
            freq_mhz: 0,
            max_freq_mhz: 0,
            core_count: 0,
            sram_power_w: 0.0,
            tflops: 0.0,
        }
    }
}

/// ANE (Apple Neural Engine) information
#[derive(Debug, Clone, Serialize)]
pub struct AneInfo {
    pub usage_percentage: f32,
    pub power_w: f64,
}

impl Default for AneInfo {
    fn default() -> Self {
        Self {
            usage_percentage: 0.0,
            power_w: 0.0,
        }
    }
}

/// DRAM bandwidth information
#[derive(Debug, Clone, Serialize)]
pub struct DramInfo {
    pub read_bytes_per_sec: f64,
    pub write_bytes_per_sec: f64,
    pub total_bytes_per_sec: f64,
    pub power_w: f64,
}

impl Default for DramInfo {
    fn default() -> Self {
        Self {
            read_bytes_per_sec: 0.0,
            write_bytes_per_sec: 0.0,
            total_bytes_per_sec: 0.0,
            power_w: 0.0,
        }
    }
}

/// Thunderbolt device information
#[derive(Debug, Clone, Serialize)]
pub struct ThunderboltDevice {
    pub name: String,
    pub vendor: String,
    pub mode: String,
    pub speed: String,
}

/// Thunderbolt bus information
#[derive(Debug, Clone, Serialize)]
pub struct ThunderboltBus {
    pub name: String,
    pub status: String,
    pub speed: String,
    pub devices: Vec<ThunderboltDevice>,
    pub rx_bytes_per_sec: f64,
    pub tx_bytes_per_sec: f64,
}

/// Thunderbolt information
#[derive(Debug, Clone, Default, Serialize)]
pub struct ThunderboltInfo {
    pub buses: Vec<ThunderboltBus>,
}

/// Disk I/O information
#[derive(Debug, Clone, Serialize)]
pub struct DiskIoInfo {
    pub read_bytes_per_sec: f64,
    pub write_bytes_per_sec: f64,
    pub read_ops_per_sec: f64,
    pub write_ops_per_sec: f64,
}

impl Default for DiskIoInfo {
    fn default() -> Self {
        Self {
            read_bytes_per_sec: 0.0,
            write_bytes_per_sec: 0.0,
            read_ops_per_sec: 0.0,
            write_ops_per_sec: 0.0,
        }
    }
}

/// Fan information with detailed stats
#[derive(Debug, Clone, Serialize)]
pub struct FanInfo {
    pub id: usize,
    pub current_rpm: u32,
    pub min_rpm: u32,
    pub max_rpm: u32,
    pub target_rpm: u32,
    pub mode: String, // "Auto" or "Manual"
}

/// Thermal state as reported by macOS
#[derive(Debug, Clone, Serialize)]
pub enum ThermalState {
    Nominal,
    Fair,
    Serious,
    Critical,
}

impl Default for ThermalState {
    fn default() -> Self {
        Self::Nominal
    }
}

impl std::fmt::Display for CPUMetrics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "E-Cluster Active: {}%\nP-Cluster Active: {}%\nE-Cluster Freq: {} MHz\nP-Cluster Freq: {} MHz\nCPU Power: {:.2}W\nGPU Power: {:.2}W\nANE Power: {:.2}W\nDRAM Power: {:.2}W\nPackage Power: {:.2}W",
        self.e_cluster_active,
        self.p_cluster_active,
        self.e_cluster_freq_mhz,
        self.p_cluster_freq_mhz,
        self.cpu_w,
        self.gpu_w,
        self.ane_w,
        self.dram_w,
        self.package_w,
        )
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SystemInfo {
    pub name: String,
    pub kernel_version: String,
    pub os_version: String,
    pub host_name: String,
    pub cpu_arch: String,
    pub cpu_brand: String,
    /// Number of CPU cores
    pub cpu_core_count: usize,
    /// Number of E-cores (efficiency)
    pub e_core_count: usize,
    /// Number of P-cores (performance)
    pub p_core_count: usize,
    /// Number of GPU cores
    pub gpu_core_count: usize,
    /// Chip model name (e.g. "Apple M2 Pro")
    pub chip_name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CpuInfo {
    pub core_usages: Vec<f32>,
    pub average_usage: f32,
    pub power_metrics: CPUMetrics,
}

#[derive(Debug, Clone, Serialize)]
pub struct MemoryInfo {
    pub total_memory: u64,
    pub used_memory: u64,
    pub available_memory: u64,
    pub total_swap: u64,
    pub used_swap: u64,
    pub usage_percentage: u16,
}

#[derive(Debug, Clone, Serialize)]
pub struct NetworkInterface {
    pub name: String,
    pub bytes_received: u64,
    pub bytes_transmitted: u64,
    pub packets_received: u64,
    pub packets_transmitted: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct TemperatureInfo {
    pub label: String,
    pub temperature: f32,
    pub critical_temperature: f32,
}

fn serialize_pid<S: serde::Serializer>(pid: &Pid, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_u64(pid.as_u32() as u64)
}

#[derive(Debug, Clone, Serialize)]
pub struct ProcessInfo {
    #[serde(serialize_with = "serialize_pid")]
    pub pid: Pid,
    pub name: String,
    pub cpu_usage: f32,
    pub memory_usage: u64,
    pub disk_read_bytes: u64,
    pub disk_write_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct BatteryInfo {
    pub percentage: f32,
    pub is_charging: bool,
    pub is_plugged: bool,
    pub health_percentage: f32,
    pub cycle_count: u32,
    pub time_remaining: Option<u32>, // minutes
    pub power_adapter_wattage: f32,
    pub current_capacity: u32, // mAh
    pub design_capacity: u32,  // mAh
    pub voltage: f32,          // V
    pub amperage: f32,         // A
    pub temperature: f32,      // °C
}

impl Default for BatteryInfo {
    fn default() -> Self {
        Self {
            percentage: 0.0,
            is_charging: false,
            is_plugged: false,
            health_percentage: 0.0,
            cycle_count: 0,
            time_remaining: None,
            power_adapter_wattage: 0.0,
            current_capacity: 0,
            design_capacity: 0,
            voltage: 0.0,
            amperage: 0.0,
            temperature: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ThermalInfo {
    pub fans: Vec<FanInfo>,
    pub thermal_state: ThermalState,
    pub thermal_throttling: bool,
    pub heat_dissipation_rate: f32, // W
    pub thermal_pressure: u8,       // 0-100
    /// Legacy fan_speeds for backward compatibility
    pub fan_speeds: Vec<u32>, // RPM
}

impl Default for ThermalInfo {
    fn default() -> Self {
        Self {
            fans: Vec::new(),
            thermal_state: ThermalState::Nominal,
            thermal_throttling: false,
            heat_dissipation_rate: 0.0,
            thermal_pressure: 0,
            fan_speeds: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PerformanceMetrics {
    pub instructions_per_watt: f64,
    pub performance_per_watt: f64,
    pub frequency_efficiency: f64,
    pub workload_type: String, // "compute", "graphics", "mixed", "idle"
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            instructions_per_watt: 0.0,
            performance_per_watt: 0.0,
            frequency_efficiency: 0.0,
            workload_type: "idle".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SystemHealthInfo {
    pub uptime_seconds: u64,
    pub sleep_wake_efficiency: f32, // %
    pub power_quality_score: u8,    // 0-100
    pub system_load_1min: f64,
    pub system_load_5min: f64,
    pub system_load_15min: f64,
}

impl Default for SystemHealthInfo {
    fn default() -> Self {
        Self {
            uptime_seconds: 0,
            sleep_wake_efficiency: 0.0,
            power_quality_score: 0,
            system_load_1min: 0.0,
            system_load_5min: 0.0,
            system_load_15min: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct TerminalInfo {
    pub total_ptmx_count: u32,
    pub local_ptmx_count: u32,
    pub terminal_processes: Vec<TerminalProcess>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TerminalProcess {
    pub pid: u32,
    pub name: String,
    pub pty_count: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct SystemData {
    pub system_info: SystemInfo,
    pub cpu_info: CpuInfo,
    pub gpu_info: GpuInfo,
    pub ane_info: AneInfo,
    pub dram_info: DramInfo,
    pub thunderbolt_info: ThunderboltInfo,
    pub disk_io_info: DiskIoInfo,
    pub memory_info: MemoryInfo,
    pub network_info: Vec<NetworkInterface>,
    pub temperature_info: Vec<TemperatureInfo>,
    pub process_info: Vec<ProcessInfo>,
    pub battery_info: BatteryInfo,
    pub thermal_info: ThermalInfo,
    pub performance_metrics: PerformanceMetrics,
    pub system_health: SystemHealthInfo,
    pub terminal_info: TerminalInfo,
    #[serde(skip)]
    pub timestamp: Instant,
}

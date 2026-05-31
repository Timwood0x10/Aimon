//! Collectors module for system data
//! Contains specialized collectors for different system metrics

pub mod cpu;
pub mod disk_io;
pub mod dram;
pub mod gpu;
pub mod health;
pub mod memory;
pub mod network;
pub mod performance;
pub mod process;
pub mod temperature;
pub mod terminal;
pub mod thermal;
pub mod thunderbolt;

use crate::battery_collector::FastBatteryCollector;
use crate::carbon::tracker::CarbonTracker;
use crate::types::*;
use std::time::{Duration, Instant};
use sysinfo::{Components, CpuRefreshKind, Networks, ProcessRefreshKind, System};

/// Main data collector that coordinates all sub-collectors
pub struct DataCollector {
    system: System,
    networks: Networks,
    components: Components,
    last_powermetrics: Option<Instant>,
    cached_cpu_metrics: Option<CPUMetrics>,
    cached_powermetrics_output: Option<String>,
    powermetrics_cache_duration: Duration,
    battery_collector: FastBatteryCollector,
    cached_system_info: Option<SystemInfo>,
    carbon_tracker: CarbonTracker,
    last_carbon_sample: Option<Instant>,
}

impl Default for DataCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl DataCollector {
    pub fn new() -> Self {
        Self {
            system: System::new_all(),
            networks: Networks::new_with_refreshed_list(),
            components: Components::new_with_refreshed_list(),
            last_powermetrics: None,
            cached_cpu_metrics: None,
            cached_powermetrics_output: None,
            powermetrics_cache_duration: Duration::from_secs(2),
            battery_collector: FastBatteryCollector::new(),
            cached_system_info: None,
            carbon_tracker: CarbonTracker::new(),
            last_carbon_sample: None,
        }
    }

    /// Fast startup version with lazy initialization
    pub fn new_fast() -> Self {
        Self {
            system: System::new(),
            networks: Networks::new(),
            components: Components::new(),
            last_powermetrics: None,
            cached_cpu_metrics: None,
            cached_powermetrics_output: None,
            powermetrics_cache_duration: Duration::from_secs(1),
            battery_collector: FastBatteryCollector::new(),
            cached_system_info: None,
            carbon_tracker: CarbonTracker::new(),
            last_carbon_sample: None,
        }
    }

    /// Collect all system data
    /// Optimized: CPU + parallel collectors run concurrently via tokio::select!
    /// CPU usage is instant (sysinfo), powermetrics uses cache for speed.
    pub async fn collect_all_data(&mut self) -> Result<SystemData, Box<dyn std::error::Error>> {
        self.refresh_realtime_data();

        // ── Sync collectors (fast, no I/O, run instantly) ──────────────
        let system_info = self.collect_system_info();
        let memory_info = self.collect_memory_info();
        let network_info = self.collect_network_info();
        let temperature_info = self.collect_temperature_info();
        let process_info = self.collect_process_info();
        let terminal_info = terminal::collect_terminal_info().await;

        // ── CPU first (uses &mut self, must complete before other self methods) ──
        // Speed: powermetrics uses cache after first run, so this is fast (~1ms)
        let cpu_info = self.collect_cpu_info().await?;
        let ane_info = self.collect_ane_info(&cpu_info);
        let dram_info = self.collect_dram_info(&cpu_info);
        let total_power = cpu_info.power_metrics.package_w;

        // ── Parallel async collectors ─────────────────────────────────
        let (
            mut gpu_info,
            thunderbolt_info,
            disk_io_info,
            thermal_info,
            performance_metrics,
            system_health,
        ) = tokio::join!(
            gpu::collect_gpu_info_from_powermetrics(self.cached_powermetrics_output.as_deref()),
            thunderbolt::collect_thunderbolt_info(),
            disk_io::collect_disk_io_info(),
            self.collect_thermal_info(),
            self.collect_performance_metrics(&cpu_info, total_power),
            self.collect_system_health(),
        );

        gpu::complete_static_gpu_info(&mut gpu_info).await;

        // battery (separate borrow)
        let battery_info = self.battery_collector.get_battery_info().await;
        self.record_carbon_sample(total_power);
        let carbon_info = self.carbon_tracker.clone();

        Ok(SystemData {
            system_info,
            cpu_info,
            gpu_info,
            ane_info,
            dram_info,
            thunderbolt_info,
            disk_io_info,
            memory_info,
            network_info,
            temperature_info,
            process_info,
            battery_info,
            thermal_info,
            performance_metrics,
            system_health,
            terminal_info,
            carbon_info,
            timestamp: Instant::now(),
        })
    }

    fn record_carbon_sample(&mut self, watts: f64) {
        let now = Instant::now();
        let elapsed_secs = self
            .last_carbon_sample
            .map(|last| now.duration_since(last).as_secs_f64())
            .unwrap_or(0.0);
        self.last_carbon_sample = Some(now);
        self.carbon_tracker.record(watts, elapsed_secs);
    }

    fn refresh_realtime_data(&mut self) {
        self.system
            .refresh_cpu_specifics(CpuRefreshKind::new().with_cpu_usage().with_frequency());
        self.system.refresh_memory();
        self.system.refresh_processes_specifics(
            ProcessRefreshKind::new()
                .with_cpu()
                .with_memory()
                .with_disk_usage(),
        );

        if self.networks.is_empty() {
            self.networks.refresh_list();
        } else {
            self.networks.refresh();
        }

        if self.components.list().is_empty() {
            self.components.refresh_list();
        } else {
            self.components.refresh();
        }
    }

    fn collect_system_info(&mut self) -> SystemInfo {
        if let Some(system_info) = &self.cached_system_info {
            return system_info.clone();
        }

        let cpu_brand = self
            .system
            .cpus()
            .first()
            .map(|cpu| cpu.brand().to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        let total_cores = self.system.cpus().len();

        // Determine E-core and P-core counts based on chip model
        let (e_core_count, p_core_count, gpu_core_count) =
            detect_core_counts(&cpu_brand, total_cores);

        // Extract chip name from brand string
        let chip_name = extract_chip_name(&cpu_brand);

        let system_info = SystemInfo {
            name: System::name().unwrap_or_else(|| "Unknown".to_string()),
            kernel_version: System::kernel_version().unwrap_or_else(|| "Unknown".to_string()),
            os_version: System::os_version().unwrap_or_else(|| "Unknown".to_string()),
            host_name: System::host_name().unwrap_or_else(|| "Unknown".to_string()),
            cpu_arch: System::cpu_arch().unwrap_or_else(|| "Unknown".to_string()),
            cpu_brand: cpu_brand.clone(),
            cpu_core_count: total_cores,
            e_core_count,
            p_core_count,
            gpu_core_count,
            chip_name,
        };
        self.cached_system_info = Some(system_info.clone());
        system_info
    }

    /// Collect ANE info from CPU power metrics
    fn collect_ane_info(&self, cpu_info: &CpuInfo) -> AneInfo {
        AneInfo {
            usage_percentage: if cpu_info.power_metrics.ane_w > 0.0 {
                // ANE usage estimation based on power (rough approximation)
                (cpu_info.power_metrics.ane_w / 15.0 * 100.0).min(100.0) as f32
            } else {
                0.0
            },
            power_w: cpu_info.power_metrics.ane_w,
        }
    }

    /// Collect DRAM info from CPU power metrics
    fn collect_dram_info(&self, cpu_info: &CpuInfo) -> DramInfo {
        DramInfo {
            read_bytes_per_sec: 0.0,
            write_bytes_per_sec: 0.0,
            total_bytes_per_sec: 0.0,
            power_w: cpu_info.power_metrics.dram_w,
        }
    }

    async fn collect_cpu_info(&mut self) -> Result<CpuInfo, Box<dyn std::error::Error>> {
        cpu::collect_cpu_info(
            &self.system,
            &mut self.last_powermetrics,
            &mut self.cached_cpu_metrics,
            &mut self.cached_powermetrics_output,
            self.powermetrics_cache_duration,
        )
        .await
    }

    fn collect_memory_info(&self) -> MemoryInfo {
        memory::collect_memory_info(&self.system)
    }

    fn collect_network_info(&self) -> Vec<NetworkInterface> {
        network::collect_network_info(&self.networks)
    }

    fn collect_temperature_info(&self) -> Vec<TemperatureInfo> {
        temperature::collect_temperature_info(&self.components)
    }

    fn collect_process_info(&self) -> Vec<ProcessInfo> {
        process::collect_process_info(&self.system)
    }

    async fn collect_thermal_info(&self) -> ThermalInfo {
        thermal::collect_thermal_info().await
    }

    async fn collect_performance_metrics(
        &self,
        cpu_info: &CpuInfo,
        total_power: f64,
    ) -> PerformanceMetrics {
        performance::collect_performance_metrics(cpu_info, total_power).await
    }

    async fn collect_system_health(&self) -> SystemHealthInfo {
        health::collect_system_health().await
    }
}

/// Detect E-core, P-core, and GPU core counts based on chip brand string
fn detect_core_counts(brand: &str, total_cores: usize) -> (usize, usize, usize) {
    let brand_lower = brand.to_lowercase();

    // M1 family
    if brand_lower.contains("m1 ultra") {
        (4, 16, 64)
    } else if brand_lower.contains("m1 max") {
        (2, 8, 32)
    } else if brand_lower.contains("m1 pro") {
        (2, 8, 16)
    }
    // M2 family
    else if brand_lower.contains("m2 ultra") {
        (4, 16, 76)
    } else if brand_lower.contains("m2 max") {
        (4, 8, 38)
    } else if brand_lower.contains("m2 pro") {
        (4, 4, 19)
    }
    // M3 family
    else if brand_lower.contains("m3 max") {
        (4, 12, 40)
    } else if brand_lower.contains("m3 pro") {
        (4, 6, 18)
    } else if brand_lower.contains("m3") {
        (4, 4, 10)
    }
    // M4 family
    else if brand_lower.contains("m4 max") {
        (4, 12, 40)
    } else if brand_lower.contains("m4 pro") {
        (4, 6, 20)
    } else if brand_lower.contains("m4") {
        (4, 6, 10)
    }
    // Default M1/M2
    else if brand_lower.contains("m1") {
        (4, 4, 8)
    } else if brand_lower.contains("m2") {
        (4, 4, 10)
    } else {
        // Fallback: estimate from total cores
        let e = total_cores / 4;
        let p = total_cores - e;
        (e, p, 8)
    }
}

/// Extract chip name from CPU brand string
fn extract_chip_name(brand: &str) -> String {
    let brand_lower = brand.to_lowercase();
    if brand_lower.contains("m1 ultra") {
        "Apple M1 Ultra".to_string()
    } else if brand_lower.contains("m1 max") {
        "Apple M1 Max".to_string()
    } else if brand_lower.contains("m1 pro") {
        "Apple M1 Pro".to_string()
    } else if brand_lower.contains("m2 ultra") {
        "Apple M2 Ultra".to_string()
    } else if brand_lower.contains("m2 max") {
        "Apple M2 Max".to_string()
    } else if brand_lower.contains("m2 pro") {
        "Apple M2 Pro".to_string()
    } else if brand_lower.contains("m3 max") {
        "Apple M3 Max".to_string()
    } else if brand_lower.contains("m3 pro") {
        "Apple M3 Pro".to_string()
    } else if brand_lower.contains("m3") {
        "Apple M3".to_string()
    } else if brand_lower.contains("m4 max") {
        "Apple M4 Max".to_string()
    } else if brand_lower.contains("m4 pro") {
        "Apple M4 Pro".to_string()
    } else if brand_lower.contains("m4") {
        "Apple M4".to_string()
    } else if brand_lower.contains("m2") {
        "Apple M2".to_string()
    } else if brand_lower.contains("m1") {
        "Apple M1".to_string()
    } else {
        brand.to_string()
    }
}

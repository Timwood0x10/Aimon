//! Collectors module for system data
//! Contains specialized collectors for different system metrics

pub mod backends;
pub mod capabilities;
pub mod cpu;
pub mod directory_usage;
pub mod disk_io;
pub mod disk_usage;
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
use crate::collectors::backends::powermetrics::PowermetricsSampler;
#[cfg(target_os = "macos")]
use crate::collectors::backends::IoReportSampler;
#[cfg(target_os = "macos")]
use crate::collectors::backends::MachCpuSampler;
use crate::collectors::capabilities::CollectorCapabilities;
#[cfg(target_os = "macos")]
use crate::collectors::cpu::get_fallback_cpu_metrics;
use crate::config::FanControlConfig;
use crate::types::*;
use std::sync::Arc;
use std::time::{Duration, Instant};
use sysinfo::{Components, CpuRefreshKind, Networks, ProcessRefreshKind, System};
use tokio::sync::RwLock;

/// Main data collector that coordinates all sub-collectors
pub struct DataCollector {
    system: System,
    networks: Networks,
    components: Components,
    powermetrics_sampler: PowermetricsSampler,
    battery_collector: FastBatteryCollector,
    cached_system_info: Option<SystemInfo>,
    carbon_tracker: CarbonTracker,
    last_carbon_sample: Option<Instant>,
    pub capabilities: CollectorCapabilities,
    fan_control: FanControlConfig,
    directory_usage_cache: Arc<RwLock<Vec<DirectoryUsageInfo>>>,
    directory_usage_started: bool,
    #[cfg(target_os = "macos")]
    mach_sampler: MachCpuSampler,
    #[cfg(target_os = "macos")]
    ioreport_sampler: Option<IoReportSampler>,
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
            powermetrics_sampler: PowermetricsSampler::new(Duration::from_secs(2)),
            battery_collector: FastBatteryCollector::new(),
            cached_system_info: None,
            carbon_tracker: CarbonTracker::new(),
            last_carbon_sample: None,
            capabilities: CollectorCapabilities::detect(),
            fan_control: FanControlConfig::default(),
            directory_usage_cache: Arc::new(RwLock::new(Vec::new())),
            directory_usage_started: false,
            #[cfg(target_os = "macos")]
            mach_sampler: MachCpuSampler::new(),
            #[cfg(target_os = "macos")]
            ioreport_sampler: IoReportSampler::new().ok(),
        }
    }

    /// Fast startup version with lazy initialization
    pub fn new_fast() -> Self {
        Self {
            system: System::new(),
            networks: Networks::new(),
            components: Components::new(),
            powermetrics_sampler: PowermetricsSampler::new(Duration::from_secs(1)),
            battery_collector: FastBatteryCollector::new(),
            cached_system_info: None,
            carbon_tracker: CarbonTracker::new(),
            last_carbon_sample: None,
            capabilities: CollectorCapabilities::detect(),
            fan_control: FanControlConfig::default(),
            directory_usage_cache: Arc::new(RwLock::new(Vec::new())),
            directory_usage_started: false,
            #[cfg(target_os = "macos")]
            mach_sampler: MachCpuSampler::new(),
            #[cfg(target_os = "macos")]
            ioreport_sampler: IoReportSampler::new().ok(),
        }
    }

    pub fn with_fan_control(mut self, fan_control: FanControlConfig) -> Self {
        self.fan_control = fan_control;
        self.capabilities.smc_write_available = self.fan_control.enabled
            && self.fan_control.runtime_allowed
            && self.capabilities.smc_read_available;
        self
    }

    fn start_directory_usage_once(&mut self) {
        if self.directory_usage_started {
            return;
        }
        self.directory_usage_started = true;
        let cache = Arc::clone(&self.directory_usage_cache);
        tokio::spawn(async move {
            let entries = directory_usage::collect_directory_usage_once().await;
            let mut cache = cache.write().await;
            *cache = entries;
        });
    }

    /// Collect all system data
    /// Optimized: CPU + parallel collectors run concurrently via tokio::select!
    /// CPU usage is instant (sysinfo), powermetrics uses cache for speed.
    pub async fn collect_all_data(&mut self) -> Result<SystemData, Box<dyn std::error::Error>> {
        self.refresh_realtime_data();
        self.start_directory_usage_once();

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
            disk_usage_info,
            thermal_info,
            performance_metrics,
            system_health,
        ) = tokio::join!(
            gpu::collect_gpu_info_from_powermetrics(self.powermetrics_sampler.get_cached_output()),
            thunderbolt::collect_thunderbolt_info(),
            disk_io::collect_disk_io_info(),
            async { disk_usage::collect_disk_usage_info() },
            self.collect_thermal_info(),
            self.collect_performance_metrics(&cpu_info, total_power),
            self.collect_system_health(),
        );

        gpu::complete_static_gpu_info(&mut gpu_info).await;
        let directory_usage_info = self.directory_usage_cache.read().await.clone();

        // battery (separate borrow)
        let battery_info = self.battery_collector.get_battery_info().await;
        self.record_carbon_sample(total_power, cpu_info.average_usage, &process_info);
        let carbon_info = self.carbon_tracker.clone();

        Ok(SystemData {
            system_info,
            cpu_info,
            gpu_info,
            ane_info,
            dram_info,
            thunderbolt_info,
            disk_io_info,
            disk_usage_info,
            directory_usage_info,
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
            capabilities: self.capabilities.clone(),
            timestamp: Instant::now(),
        })
    }

    fn record_carbon_sample(&mut self, watts: f64, cpu_usage: f32, processes: &[ProcessInfo]) {
        let now = Instant::now();
        let elapsed_secs = self
            .last_carbon_sample
            .map(|last| now.duration_since(last).as_secs_f64())
            .unwrap_or(0.0);
        self.last_carbon_sample = Some(now);
        let process_samples: Vec<(String, f32)> = processes
            .iter()
            .map(|process| (process.name.clone(), process.cpu_usage))
            .collect();
        self.carbon_tracker
            .record_with_processes(watts, elapsed_secs, cpu_usage, &process_samples);
    }

    fn refresh_realtime_data(&mut self) {
        use sysinfo::ProcessesToUpdate;
        self.system
            .refresh_cpu_specifics(CpuRefreshKind::nothing().with_cpu_usage().with_frequency());
        self.system.refresh_memory();
        let kind = ProcessRefreshKind::nothing()
            .with_cpu()
            .with_memory()
            .with_disk_usage();
        self.system
            .refresh_processes_specifics(ProcessesToUpdate::All, false, kind);

        self.networks.refresh(false);
        self.components.refresh(false);
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
            cpu_arch: { let a = System::cpu_arch(); if a.is_empty() { "Unknown".to_string() } else { a } },
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

    /// Collect DRAM info from cached powermetrics output
    fn collect_dram_info(&self, cpu_info: &CpuInfo) -> DramInfo {
        // Try to extract DRAM info from cached powermetrics output
        let raw_dram_w = self
            .powermetrics_sampler
            .get_cached_output()
            .and_then(backends::powermetrics::extract_dram_power);
        let power_w = raw_dram_w.unwrap_or(cpu_info.power_metrics.dram_w);
        DramInfo {
            read_bytes_per_sec: 0.0,
            write_bytes_per_sec: 0.0,
            total_bytes_per_sec: 0.0,
            power_w,
            meta: if raw_dram_w.is_some() {
                MetricMeta::measured(MetricSource::Powermetrics)
            } else {
                MetricMeta::estimated(
                    MetricSource::Estimate,
                    "no DRAM line in powermetrics, using derived value",
                )
            },
        }
    }

    /// Try Mach CPU sampling first (native, no subprocess), fall back to sysinfo.
    async fn collect_cpu_info(&mut self) -> Result<CpuInfo, Box<dyn std::error::Error>> {
        #[cfg(target_os = "macos")]
        if let Some(cpu_usages) = self.try_mach_cpu() {
            let average_usage = if !cpu_usages.is_empty() {
                cpu_usages.iter().sum::<f32>() / cpu_usages.len() as f32
            } else {
                0.0
            };
            // Prefer IOReport for power data (no sudo, native counters),
            // fall back to powermetrics if IOReport isn't ready yet.
            let (power_metrics, power_meta) = if let Some(Some(ioreport_sample)) =
                self.ioreport_sampler.as_mut().map(|s| s.sample_power())
            {
                let m = CPUMetrics {
                    cpu_w: ioreport_sample.cpu_w,
                    gpu_w: ioreport_sample.gpu_w,
                    ane_w: ioreport_sample.ane_w,
                    dram_w: ioreport_sample.dram_w,
                    package_w: ioreport_sample.package_w,
                    ..Default::default()
                };
                let meta = if ioreport_sample.package_w > 0.0 {
                    MetricMeta::measured(MetricSource::IoReport)
                } else {
                    MetricMeta::estimated(MetricSource::Estimate, "IOReport returned zero")
                };
                (m, meta)
            } else {
                match self.powermetrics_sampler.sample().await {
                    Ok(m) => {
                        let meta = if m.package_w > 0.0 || m.cpu_w > 0.0 {
                            MetricMeta::measured(MetricSource::Powermetrics)
                        } else {
                            MetricMeta::estimated(
                                MetricSource::Estimate,
                                "powermetrics unavailable",
                            )
                        };
                        (m, meta)
                    }
                    Err(e) => {
                        log::warn!("Powermetrics failed: {}", e);
                        (
                            get_fallback_cpu_metrics(&self.system),
                            MetricMeta::estimated(
                                MetricSource::Estimate,
                                "IOReport+Powermetrics both unavailable",
                            ),
                        )
                    }
                }
            };
            return Ok(CpuInfo {
                core_usages: cpu_usages,
                average_usage,
                power_metrics,
                usage_meta: MetricMeta::measured(MetricSource::Mach),
                power_meta,
            });
        }

        // Fallback to sysinfo
        cpu::collect_cpu_info(&self.system, &mut self.powermetrics_sampler).await
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
        thermal::collect_thermal_info(&self.fan_control).await
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
    #[cfg(target_os = "macos")]
    fn try_mach_cpu(&mut self) -> Option<Vec<f32>> {
        self.mach_sampler.sample_usage().ok()
    }
}

#[cfg(not(target_os = "macos"))]
impl DataCollector {
    fn try_mach_cpu(&mut self) -> Option<Vec<f32>> {
        None
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

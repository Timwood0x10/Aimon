//! Collectors module for system data
//! Contains specialized collectors for different system metrics

pub mod cpu;
pub mod memory;
pub mod network;
pub mod temperature;
pub mod process;
pub mod thermal;
pub mod performance;
pub mod health;

use crate::types::*;
use crate::battery_collector::FastBatteryCollector;
use sysinfo::{Components, Networks, System};
use std::time::{Duration, Instant};

/// Main data collector that coordinates all sub-collectors
pub struct DataCollector {
    system: System,
    networks: Networks,
    components: Components,
    last_powermetrics: Option<Instant>,
    cached_cpu_metrics: Option<CPUMetrics>,
    powermetrics_cache_duration: Duration,
    battery_collector: FastBatteryCollector,
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
            powermetrics_cache_duration: Duration::from_secs(2),
            battery_collector: FastBatteryCollector::new(),
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
            powermetrics_cache_duration: Duration::from_secs(1),
            battery_collector: FastBatteryCollector::new(),
        }
    }

    /// Collect all system data
    pub async fn collect_all_data(&mut self) -> Result<SystemData, Box<dyn std::error::Error>> {
        // Refresh system data
        self.system.refresh_all();
        self.system.refresh_processes();
        self.networks.refresh();
        self.components.refresh();

        let system_info = self.collect_system_info();
        let cpu_info = self.collect_cpu_info().await?;
        let memory_info = self.collect_memory_info();
        let network_info = self.collect_network_info();
        let temperature_info = self.collect_temperature_info();
        let process_info = self.collect_process_info();
        let total_power = cpu_info.power_metrics.package_w;

        let battery_info = self.battery_collector.get_battery_info().await;
        let thermal_info = self.collect_thermal_info().await;
        let performance_metrics = self.collect_performance_metrics(&cpu_info, total_power).await;
        let system_health = self.collect_system_health().await;

        Ok(SystemData {
            system_info,
            cpu_info,
            memory_info,
            network_info,
            temperature_info,
            process_info,
            battery_info,
            thermal_info,
            performance_metrics,
            system_health,
            timestamp: Instant::now(),
        })
    }

    fn collect_system_info(&self) -> SystemInfo {
        SystemInfo {
            name: System::name().unwrap_or_else(|| "Unknown".to_string()),
            kernel_version: System::kernel_version().unwrap_or_else(|| "Unknown".to_string()),
            os_version: System::os_version().unwrap_or_else(|| "Unknown".to_string()),
            host_name: System::host_name().unwrap_or_else(|| "Unknown".to_string()),
            cpu_arch: System::cpu_arch().unwrap_or_else(|| "Unknown".to_string()),
            cpu_brand: self.system.cpus().first()
                .map(|cpu| cpu.brand().to_string())
                .unwrap_or_else(|| "Unknown".to_string()),
        }
    }

    async fn collect_cpu_info(&mut self) -> Result<CpuInfo, Box<dyn std::error::Error>> {
        cpu::collect_cpu_info(
            &self.system,
            &mut self.last_powermetrics,
            &mut self.cached_cpu_metrics,
            self.powermetrics_cache_duration,
        ).await
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

    async fn collect_performance_metrics(&self, cpu_info: &CpuInfo, total_power: f64) -> PerformanceMetrics {
        performance::collect_performance_metrics(cpu_info, total_power).await
    }

    async fn collect_system_health(&self) -> SystemHealthInfo {
        health::collect_system_health().await
    }
}
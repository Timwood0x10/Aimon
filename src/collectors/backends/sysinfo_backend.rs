//! Sysinfo-based data collection backend
//!
//! Uses the `sysinfo` crate to collect CPU usage, memory, processes,
//! network interfaces, and temperature components. This is the default
//! fallback for all platforms and does not require special privileges.

use sysinfo::{Components, CpuRefreshKind, Networks, ProcessRefreshKind, System};

/// Collector wrapping sysinfo for basic system data
pub struct SysinfoCollector {
    pub system: System,
    pub networks: Networks,
    pub components: Components,
}

impl SysinfoCollector {
    pub fn new() -> Self {
        Self {
            system: System::new_all(),
            networks: Networks::new_with_refreshed_list(),
            components: Components::new_with_refreshed_list(),
        }
    }

    /// Lightweight initialisation, refreshes lazily
    pub fn new_lazy() -> Self {
        Self {
            system: System::new(),
            networks: Networks::new(),
            components: Components::new(),
        }
    }

    /// Refresh CPU-specific data (usage + frequency)
    pub fn refresh_cpu(&mut self) {
        self.system
            .refresh_cpu_specifics(CpuRefreshKind::new().with_cpu_usage().with_frequency());
    }

    /// Refresh memory info
    pub fn refresh_memory(&mut self) {
        self.system.refresh_memory();
    }

    /// Refresh process table with CPU, memory and disk usage
    pub fn refresh_processes(&mut self) {
        self.system.refresh_processes_specifics(
            ProcessRefreshKind::new()
                .with_cpu()
                .with_memory()
                .with_disk_usage(),
        );
    }

    /// Refresh network interfaces
    pub fn refresh_networks(&mut self) {
        if self.networks.is_empty() {
            self.networks.refresh_list();
        } else {
            self.networks.refresh();
        }
    }

    /// Refresh temperature components
    pub fn refresh_temperatures(&mut self) {
        if self.components.list().is_empty() {
            self.components.refresh_list();
        } else {
            self.components.refresh();
        }
    }

    /// Refresh all real-time data in one call
    pub fn refresh_all(&mut self) {
        self.refresh_cpu();
        self.refresh_memory();
        self.refresh_processes();
        self.refresh_networks();
        self.refresh_temperatures();
    }

    pub fn cpu_usage(&self) -> Vec<f32> {
        self.system.cpus().iter().map(|c| c.cpu_usage()).collect()
    }

    pub fn average_cpu_usage(&self) -> f32 {
        let usages = self.cpu_usage();
        if usages.is_empty() {
            0.0
        } else {
            usages.iter().sum::<f32>() / usages.len() as f32
        }
    }

    pub fn cpu_count(&self) -> usize {
        self.system.cpus().len()
    }

    pub fn cpu_brand(&self) -> String {
        let brand = self
            .system
            .cpus()
            .first()
            .map(|c| c.brand().to_string())
            .unwrap_or_default();

        if brand.trim().is_empty() {
            "Unknown".to_string()
        } else {
            brand
        }
    }

    pub fn system_ref(&self) -> &System {
        &self.system
    }

    pub fn system_mut(&mut self) -> &mut System {
        &mut self.system
    }

    pub fn networks_ref(&self) -> &Networks {
        &self.networks
    }

    pub fn components_ref(&self) -> &Components {
        &self.components
    }
}

impl Default for SysinfoCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sysinfo_collector_new() {
        let collector = SysinfoCollector::new();
        assert!(
            !collector.system.cpus().is_empty(),
            "should detect at least 1 CPU"
        );
    }

    #[test]
    fn test_sysinfo_collector_new_lazy() {
        let collector = SysinfoCollector::new_lazy();
        // Lazy version should still work
        assert!(
            collector.cpu_count() == 0,
            "lazy should have 0 CPUs before refresh"
        );
    }

    #[test]
    fn test_sysinfo_collector_cpu_count() {
        let collector = SysinfoCollector::new();
        assert!(
            collector.cpu_count() > 0,
            "SysinfoCollector::new should discover at least one CPU"
        );

        let cpu_brand = collector.cpu_brand();
        assert!(
            !cpu_brand.trim().is_empty() || cpu_brand == "Unknown",
            "cpu_brand should either contain a non-empty brand or the Unknown fallback, got {:?}",
            cpu_brand
        );
    }

    #[test]
    fn test_sysinfo_collector_average_usage() {
        let collector = SysinfoCollector::new();
        let avg = collector.average_cpu_usage();
        assert!(avg >= 0.0, "average CPU usage should be >= 0");
    }

    #[test]
    fn test_sysinfo_collector_refresh_all() {
        let mut collector = SysinfoCollector::new_lazy();
        collector.refresh_all();
        assert!(
            collector.cpu_count() > 0,
            "after refresh_all, CPUs should be detected"
        );
    }

    #[test]
    fn test_sysinfo_collector_default_impl() {
        let collector = SysinfoCollector::default();
        assert!(collector.cpu_count() > 0);
    }
}

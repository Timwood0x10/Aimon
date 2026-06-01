// History data storage for system metrics
// Maintains circular buffers for trend analysis

use crate::types::SystemData;
use std::collections::VecDeque;
use std::time::Instant;

/// Circular buffer for historical system data
#[derive(Clone)]
pub struct HistoryData {
    pub cpu_history: VecDeque<f32>,
    pub memory_history: VecDeque<u16>,
    pub network_rx_history: VecDeque<u64>,
    pub network_tx_history: VecDeque<u64>,
    pub network_rx_rate_history: VecDeque<f64>, // bytes per second
    pub network_tx_rate_history: VecDeque<f64>, // bytes per second
    pub temperature_history: VecDeque<f32>,
    pub battery_history: VecDeque<f32>,
    pub disk_usage_history: VecDeque<f32>,
    pub disk_io_rate_history: VecDeque<f64>,
    max_size: usize,
    last_update: Option<Instant>,
    last_network_rx: Option<u64>,
    last_network_tx: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collectors::capabilities::CollectorCapabilities;
    use crate::types::*;

    fn sample_data(usage_percentage: f32, read_rate: f64, write_rate: f64) -> SystemData {
        SystemData {
            system_info: SystemInfo {
                name: "test".to_string(),
                kernel_version: "test".to_string(),
                os_version: "test".to_string(),
                host_name: "test".to_string(),
                cpu_arch: "arm64".to_string(),
                cpu_brand: "test".to_string(),
                cpu_core_count: 1,
                e_core_count: 0,
                p_core_count: 1,
                gpu_core_count: 0,
                chip_name: "test".to_string(),
            },
            cpu_info: CpuInfo {
                core_usages: vec![0.0],
                average_usage: 0.0,
                power_metrics: CPUMetrics::default(),
                usage_meta: MetricMeta::unavailable(),
                power_meta: MetricMeta::unavailable(),
            },
            gpu_info: GpuInfo::default(),
            ane_info: AneInfo::default(),
            dram_info: DramInfo::default(),
            thunderbolt_info: ThunderboltInfo::default(),
            disk_io_info: DiskIoInfo {
                read_bytes_per_sec: read_rate,
                write_bytes_per_sec: write_rate,
                read_ops_per_sec: 0.0,
                write_ops_per_sec: 0.0,
            },
            disk_usage_info: vec![DiskUsageInfo {
                name: "root".to_string(),
                mount_point: "/".to_string(),
                file_system: "apfs".to_string(),
                total_bytes: 100,
                available_bytes: 100u64.saturating_sub(usage_percentage as u64),
                used_bytes: usage_percentage as u64,
                usage_percentage,
                is_removable: false,
            }],
            directory_usage_info: Vec::new(),
            memory_info: MemoryInfo {
                total_memory: 0,
                used_memory: 0,
                available_memory: 0,
                total_swap: 0,
                used_swap: 0,
                usage_percentage: 0,
            },
            network_info: Vec::new(),
            temperature_info: Vec::new(),
            process_info: Vec::new(),
            battery_info: BatteryInfo::default(),
            thermal_info: ThermalInfo::default(),
            performance_metrics: PerformanceMetrics::default(),
            system_health: SystemHealthInfo::default(),
            terminal_info: TerminalInfo::default(),
            carbon_info: crate::carbon::tracker::CarbonTracker::new(),
            capabilities: CollectorCapabilities::default(),
            timestamp: Instant::now(),
        }
    }

    /// Objective: Verify storage histories are updated from `SystemData`.
    /// Invariants: Disk usage and combined I/O rate preserve latest values.
    #[test]
    fn test_storage_history_updates_from_system_data() {
        let mut history = HistoryData::new(3);
        history.update_from_system_data(&sample_data(42.0, 100.0, 50.0));

        assert_eq!(
            history.disk_usage_history.back().copied(),
            Some(42.0),
            "disk usage history should store the primary disk percentage"
        );
        assert_eq!(
            history.get_disk_io_rate(),
            150.0,
            "disk I/O rate should combine read and write bytes per second"
        );
    }
}

impl HistoryData {
    pub fn new(max_size: usize) -> Self {
        Self {
            cpu_history: VecDeque::with_capacity(max_size),
            memory_history: VecDeque::with_capacity(max_size),
            network_rx_history: VecDeque::with_capacity(max_size),
            network_tx_history: VecDeque::with_capacity(max_size),
            network_rx_rate_history: VecDeque::with_capacity(max_size),
            network_tx_rate_history: VecDeque::with_capacity(max_size),
            temperature_history: VecDeque::with_capacity(max_size),
            battery_history: VecDeque::with_capacity(max_size),
            disk_usage_history: VecDeque::with_capacity(max_size),
            disk_io_rate_history: VecDeque::with_capacity(max_size),
            max_size,
            last_update: None,
            last_network_rx: None,
            last_network_tx: None,
        }
    }

    /// Update all history buffers from current system data
    pub fn update_from_system_data(&mut self, data: &SystemData) {
        let cpu_history = &mut self.cpu_history;
        let memory_history = &mut self.memory_history;
        let network_rx_history = &mut self.network_rx_history;
        let network_tx_history = &mut self.network_tx_history;
        let network_rx_rate_history = &mut self.network_rx_rate_history;
        let network_tx_rate_history = &mut self.network_tx_rate_history;
        let temperature_history = &mut self.temperature_history;
        let battery_history = &mut self.battery_history;
        let disk_usage_history = &mut self.disk_usage_history;
        let disk_io_rate_history = &mut self.disk_io_rate_history;
        let max_size = self.max_size;

        Self::push_capped_static(cpu_history, max_size, data.cpu_info.average_usage);
        Self::push_capped_static(memory_history, max_size, data.memory_info.usage_percentage);
        Self::push_capped_static(battery_history, max_size, data.battery_info.percentage);
        if let Some(primary_disk) = data
            .disk_usage_info
            .iter()
            .find(|disk| disk.mount_point == "/")
            .or_else(|| data.disk_usage_info.first())
        {
            Self::push_capped_static(disk_usage_history, max_size, primary_disk.usage_percentage);
        }
        Self::push_capped_static(
            disk_io_rate_history,
            max_size,
            data.disk_io_info.read_bytes_per_sec + data.disk_io_info.write_bytes_per_sec,
        );

        let total_rx: u64 = data.network_info.iter().map(|n| n.bytes_received).sum();
        let total_tx: u64 = data.network_info.iter().map(|n| n.bytes_transmitted).sum();
        Self::push_capped_static(network_rx_history, max_size, total_rx);
        Self::push_capped_static(network_tx_history, max_size, total_tx);

        // Calculate network rate
        let now = Instant::now();
        if let (Some(last_time), Some(last_rx), Some(last_tx)) =
            (self.last_update, self.last_network_rx, self.last_network_tx)
        {
            let elapsed = now.duration_since(last_time).as_secs_f64();
            if elapsed > 0.0 {
                let rx_rate = (total_rx as f64 - last_rx as f64) / elapsed;
                let tx_rate = (total_tx as f64 - last_tx as f64) / elapsed;
                Self::push_capped_static(network_rx_rate_history, max_size, rx_rate.max(0.0));
                Self::push_capped_static(network_tx_rate_history, max_size, tx_rate.max(0.0));
            }
        }
        self.last_update = Some(now);
        self.last_network_rx = Some(total_rx);
        self.last_network_tx = Some(total_tx);

        if !data.temperature_info.is_empty() {
            let avg_temp = data
                .temperature_info
                .iter()
                .map(|t| t.temperature)
                .sum::<f32>()
                / data.temperature_info.len() as f32;
            Self::push_capped_static(temperature_history, max_size, avg_temp);
        }
    }

    /// Static version of push_capped
    fn push_capped_static<T>(deque: &mut VecDeque<T>, max_size: usize, value: T) {
        if deque.len() >= max_size {
            deque.pop_front();
        }
        deque.push_back(value);
    }

    /// Calculate CPU usage trend (positive = increasing)
    pub fn get_cpu_trend(&self) -> Option<f32> {
        Self::calculate_trend(&self.cpu_history)
    }

    /// Calculate memory usage trend
    pub fn get_memory_trend(&self) -> Option<f32> {
        if self.memory_history.len() < 2 {
            return None;
        }
        let count = self.memory_history.len().min(5) as f32;
        let recent: f32 = self
            .memory_history
            .iter()
            .rev()
            .take(5)
            .map(|&x| x as f32)
            .sum::<f32>()
            / count;
        let older: f32 = self
            .memory_history
            .iter()
            .take(5)
            .map(|&x| x as f32)
            .sum::<f32>()
            / count;
        Some(recent - older)
    }

    /// Get current network receive rate (bytes per second)
    pub fn get_network_rx_rate(&self) -> f64 {
        self.network_rx_rate_history.back().copied().unwrap_or(0.0)
    }

    /// Get current network transmit rate (bytes per second)
    pub fn get_network_tx_rate(&self) -> f64 {
        self.network_tx_rate_history.back().copied().unwrap_or(0.0)
    }

    /// Get current total disk I/O rate (bytes per second).
    pub fn get_disk_io_rate(&self) -> f64 {
        self.disk_io_rate_history.back().copied().unwrap_or(0.0)
    }

    /// Calculate primary disk usage trend.
    pub fn get_disk_usage_trend(&self) -> Option<f32> {
        Self::calculate_trend(&self.disk_usage_history)
    }

    /// Generic trend calculation for f32 values
    fn calculate_trend(history: &VecDeque<f32>) -> Option<f32> {
        if history.len() < 2 {
            return None;
        }
        let count = history.len().min(5) as f32;
        let recent: f32 = history.iter().rev().take(5).sum::<f32>() / count;
        let older: f32 = history.iter().take(5).sum::<f32>() / count;
        Some(recent - older)
    }
}

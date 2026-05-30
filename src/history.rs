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
    max_size: usize,
    last_update: Option<Instant>,
    last_network_rx: Option<u64>,
    last_network_tx: Option<u64>,
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
        let max_size = self.max_size;

        Self::push_capped_static(cpu_history, max_size, data.cpu_info.average_usage);
        Self::push_capped_static(memory_history, max_size, data.memory_info.usage_percentage);

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
            let avg_temp = data.temperature_info.iter().map(|t| t.temperature).sum::<f32>()
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
        let recent: f32 = self.memory_history.iter().rev().take(5).map(|&x| x as f32).sum::<f32>() / count;
        let older: f32 = self.memory_history.iter().take(5).map(|&x| x as f32).sum::<f32>() / count;
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

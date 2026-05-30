use std::collections::VecDeque;
use std::time::SystemTime;
use serde::{Deserialize, Serialize};

/// A snapshot of system state at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemSnapshot {
    pub cpu_avg: f32,
    pub memory_pct: f32,
    pub battery_pct: f32,
    pub network_rx_rate: f64,
    pub network_tx_rate: f64,
    pub temperature_avg: f32,
    pub thermal_pressure: u8,
    pub timestamp: String,
    pub event_label: Option<String>,
}

/// Time travel replay engine
pub struct TimeTravel {
    pub snapshots: VecDeque<SystemSnapshot>,
    pub position: usize,
    pub active: bool,
    max_snapshots: usize,
    // Previous values for event detection
    prev_cpu: Option<f32>,
    prev_memory: Option<f32>,
}

impl TimeTravel {
    /// Create a new TimeTravel instance
    pub fn new(max_snapshots: usize) -> Self {
        Self {
            snapshots: VecDeque::with_capacity(max_snapshots),
            position: 0,
            active: false,
            max_snapshots,
            prev_cpu: None,
            prev_memory: None,
        }
    }

    /// Toggle time travel mode on/off
    pub fn toggle(&mut self) {
        self.active = !self.active;
        if self.active && !self.snapshots.is_empty() {
            self.position = self.snapshots.len().saturating_sub(1);
        }
    }

    /// Record a new snapshot from current system data
    pub fn record_snapshot(
        &mut self,
        cpu_avg: f32,
        memory_pct: f32,
        battery_pct: f32,
        network_rx_rate: f64,
        network_tx_rate: f64,
        temperature_avg: f32,
        thermal_pressure: u8,
    ) {
        // Auto-detect events
        let event_label = self.detect_event(cpu_avg, memory_pct);

        let snapshot = SystemSnapshot {
            cpu_avg,
            memory_pct,
            battery_pct,
            network_rx_rate,
            network_tx_rate,
            temperature_avg,
            thermal_pressure,
            timestamp: format_timestamp(),
            event_label,
        };

        if self.snapshots.len() >= self.max_snapshots {
            self.snapshots.pop_front();
            // Adjust position if needed
            if self.position > 0 {
                self.position -= 1;
            }
        }

        self.snapshots.push_back(snapshot);

        // Update position to latest if not actively browsing
        if !self.active {
            self.position = self.snapshots.len().saturating_sub(1);
        }

        // Update previous values for event detection
        self.prev_cpu = Some(cpu_avg);
        self.prev_memory = Some(memory_pct);
    }

    /// Detect events based on metric changes
    fn detect_event(&self, cpu_avg: f32, memory_pct: f32) -> Option<String> {
        let mut events = Vec::new();

        // CPU spike: > 30% increase from previous
        if let Some(prev) = self.prev_cpu {
            if cpu_avg - prev > 30.0 {
                events.push(format!("CPU spike: {:.0}% -> {:.0}%", prev, cpu_avg));
            }
        }

        // Memory jump: > 10% increase from previous
        if let Some(prev) = self.prev_memory {
            if memory_pct - prev > 10.0 {
                events.push(format!("Memory jump: {:.0}% -> {:.0}%", prev, memory_pct));
            }
        }

        if events.is_empty() {
            None
        } else {
            Some(events.join("; "))
        }
    }

    /// Navigate backward one snapshot
    pub fn move_backward(&mut self) {
        if self.position > 0 {
            self.position -= 1;
        }
    }

    /// Navigate forward one snapshot
    pub fn move_forward(&mut self) {
        if self.position < self.snapshots.len().saturating_sub(1) {
            self.position += 1;
        }
    }

    /// Jump to the earliest snapshot
    pub fn go_to_start(&mut self) {
        if !self.snapshots.is_empty() {
            self.position = 0;
        }
    }

    /// Jump to the latest snapshot
    pub fn go_to_end(&mut self) {
        self.position = self.snapshots.len().saturating_sub(1);
    }

    /// Get the current snapshot at the current position
    pub fn current_snapshot(&self) -> Option<&SystemSnapshot> {
        self.snapshots.get(self.position)
    }

    /// Get the total number of snapshots
    pub fn snapshot_count(&self) -> usize {
        self.snapshots.len()
    }

    /// Get snapshots with events for timeline display
    pub fn event_snapshots(&self) -> Vec<(usize, &SystemSnapshot)> {
        self.snapshots
            .iter()
            .enumerate()
            .filter(|(_, s)| s.event_label.is_some())
            .collect()
    }

    /// Clear all snapshots
    pub fn clear(&mut self) {
        self.snapshots.clear();
        self.position = 0;
        self.prev_cpu = None;
        self.prev_memory = None;
    }
}

impl Default for TimeTravel {
    fn default() -> Self {
        Self::new(300) // Default 300 snapshots (5 minutes at 1s intervals)
    }
}

/// Format current time as a simple HH:MM:SS string
fn format_timestamp() -> String {
    let Ok(duration) = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) else {
        return "??:??:??".to_string();
    };
    let secs = duration.as_secs() % 86400;
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    format!("{:02}:{:02}:{:02}", h, m, s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_snapshot() {
        let mut tt = TimeTravel::new(100);
        assert_eq!(tt.snapshot_count(), 0);

        tt.record_snapshot(50.0, 60.0, 80.0, 1000.0, 500.0, 45.0, 20);
        assert_eq!(tt.snapshot_count(), 1);

        let snap = tt.current_snapshot().unwrap();
        assert!((snap.cpu_avg - 50.0).abs() < f32::EPSILON);
        assert!((snap.memory_pct - 60.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_navigation() {
        let mut tt = TimeTravel::new(100);

        for i in 0..5 {
            tt.record_snapshot(i as f32 * 10.0, 50.0, 80.0, 0.0, 0.0, 40.0, 10);
        }

        assert_eq!(tt.snapshot_count(), 5);
        assert_eq!(tt.position, 4); // at latest

        tt.move_backward();
        assert_eq!(tt.position, 3);
        tt.move_backward();
        assert_eq!(tt.position, 2);
        tt.move_forward();
        assert_eq!(tt.position, 3);

        tt.go_to_start();
        assert_eq!(tt.position, 0);
        tt.go_to_end();
        assert_eq!(tt.position, 4);
    }

    #[test]
    fn test_bounds() {
        let mut tt = TimeTravel::new(100);

        // Navigate with no snapshots - should not panic
        tt.move_backward();
        assert_eq!(tt.position, 0);
        tt.move_forward();
        assert_eq!(tt.position, 0);
        tt.go_to_start();
        assert_eq!(tt.position, 0);
        tt.go_to_end();
        assert_eq!(tt.position, 0);

        // Record one snapshot and test boundaries
        tt.record_snapshot(50.0, 50.0, 50.0, 0.0, 0.0, 40.0, 10);
        tt.go_to_start();
        assert_eq!(tt.position, 0);
        tt.move_backward(); // Should stay at 0
        assert_eq!(tt.position, 0);
    }

    #[test]
    fn test_event_detection() {
        let mut tt = TimeTravel::new(100);

        // Normal CPU
        tt.record_snapshot(30.0, 50.0, 80.0, 0.0, 0.0, 40.0, 10);
        assert!(tt.snapshots[0].event_label.is_none());

        // CPU spike (> 30% increase)
        tt.record_snapshot(70.0, 50.0, 80.0, 0.0, 0.0, 40.0, 10);
        assert!(tt.snapshots[1].event_label.is_some());
        assert!(tt.snapshots[1].event_label.as_ref().unwrap().contains("CPU spike"));
    }

    #[test]
    fn test_max_snapshots_limit() {
        let mut tt = TimeTravel::new(3);

        for i in 0..5 {
            tt.record_snapshot(i as f32, 50.0, 80.0, 0.0, 0.0, 40.0, 10);
        }

        assert_eq!(tt.snapshot_count(), 3);
        // Oldest snapshots should have been removed
        assert!((tt.snapshots[0].cpu_avg - 2.0).abs() < f32::EPSILON);
    }
}

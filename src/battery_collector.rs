// Battery data collector with caching for macOS
// Optimized for fast startup and minimal system calls

use crate::types::BatteryInfo;
use regex::Regex;
use std::time::{Duration, Instant};
use tokio::time::timeout;

// Centralized regex patterns for battery parsing
lazy_static::lazy_static! {
    // pmset patterns
    static ref PMSET_PERCENTAGE: Regex = Regex::new(r"(\d+)%").unwrap();
    static ref PMSET_TIME: Regex = Regex::new(r"(\d+):(\d+) remaining").unwrap();

    // ioreg patterns
    static ref IOREG_CURRENT_CAPACITY: Regex = Regex::new(r#""AppleRawCurrentCapacity"\s*=\s*(\d+)"#).unwrap();
    static ref IOREG_DESIGN_CAPACITY: Regex = Regex::new(r#""DesignCapacity"\s*=\s*(\d+)"#).unwrap();
    static ref IOREG_CYCLE_COUNT: Regex = Regex::new(r#""CycleCount"\s*=\s*(\d+)"#).unwrap();
    static ref IOREG_ALT_CURRENT: Regex = Regex::new(r#""CurrentCapacity"\s*=\s*(\d+)"#).unwrap();
    static ref IOREG_ALT_DESIGN: Regex = Regex::new(r#""MaxCapacity"\s*=\s*(\d+)"#).unwrap();

    // system_profiler patterns
    static ref PROFILER_MAX_CAPACITY: Regex = Regex::new(r"Maximum Capacity:\s*(\d+)%").unwrap();
    static ref PROFILER_CYCLE_COUNT: Regex = Regex::new(r"Cycle Count:\s*(\d+)").unwrap();
    static ref PROFILER_CONDITION: Regex = Regex::new(r"Condition:\s*(\w+(?:\s+\w+)*)").unwrap();
    static ref PROFILER_WATTAGE: Regex = Regex::new(r"Wattage \(W\):\s*(\d+)").unwrap();
}

/// Cached battery data structure
#[derive(Debug, Clone)]
struct BatteryCache {
    data: BatteryInfo,
    timestamp: Instant,
}

/// Fast battery collector with intelligent caching
pub struct FastBatteryCollector {
    cache: Option<BatteryCache>,
    cache_duration: Duration,
}

impl Default for FastBatteryCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl FastBatteryCollector {
    pub fn new() -> Self {
        Self {
            cache: None,
            cache_duration: Duration::from_secs(5),
        }
    }

    /// Get battery info with caching
    pub async fn get_battery_info(&mut self) -> BatteryInfo {
        // Return cached data if still valid
        if let Some(ref cached) = self.cache {
            if cached.timestamp.elapsed() < self.cache_duration {
                return cached.data.clone();
            }
        }

        // Collect fresh data
        match self.collect_battery_data().await {
            Ok(info) => {
                self.cache = Some(BatteryCache {
                    data: info.clone(),
                    timestamp: Instant::now(),
                });
                info
            }
            Err(_) => self
                .cache
                .as_ref()
                .map(|c| c.data.clone())
                .unwrap_or_default(),
        }
    }

    /// Collect battery data from multiple sources
    async fn collect_battery_data(&self) -> Result<BatteryInfo, Box<dyn std::error::Error>> {
        let mut info = BatteryInfo::default();

        // Fast path: pmset for basic status
        if let Ok(basic) = self.get_pmset_data().await {
            info.percentage = basic.percentage;
            info.is_charging = basic.is_charging;
            info.is_plugged = basic.is_plugged;
            info.time_remaining = basic.time_remaining;
        }

        // Detailed path: system_profiler for health
        if let Ok(detailed) = self.get_profiler_data().await {
            info.health_percentage = detailed.health_percentage;
            info.cycle_count = detailed.cycle_count;
            info.power_adapter_wattage = detailed.power_adapter_wattage;
        }

        // Supplementary: ioreg for capacity
        if let Ok(capacity) = self.get_ioreg_data().await {
            info.current_capacity = capacity.current_capacity;
            info.design_capacity = capacity.design_capacity;

            if info.cycle_count == 0 {
                info.cycle_count = capacity.cycle_count;
            }

            if info.health_percentage == 0.0 && capacity.design_capacity > 0 {
                info.health_percentage =
                    (capacity.current_capacity as f32 / capacity.design_capacity as f32) * 100.0;
            }
        }

        Ok(info)
    }

    /// Get basic battery status from pmset
    async fn get_pmset_data(&self) -> Result<BatteryInfo, Box<dyn std::error::Error>> {
        let output = timeout(
            Duration::from_secs(1),
            tokio::process::Command::new("pmset")
                .arg("-g")
                .arg("batt")
                .output(),
        )
        .await??;

        if !output.status.success() {
            return Err("pmset failed".into());
        }

        let text = String::from_utf8_lossy(&output.stdout);
        let mut info = BatteryInfo::default();

        for line in text.lines() {
            if line.contains("InternalBattery") {
                if let Some(caps) = PMSET_PERCENTAGE.captures(line) {
                    info.percentage = caps[1].parse().unwrap_or(0.0);
                }

                info.is_charging = line.contains("charging");
                info.is_plugged = !line.contains("Battery Power");

                if let Some(caps) = PMSET_TIME.captures(line) {
                    let hours: u32 = caps[1].parse().unwrap_or(0);
                    let mins: u32 = caps[2].parse().unwrap_or(0);
                    info.time_remaining = Some(hours * 3600 + mins * 60);
                }
            }

            if line.contains("Now drawing from") {
                info.is_plugged = line.contains("AC Power");
            }
        }

        Ok(info)
    }

    /// Get detailed battery info from system_profiler
    async fn get_profiler_data(&self) -> Result<BatteryDetailedInfo, Box<dyn std::error::Error>> {
        let output = timeout(
            Duration::from_secs(3),
            tokio::process::Command::new("system_profiler")
                .arg("SPPowerDataType")
                .output(),
        )
        .await??;

        if !output.status.success() {
            return Err("system_profiler failed".into());
        }

        let text = String::from_utf8_lossy(&output.stdout);
        let mut info = BatteryDetailedInfo::default();

        for line in text.lines() {
            let line = line.trim();

            if let Some(caps) = PROFILER_MAX_CAPACITY.captures(line) {
                info.health_percentage = caps[1].parse().unwrap_or(0.0);
            } else if let Some(caps) = PROFILER_CYCLE_COUNT.captures(line) {
                info.cycle_count = caps[1].parse().unwrap_or(0);
            } else if let Some(caps) = PROFILER_CONDITION.captures(line) {
                if info.health_percentage == 0.0 {
                    info.health_percentage = match &caps[1] {
                        "Normal" => 95.0,
                        "Replace Soon" => 75.0,
                        "Replace Now" => 50.0,
                        "Service Battery" => 30.0,
                        _ => 85.0,
                    };
                }
            } else if let Some(caps) = PROFILER_WATTAGE.captures(line) {
                info.power_adapter_wattage = caps[1].parse().unwrap_or(0.0);
            }
        }

        Ok(info)
    }

    /// Get capacity info from ioreg
    async fn get_ioreg_data(&self) -> Result<BatteryCapacityInfo, Box<dyn std::error::Error>> {
        let output = timeout(
            Duration::from_secs(2),
            tokio::process::Command::new("sh")
                .arg("-c")
                .arg("ioreg -l | grep -i 'Capacity\\|CycleCount'")
                .output(),
        )
        .await??;

        if !output.status.success() {
            return Err("ioreg failed".into());
        }

        let text = String::from_utf8_lossy(&output.stdout);
        let mut info = BatteryCapacityInfo::default();

        for line in text.lines() {
            let line = line.trim();

            if let Some(caps) = IOREG_CURRENT_CAPACITY.captures(line) {
                info.current_capacity = caps[1].parse().unwrap_or(0);
            } else if let Some(caps) = IOREG_ALT_CURRENT.captures(line) {
                info.current_capacity = caps[1].parse().unwrap_or(0);
            }

            if let Some(caps) = IOREG_DESIGN_CAPACITY.captures(line) {
                info.design_capacity = caps[1].parse().unwrap_or(0);
            } else if let Some(caps) = IOREG_ALT_DESIGN.captures(line) {
                info.design_capacity = caps[1].parse().unwrap_or(0);
            }

            if let Some(caps) = IOREG_CYCLE_COUNT.captures(line) {
                info.cycle_count = caps[1].parse().unwrap_or(0);
            }
        }

        Ok(info)
    }
}

/// Internal struct for detailed battery info
#[derive(Debug, Default)]
struct BatteryDetailedInfo {
    health_percentage: f32,
    cycle_count: u32,
    power_adapter_wattage: f32,
}

/// Internal struct for capacity info
#[derive(Debug, Default)]
struct BatteryCapacityInfo {
    current_capacity: u32,
    design_capacity: u32,
    cycle_count: u32,
}

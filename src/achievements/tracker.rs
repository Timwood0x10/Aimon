use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;
use serde::{Deserialize, Serialize};
use super::definitions::{AchievementId, all_achievements};

/// Stored achievement unlock data
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AchievementRecord {
    id: AchievementId,
    unlocked_at: String, // ISO 8601 timestamp
}

/// Serializable state for save/load
#[derive(Debug, Serialize, Deserialize)]
struct AchievementState {
    records: Vec<AchievementRecord>,
    #[serde(default)]
    progress_counts: std::collections::HashMap<String, u32>,
}

/// Tracks which achievements have been unlocked
pub struct AchievementTracker {
    unlocked: HashSet<AchievementId>,
    progress_counts: std::collections::HashMap<String, u32>,
    config_path: PathBuf,
}

impl AchievementTracker {
    /// Create a new tracker, loading saved state if available
    pub fn new() -> Self {
        let config_path = Self::config_file_path();
        let mut tracker = Self {
            unlocked: HashSet::new(),
            progress_counts: std::collections::HashMap::new(),
            config_path,
        };
        tracker.load();
        tracker
    }

    /// Get the path to the achievements config file
    fn config_file_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".config/system-alert/achievements.json")
    }

    /// Check current system state against all achievement conditions
    pub fn check(
        &mut self,
        cpu_avg: f32,
        max_core_usage: f32,
        memory_pct: u16,
        battery_pct: f32,
        is_charging: bool,
        temp_avg: f32,
        network_rx_rate: f64,
        network_tx_rate: f64,
        thermal_throttling: bool,
        p_cluster_freq: i32,
        uptime_seconds: u64,
    ) {
        // Ice Cold: CPU temp below 40C
        if temp_avg > 0.0 && temp_avg < 40.0 {
            self.try_unlock(AchievementId::IceCold);
        }

        // Power Saver: battery > 80% and not charging
        if battery_pct > 80.0 && !is_charging {
            self.try_unlock(AchievementId::PowerSaver);
        }

        // Marathon: uptime > 24 hours
        if uptime_seconds > 86400 {
            self.try_unlock(AchievementId::Marathon);
        }

        // Fire Hazard: CPU temp > 90C
        if temp_avg > 90.0 {
            self.try_unlock(AchievementId::FireHazard);
        }

        // Memory Hog: memory usage > 90%
        if memory_pct > 90 {
            self.try_unlock(AchievementId::MemoryHog);
        }

        // Network Flood: throughput > 100 MB/s
        let total_rate = network_rx_rate + network_tx_rate;
        if total_rate > 100.0 * 1024.0 * 1024.0 {
            self.try_unlock(AchievementId::NetworkFlood);
        }

        // Thermal Throttle
        if thermal_throttling {
            self.try_unlock(AchievementId::ThermalThrottle);
        }

        // Full Charge: battery at 100%
        if battery_pct >= 100.0 {
            self.try_unlock(AchievementId::FullCharge);
        }

        // Centurion: any core at 100%
        if max_core_usage >= 100.0 {
            self.try_unlock(AchievementId::Centurion);
        }

        // Balanced: CPU and memory both 40-60%
        if (40.0..=60.0).contains(&cpu_avg) && (40..=60).contains(&memory_pct) {
            self.try_unlock(AchievementId::Balanced);
        }

        // Overclocker: P-Cluster freq > 3500 MHz
        if p_cluster_freq > 3500 {
            self.try_unlock(AchievementId::Overclocker);
        }

        // Night Owl: active between midnight and 5 AM
        if let Ok(now) = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            let secs_of_day = now.as_secs() % 86400;
            let hour = (secs_of_day / 3600) as u32;
            if hour < 5 {
                self.try_unlock(AchievementId::NightOwl);
            }
        }
    }

    /// Attempt to unlock an achievement; returns true if newly unlocked
    fn try_unlock(&mut self, id: AchievementId) -> bool {
        if self.unlocked.insert(id) {
            // Increment progress count
            let key = format!("{:?}", id);
            *self.progress_counts.entry(key).or_insert(0) += 1;
            let _ = self.save();
            true
        } else {
            false
        }
    }

    /// Get list of unlocked achievement IDs
    pub fn get_unlocked(&self) -> Vec<AchievementId> {
        self.unlocked.iter().copied().collect()
    }

    /// Check if a specific achievement is unlocked
    pub fn is_unlocked(&self, id: AchievementId) -> bool {
        self.unlocked.contains(&id)
    }

    /// Get progress count for an achievement (how many times it was achieved)
    pub fn get_progress_count(&self, id: AchievementId) -> u32 {
        let key = format!("{:?}", id);
        self.progress_counts.get(&key).copied().unwrap_or(0)
    }

    /// Get total number of unlocked achievements
    pub fn unlocked_count(&self) -> usize {
        self.unlocked.len()
    }

    /// Get total number of possible achievements
    pub fn total_count(&self) -> usize {
        all_achievements().len()
    }

    /// Save achievements to disk
    pub fn save(&self) -> Result<(), std::io::Error> {
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let state = AchievementState {
            records: self
                .unlocked
                .iter()
                .map(|id| AchievementRecord {
                    id: *id,
                    unlocked_at: format_timestamp(),
                })
                .collect(),
            progress_counts: self.progress_counts.clone(),
        };

        let json = serde_json::to_string_pretty(&state)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        fs::write(&self.config_path, json)
    }

    /// Load achievements from disk
    fn load(&mut self) {
        if !self.config_path.exists() {
            return;
        }

        if let Ok(contents) = fs::read_to_string(&self.config_path) {
            if let Ok(state) = serde_json::from_str::<AchievementState>(&contents) {
                for record in state.records {
                    self.unlocked.insert(record.id);
                }
                self.progress_counts = state.progress_counts;
            }
        }
    }

    /// Reset all achievements (for testing)
    pub fn reset(&mut self) {
        self.unlocked.clear();
        self.progress_counts.clear();
        let _ = self.save();
    }
}

impl Default for AchievementTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple ISO 8601 timestamp without external crate
fn format_timestamp() -> String {
    let Ok(duration) = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) else {
        return "unknown".to_string();
    };
    let secs = duration.as_secs();
    let days = secs / 86400;
    let time_of_day = secs % 86400;
    let h = time_of_day / 3600;
    let m = (time_of_day % 3600) / 60;
    let s = time_of_day % 60;
    let year = 1970 + days / 365;
    let day_of_year = days % 365;
    let month = (day_of_year / 30) + 1;
    let day = (day_of_year % 30) + 1;
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", year, month, day, h, m, s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use tempfile::TempDir;

    fn make_tracker_with_tempdir() -> (AchievementTracker, TempDir) {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("achievements.json");
        let mut tracker = AchievementTracker {
            unlocked: HashSet::new(),
            progress_counts: std::collections::HashMap::new(),
            config_path: path,
        };
        tracker.load();
        (tracker, tmp)
    }

    #[test]
    fn test_initial_state() {
        let (tracker, _tmp) = make_tracker_with_tempdir();
        assert_eq!(tracker.unlocked_count(), 0);
        assert_eq!(tracker.total_count(), 12);
        assert!(tracker.get_unlocked().is_empty());
    }

    #[test]
    fn test_ice_cold() {
        let (mut tracker, _tmp) = make_tracker_with_tempdir();
        assert!(!tracker.is_unlocked(AchievementId::IceCold));

        // Temp below 40 triggers Ice Cold
        tracker.check(
            20.0, 25.0, 50, 80.0, false,
            35.0, 0.0, 0.0, false, 2000, 3600,
        );
        assert!(tracker.is_unlocked(AchievementId::IceCold));
    }

    #[test]
    fn test_fire_hazard() {
        let (mut tracker, _tmp) = make_tracker_with_tempdir();
        assert!(!tracker.is_unlocked(AchievementId::FireHazard));

        // Temp above 90 triggers Fire Hazard
        tracker.check(
            95.0, 99.0, 50, 50.0, false,
            92.0, 0.0, 0.0, false, 2000, 3600,
        );
        assert!(tracker.is_unlocked(AchievementId::FireHazard));
    }

    #[test]
    fn test_save_load() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("achievements.json");

        {
            let mut tracker = AchievementTracker {
                unlocked: HashSet::new(),
                progress_counts: std::collections::HashMap::new(),
                config_path: path.clone(),
            };
            tracker.try_unlock(AchievementId::IceCold);
            tracker.try_unlock(AchievementId::Marathon);
            tracker.save().unwrap();
        }

        {
            let mut tracker = AchievementTracker {
                unlocked: HashSet::new(),
                progress_counts: std::collections::HashMap::new(),
                config_path: path,
            };
            tracker.load();
            assert!(tracker.is_unlocked(AchievementId::IceCold));
            assert!(tracker.is_unlocked(AchievementId::Marathon));
            assert!(!tracker.is_unlocked(AchievementId::FireHazard));
        }
    }

    #[test]
    fn test_progress_count() {
        let (mut tracker, _tmp) = make_tracker_with_tempdir();

        // First unlock
        assert!(tracker.try_unlock(AchievementId::IceCold));
        assert_eq!(tracker.get_progress_count(AchievementId::IceCold), 1);

        // Already unlocked, should return false and not increment
        assert!(!tracker.try_unlock(AchievementId::IceCold));
        assert_eq!(tracker.get_progress_count(AchievementId::IceCold), 1);
    }
}

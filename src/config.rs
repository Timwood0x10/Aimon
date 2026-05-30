// Configuration management with validation

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Process sorting options
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProcessSortBy {
    #[serde(rename = "cpu")]
    Cpu,
    #[serde(rename = "memory")]
    Memory,
    #[serde(rename = "pid")]
    Pid,
    #[serde(rename = "name")]
    Name,
}

impl Default for ProcessSortBy {
    fn default() -> Self {
        Self::Cpu
    }
}

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub refresh_rate: u64,
    pub minimal_mode: bool,
    pub thresholds: ThresholdConfig,
    pub display: DisplayConfig,
    pub notifications: NotificationConfig,
    #[serde(default)]
    pub process_sort_by: ProcessSortBy,
}

/// Alert threshold configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdConfig {
    pub cpu_warning: f32,
    pub cpu_critical: f32,
    pub memory_warning: u16,
    pub memory_critical: u16,
    pub temperature_warning: f32,
    pub temperature_critical: f32,
}

/// Display settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    pub show_temperatures: bool,
    pub show_network: bool,
    pub show_processes: bool,
    pub show_history: bool,
    pub history_size: usize,
    pub theme: String,
}

/// Notification settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    pub enabled: bool,
    pub cooldown_seconds: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            refresh_rate: 1,
            minimal_mode: false,
            thresholds: ThresholdConfig {
                cpu_warning: 75.0,
                cpu_critical: 90.0,
                memory_warning: 75,
                memory_critical: 90,
                temperature_warning: 70.0,
                temperature_critical: 85.0,
            },
            display: DisplayConfig {
                show_temperatures: true,
                show_network: true,
                show_processes: true,
                show_history: true,
                history_size: 60,
                theme: "default".to_string(),
            },
            notifications: NotificationConfig {
                enabled: true,
                cooldown_seconds: 30,
            },
            process_sort_by: ProcessSortBy::default(),
        }
    }
}

impl Config {
    /// Load configuration from TOML file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    /// Save configuration to TOML file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Merge CLI arguments into config
    pub fn merge_with_cli(&mut self, refresh_rate: Option<u64>, minimal_mode: bool, theme: Option<&str>) {
        if let Some(rate) = refresh_rate {
            self.refresh_rate = rate;
        }
        if minimal_mode {
            self.minimal_mode = true;
        }
        if let Some(theme_name) = theme {
            self.display.theme = theme_name.to_string();
        }
    }

    /// Validate configuration values
    fn validate(&self) -> Result<(), String> {
        if self.refresh_rate == 0 {
            return Err("refresh_rate must be greater than 0".to_string());
        }

        if self.refresh_rate > 60 {
            return Err("refresh_rate should not exceed 60 seconds".to_string());
        }

        if self.thresholds.cpu_warning >= self.thresholds.cpu_critical {
            return Err("cpu_warning must be less than cpu_critical".to_string());
        }

        if self.thresholds.memory_warning >= self.thresholds.memory_critical {
            return Err("memory_warning must be less than memory_critical".to_string());
        }

        if self.thresholds.temperature_warning >= self.thresholds.temperature_critical {
            return Err("temperature_warning must be less than temperature_critical".to_string());
        }

        if self.display.history_size == 0 || self.display.history_size > 1000 {
            return Err("history_size must be between 1 and 1000".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_valid_config() -> Config {
        Config::default()
    }

    #[test]
    fn test_default_config_is_valid() {
        let config = create_valid_config();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_refresh_rate_zero_is_invalid() {
        let mut config = create_valid_config();
        config.refresh_rate = 0;
        assert!(config.validate().is_err());
        assert_eq!(
            config.validate().unwrap_err(),
            "refresh_rate must be greater than 0"
        );
    }

    #[test]
    fn test_refresh_rate_above_60_is_invalid() {
        let mut config = create_valid_config();
        config.refresh_rate = 61;
        assert!(config.validate().is_err());
        assert_eq!(
            config.validate().unwrap_err(),
            "refresh_rate should not exceed 60 seconds"
        );
    }

    #[test]
    fn test_refresh_rate_boundary_values() {
        let mut config = create_valid_config();

        // Test minimum valid value
        config.refresh_rate = 1;
        assert!(config.validate().is_ok());

        // Test maximum valid value
        config.refresh_rate = 60;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_cpu_warning_must_be_less_than_critical() {
        let mut config = create_valid_config();

        // Equal values should be invalid
        config.thresholds.cpu_warning = 90.0;
        config.thresholds.cpu_critical = 90.0;
        assert!(config.validate().is_err());

        // Warning greater than critical should be invalid
        config.thresholds.cpu_warning = 95.0;
        config.thresholds.cpu_critical = 90.0;
        assert!(config.validate().is_err());

        // Valid case
        config.thresholds.cpu_warning = 75.0;
        config.thresholds.cpu_critical = 90.0;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_memory_warning_must_be_less_than_critical() {
        let mut config = create_valid_config();

        // Equal values should be invalid
        config.thresholds.memory_warning = 90;
        config.thresholds.memory_critical = 90;
        assert!(config.validate().is_err());

        // Warning greater than critical should be invalid
        config.thresholds.memory_warning = 95;
        config.thresholds.memory_critical = 90;
        assert!(config.validate().is_err());

        // Valid case
        config.thresholds.memory_warning = 75;
        config.thresholds.memory_critical = 90;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_temperature_warning_must_be_less_than_critical() {
        let mut config = create_valid_config();

        // Equal values should be invalid
        config.thresholds.temperature_warning = 85.0;
        config.thresholds.temperature_critical = 85.0;
        assert!(config.validate().is_err());

        // Warning greater than critical should be invalid
        config.thresholds.temperature_warning = 90.0;
        config.thresholds.temperature_critical = 85.0;
        assert!(config.validate().is_err());

        // Valid case
        config.thresholds.temperature_warning = 70.0;
        config.thresholds.temperature_critical = 85.0;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_history_size_boundary_values() {
        let mut config = create_valid_config();

        // Zero is invalid
        config.display.history_size = 0;
        assert!(config.validate().is_err());
        assert_eq!(
            config.validate().unwrap_err(),
            "history_size must be between 1 and 1000"
        );

        // 1001 is invalid
        config.display.history_size = 1001;
        assert!(config.validate().is_err());

        // Valid boundary values
        config.display.history_size = 1;
        assert!(config.validate().is_ok());

        config.display.history_size = 1000;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_merge_with_cli() {
        let mut config = create_valid_config();

        // Test merging refresh_rate
        config.merge_with_cli(Some(5), false, None);
        assert_eq!(config.refresh_rate, 5);
        assert!(!config.minimal_mode);

        // Test merging minimal_mode
        config.merge_with_cli(None, true, None);
        assert_eq!(config.refresh_rate, 5);
        assert!(config.minimal_mode);

        // Test merging both
        config.merge_with_cli(Some(10), true, None);
        assert_eq!(config.refresh_rate, 10);
        assert!(config.minimal_mode);

        // Test no changes when None and false
        config.merge_with_cli(None, false, None);
        assert_eq!(config.refresh_rate, 10);
        assert!(config.minimal_mode);

        // Test merging theme
        config.merge_with_cli(None, false, Some("nord"));
        assert_eq!(config.display.theme, "nord");
    }

    #[test]
    fn test_config_serialization() {
        let config = create_valid_config();
        let serialized = toml::to_string(&config).unwrap();
        let deserialized: Config = toml::from_str(&serialized).unwrap();

        assert_eq!(config.refresh_rate, deserialized.refresh_rate);
        assert_eq!(config.minimal_mode, deserialized.minimal_mode);
        assert_eq!(
            config.thresholds.cpu_warning,
            deserialized.thresholds.cpu_warning
        );
        assert_eq!(
            config.display.history_size,
            deserialized.display.history_size
        );
        assert_eq!(
            config.notifications.enabled,
            deserialized.notifications.enabled
        );
        assert_eq!(
            config.display.theme,
            deserialized.display.theme
        );
    }

    #[test]
    fn test_config_save_and_load() {
        use std::fs;
        use tempfile::NamedTempFile;

        let config = create_valid_config();
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        // Save config
        config.save_to_file(path).unwrap();

        // Load config
        let loaded_config = Config::load_from_file(path).unwrap();

        assert_eq!(config.refresh_rate, loaded_config.refresh_rate);
        assert_eq!(config.minimal_mode, loaded_config.minimal_mode);
    }

    #[test]
    fn test_load_invalid_toml() {
        use std::fs;
        use tempfile::NamedTempFile;

        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        // Write invalid TOML
        fs::write(path, "invalid toml content [[[").unwrap();

        let result = Config::load_from_file(path);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_nonexistent_file() {
        let result = Config::load_from_file("/nonexistent/path/config.toml");
        assert!(result.is_err());
    }
}

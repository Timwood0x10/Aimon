// Configuration management with validation

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::ui::layouts::LayoutType;

/// Process sorting options
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum ProcessSortBy {
    #[default]
    #[serde(rename = "cpu")]
    Cpu,
    #[serde(rename = "memory")]
    Memory,
    #[serde(rename = "pid")]
    Pid,
    #[serde(rename = "name")]
    Name,
}

/// Default language setting
fn default_language() -> String {
    "en".to_string()
}

/// Fan control configuration (behind `fan-control` feature gate)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FanControlConfig {
    /// Enable fan control in config. Runtime code must also receive explicit
    /// operator consent before any write-capable backend can use this value.
    pub enabled: bool,
    /// Runtime-only consent from `--allow-fan-control`.
    #[serde(default, skip_serializing)]
    pub runtime_allowed: bool,
    /// Minimum safe RPM floor (validated against SMC fan min)
    pub safe_min_rpm: u32,
    /// Maximum safe RPM ceiling (validated against SMC fan max)
    pub safe_max_rpm: u32,
    /// Optional target RPM. When absent, write support is available but idle.
    pub target_rpm: Option<u32>,
    /// Restore automatic fan control on exit
    pub restore_on_exit: bool,
}

impl Default for FanControlConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            runtime_allowed: false,
            safe_min_rpm: 1000,
            safe_max_rpm: 6000,
            target_rpm: None,
            restore_on_exit: true,
        }
    }
}

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub refresh_rate: u64,
    /// Refresh interval in milliseconds (0 = use refresh_rate in seconds)
    #[serde(default)]
    pub refresh_rate_ms: u64,
    pub minimal_mode: bool,
    pub thresholds: ThresholdConfig,
    pub display: DisplayConfig,
    pub notifications: NotificationConfig,
    #[serde(default)]
    pub process_sort_by: ProcessSortBy,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default)]
    pub fan_control: FanControlConfig,
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
            refresh_rate_ms: 0,
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
                theme: "mactop_green".to_string(),
            },
            notifications: NotificationConfig {
                enabled: true,
                cooldown_seconds: 30,
            },
            process_sort_by: ProcessSortBy::default(),
            language: default_language(),
            fan_control: FanControlConfig::default(),
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
    pub fn merge_with_cli(
        &mut self,
        refresh_rate: Option<u64>,
        minimal_mode: bool,
        theme: Option<&str>,
    ) {
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

    /// Merge fan-control specific CLI flags
    pub fn merge_fan_control(&mut self, allow_fan_control: bool) {
        if allow_fan_control {
            self.fan_control.runtime_allowed = true;
            log::warn!(
                "Fan-control runtime consent received via --allow-fan-control; write support remains feature-gated and disabled unless config also enables it"
            );
        }
    }

    /// Merge language CLI argument into config
    pub fn merge_language(&mut self, lang: Option<&str>) {
        if let Some(lang_str) = lang {
            if lang_str == "en" || lang_str == "zh" {
                self.language = lang_str.to_string();
            }
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

        if self.language != "en" && self.language != "zh" {
            return Err("language must be 'en' or 'zh'".to_string());
        }

        if self.fan_control.safe_min_rpm >= self.fan_control.safe_max_rpm {
            return Err("fan_control.safe_min_rpm must be less than safe_max_rpm".to_string());
        }

        if let Some(target_rpm) = self.fan_control.target_rpm {
            if target_rpm < self.fan_control.safe_min_rpm
                || target_rpm > self.fan_control.safe_max_rpm
            {
                return Err("fan_control.target_rpm must be inside the safe RPM range".to_string());
            }
        }

        Ok(())
    }
}

/// Runtime layout settings that persist between sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutSettings {
    /// Currently active layout
    #[serde(default)]
    pub current_layout: LayoutType,
    /// Process list sort order
    #[serde(default)]
    pub process_sort_by: ProcessSortBy,
    /// Process list scroll offset
    #[serde(default)]
    pub process_scroll_offset: usize,
    /// Party mode enabled
    #[serde(default)]
    pub party_mode: bool,
}

impl Default for LayoutSettings {
    fn default() -> Self {
        Self {
            current_layout: LayoutType::Full,
            process_sort_by: ProcessSortBy::Cpu,
            process_scroll_offset: 0,
            party_mode: false,
        }
    }
}

impl LayoutSettings {
    /// Get the default path for runtime state file
    fn default_state_path() -> std::path::PathBuf {
        // Use XDG_CONFIG_HOME or ~/.config
        let config_dir = std::env::var("XDG_CONFIG_HOME")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                std::path::PathBuf::from(home).join(".config")
            });

        config_dir.join("system-alert").join("state.toml")
    }

    /// Save runtime state to file
    pub fn save_runtime_state(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::default_state_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        fs::write(&path, content)?;
        Ok(())
    }

    /// Load runtime state from file, returning default if file doesn't exist
    pub fn load_runtime_state() -> Self {
        let path = Self::default_state_path();
        match fs::read_to_string(&path) {
            Ok(content) => match toml::from_str::<LayoutSettings>(&content) {
                Ok(settings) => settings,
                Err(e) => {
                    log::warn!("Failed to parse runtime state: {}. Using defaults.", e);
                    LayoutSettings::default()
                }
            },
            Err(_) => LayoutSettings::default(),
        }
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
        assert_eq!(config.display.theme, deserialized.display.theme);
    }

    #[test]
    fn test_config_save_and_load() {
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

    #[test]
    fn test_config_layout_settings_default() {
        let settings = LayoutSettings::default();
        assert_eq!(settings.current_layout, LayoutType::Full);
        assert_eq!(settings.process_sort_by, ProcessSortBy::Cpu);
        assert_eq!(settings.process_scroll_offset, 0);
        assert!(!settings.party_mode);
    }

    #[test]
    fn test_config_save_load_runtime_state() {
        let settings = LayoutSettings {
            current_layout: LayoutType::Compact,
            process_sort_by: ProcessSortBy::Memory,
            process_scroll_offset: 5,
            party_mode: true,
        };

        // Test serialization roundtrip
        let serialized = toml::to_string_pretty(&settings).unwrap();
        let deserialized: LayoutSettings = toml::from_str(&serialized).unwrap();

        assert_eq!(deserialized.current_layout, LayoutType::Compact);
        assert_eq!(deserialized.process_sort_by, ProcessSortBy::Memory);
        assert_eq!(deserialized.process_scroll_offset, 5);
        assert!(deserialized.party_mode);
    }

    #[test]
    fn test_layout_settings_serialization() {
        let settings = LayoutSettings::default();
        let serialized = toml::to_string(&settings).unwrap();
        let deserialized: LayoutSettings = toml::from_str(&serialized).unwrap();

        assert_eq!(settings.current_layout, deserialized.current_layout);
        assert_eq!(settings.process_sort_by, deserialized.process_sort_by);
        assert_eq!(
            settings.process_scroll_offset,
            deserialized.process_scroll_offset
        );
        assert_eq!(settings.party_mode, deserialized.party_mode);
    }

    // ── FanControlConfig tests ────────────────────────────────────────

    #[test]
    fn test_fan_control_default_values() {
        let fc = FanControlConfig::default();
        assert!(!fc.enabled, "fan control should be disabled by default");
        assert!(
            !fc.runtime_allowed,
            "runtime fan-control consent should be false by default"
        );
        assert_eq!(fc.safe_min_rpm, 1000);
        assert_eq!(fc.safe_max_rpm, 6000);
        assert!(fc.restore_on_exit);
    }

    #[test]
    fn test_fan_control_validation_passes() {
        let mut config = create_valid_config();
        config.fan_control.safe_min_rpm = 1000;
        config.fan_control.safe_max_rpm = 6000;
        assert!(
            config.validate().is_ok(),
            "valid fan_control ranges should pass"
        );
    }

    #[test]
    fn test_fan_control_min_equals_max_is_invalid() {
        let mut config = create_valid_config();
        config.fan_control.safe_min_rpm = 3000;
        config.fan_control.safe_max_rpm = 3000;
        let err = config.validate().unwrap_err();
        assert!(
            err.contains("safe_min_rpm must be less than safe_max_rpm"),
            "got: {}",
            err
        );
    }

    #[test]
    fn test_fan_control_min_greater_than_max_is_invalid() {
        let mut config = create_valid_config();
        config.fan_control.safe_min_rpm = 5000;
        config.fan_control.safe_max_rpm = 2000;
        let err = config.validate().unwrap_err();
        assert!(
            err.contains("safe_min_rpm must be less than safe_max_rpm"),
            "got: {}",
            err
        );
    }

    #[test]
    fn test_merge_fan_control_records_runtime_consent_when_true() {
        let mut config = create_valid_config();
        assert!(!config.fan_control.enabled);
        assert!(!config.fan_control.runtime_allowed);
        config.merge_fan_control(true);
        assert!(
            !config.fan_control.enabled,
            "CLI consent alone must not enable write-capable fan control"
        );
        assert!(
            config.fan_control.runtime_allowed,
            "CLI consent should be recorded separately from config enablement"
        );
    }

    #[test]
    fn test_merge_fan_control_noop_when_false() {
        let mut config = create_valid_config();
        config.fan_control.enabled = false;
        config.merge_fan_control(false);
        assert!(!config.fan_control.enabled);
        assert!(!config.fan_control.runtime_allowed);
    }

    #[test]
    fn test_fan_control_serialization_roundtrip() {
        let mut config = create_valid_config();
        config.fan_control.enabled = true;
        config.fan_control.safe_min_rpm = 1500;
        config.fan_control.safe_max_rpm = 5500;
        config.fan_control.restore_on_exit = false;

        let serialized = toml::to_string(&config).unwrap();
        let deserialized: Config = toml::from_str(&serialized).unwrap();

        assert!(deserialized.fan_control.enabled);
        assert!(
            !deserialized.fan_control.runtime_allowed,
            "runtime consent must not be persisted to config files"
        );
        assert_eq!(deserialized.fan_control.safe_min_rpm, 1500);
        assert_eq!(deserialized.fan_control.safe_max_rpm, 5500);
        assert!(!deserialized.fan_control.restore_on_exit);
    }

    #[test]
    fn test_fan_control_serde_default_on_missing() {
        // Config without fan_control section should deserialize with defaults
        let minimal_toml = r#"
            refresh_rate = 1
            minimal_mode = false
            [thresholds]
            cpu_warning = 75.0
            cpu_critical = 90.0
            memory_warning = 75
            memory_critical = 90
            temperature_warning = 70.0
            temperature_critical = 85.0
            [display]
            show_temperatures = true
            show_network = true
            show_processes = true
            show_history = true
            history_size = 60
            theme = "mactop_green"
            [notifications]
            enabled = true
            cooldown_seconds = 30
        "#;
        let config: Config = toml::from_str(minimal_toml).unwrap();
        assert!(!config.fan_control.enabled);
        assert!(!config.fan_control.runtime_allowed);
        assert_eq!(config.fan_control.safe_min_rpm, 1000);
        assert_eq!(config.fan_control.safe_max_rpm, 6000);
        assert!(config.fan_control.restore_on_exit);
    }
}

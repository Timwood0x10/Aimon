//! Layout system for the UI
//! Provides multiple layout types that can be cycled through

pub mod full;
pub mod minimal;
pub mod compact;
pub mod battery_focus;
pub mod gpu_focus;
pub mod network_focus;
pub mod system_health;

use std::fmt;
use std::str::FromStr;

/// Available layout types for the system monitor
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LayoutType {
    /// Full comprehensive system overview
    Full,
    /// Minimal CPU and memory gauges only
    Minimal,
    /// Dense single-screen with key metrics
    Compact,
    /// Battery and power management focus
    BatteryFocus,
    /// GPU and ANE performance focus
    GpuFocus,
    /// Network interface monitoring
    NetworkFocus,
    /// System health and uptime
    SystemHealth,
}

impl LayoutType {
    /// Returns all available layout types in display order
    pub fn all() -> &'static [LayoutType] {
        &[
            LayoutType::Full,
            LayoutType::Minimal,
            LayoutType::Compact,
            LayoutType::BatteryFocus,
            LayoutType::GpuFocus,
            LayoutType::NetworkFocus,
            LayoutType::SystemHealth,
        ]
    }

    /// Get the numeric key (1-7) for quick jump
    pub fn key_number(&self) -> u8 {
        match self {
            LayoutType::Full => 1,
            LayoutType::Minimal => 2,
            LayoutType::Compact => 3,
            LayoutType::BatteryFocus => 4,
            LayoutType::GpuFocus => 5,
            LayoutType::NetworkFocus => 6,
            LayoutType::SystemHealth => 7,
        }
    }

    /// Create from a key number (1-7)
    pub fn from_key_number(n: u8) -> Option<LayoutType> {
        match n {
            1 => Some(LayoutType::Full),
            2 => Some(LayoutType::Minimal),
            3 => Some(LayoutType::Compact),
            4 => Some(LayoutType::BatteryFocus),
            5 => Some(LayoutType::GpuFocus),
            6 => Some(LayoutType::NetworkFocus),
            7 => Some(LayoutType::SystemHealth),
            _ => None,
        }
    }
}

impl fmt::Display for LayoutType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LayoutType::Full => write!(f, "Full"),
            LayoutType::Minimal => write!(f, "Minimal"),
            LayoutType::Compact => write!(f, "Compact"),
            LayoutType::BatteryFocus => write!(f, "Battery"),
            LayoutType::GpuFocus => write!(f, "GPU"),
            LayoutType::NetworkFocus => write!(f, "Network"),
            LayoutType::SystemHealth => write!(f, "Health"),
        }
    }
}

impl FromStr for LayoutType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "full" => Ok(LayoutType::Full),
            "minimal" | "min" => Ok(LayoutType::Minimal),
            "compact" => Ok(LayoutType::Compact),
            "battery" | "batt" => Ok(LayoutType::BatteryFocus),
            "gpu" => Ok(LayoutType::GpuFocus),
            "network" | "net" => Ok(LayoutType::NetworkFocus),
            "health" | "system_health" => Ok(LayoutType::SystemHealth),
            _ => Err(format!("Unknown layout type: {}", s)),
        }
    }
}

impl Default for LayoutType {
    fn default() -> Self {
        LayoutType::Full
    }
}

impl serde::Serialize for LayoutType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for LayoutType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        LayoutType::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_type_all_count() {
        assert_eq!(LayoutType::all().len(), 7);
    }

    #[test]
    fn test_layout_type_display() {
        assert_eq!(LayoutType::Full.to_string(), "Full");
        assert_eq!(LayoutType::Minimal.to_string(), "Minimal");
        assert_eq!(LayoutType::Compact.to_string(), "Compact");
        assert_eq!(LayoutType::BatteryFocus.to_string(), "Battery");
        assert_eq!(LayoutType::GpuFocus.to_string(), "GPU");
        assert_eq!(LayoutType::NetworkFocus.to_string(), "Network");
        assert_eq!(LayoutType::SystemHealth.to_string(), "Health");
    }

    #[test]
    fn test_layout_type_from_str() {
        assert_eq!(LayoutType::from_str("full").unwrap(), LayoutType::Full);
        assert_eq!(LayoutType::from_str("minimal").unwrap(), LayoutType::Minimal);
        assert_eq!(LayoutType::from_str("min").unwrap(), LayoutType::Minimal);
        assert_eq!(LayoutType::from_str("compact").unwrap(), LayoutType::Compact);
        assert_eq!(LayoutType::from_str("battery").unwrap(), LayoutType::BatteryFocus);
        assert_eq!(LayoutType::from_str("batt").unwrap(), LayoutType::BatteryFocus);
        assert_eq!(LayoutType::from_str("gpu").unwrap(), LayoutType::GpuFocus);
        assert_eq!(LayoutType::from_str("network").unwrap(), LayoutType::NetworkFocus);
        assert_eq!(LayoutType::from_str("net").unwrap(), LayoutType::NetworkFocus);
        assert_eq!(LayoutType::from_str("health").unwrap(), LayoutType::SystemHealth);
        assert!(LayoutType::from_str("invalid").is_err());
    }

    #[test]
    fn test_layout_type_from_key_number() {
        assert_eq!(LayoutType::from_key_number(1), Some(LayoutType::Full));
        assert_eq!(LayoutType::from_key_number(7), Some(LayoutType::SystemHealth));
        assert_eq!(LayoutType::from_key_number(0), None);
        assert_eq!(LayoutType::from_key_number(8), None);
    }

    #[test]
    fn test_layout_type_key_number() {
        assert_eq!(LayoutType::Full.key_number(), 1);
        assert_eq!(LayoutType::SystemHealth.key_number(), 7);
    }
}

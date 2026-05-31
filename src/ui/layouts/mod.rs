//! Layout system for the UI
//! Provides multiple layout types that can be cycled through

pub mod advanced;
pub mod battery_focus;
pub mod compact;
pub mod full;
pub mod gpu_focus;
pub mod minimal;
pub mod network_focus;
pub mod system_health;
pub mod thermals;

use std::fmt;
use std::str::FromStr;

/// Available layout types for the system monitor
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LayoutType {
    /// Startup summary page
    Startup,
    /// Full comprehensive system overview
    #[default]
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
    /// Advanced layout matching mactop's full feature set
    Advanced,
    /// Thermals and fan monitoring focus
    Thermals,
}

impl LayoutType {
    /// Returns all available layout types in display order
    pub fn all() -> &'static [LayoutType] {
        &[
            LayoutType::Startup,
            LayoutType::Full,
            LayoutType::Advanced,
            LayoutType::Minimal,
            LayoutType::Compact,
            LayoutType::BatteryFocus,
            LayoutType::GpuFocus,
            LayoutType::NetworkFocus,
            LayoutType::SystemHealth,
            LayoutType::Thermals,
        ]
    }

    /// Get the numeric key (1-9) for quick jump
    pub fn key_number(&self) -> u8 {
        match self {
            LayoutType::Startup => 0,
            LayoutType::Full => 1,
            LayoutType::Advanced => 2,
            LayoutType::Minimal => 3,
            LayoutType::Compact => 4,
            LayoutType::BatteryFocus => 5,
            LayoutType::GpuFocus => 6,
            LayoutType::NetworkFocus => 7,
            LayoutType::SystemHealth => 8,
            LayoutType::Thermals => 9,
        }
    }

    /// Create from a key number (1-9)
    pub fn from_key_number(n: u8) -> Option<LayoutType> {
        match n {
            0 => Some(LayoutType::Startup),
            1 => Some(LayoutType::Full),
            2 => Some(LayoutType::Advanced),
            3 => Some(LayoutType::Minimal),
            4 => Some(LayoutType::Compact),
            5 => Some(LayoutType::BatteryFocus),
            6 => Some(LayoutType::GpuFocus),
            7 => Some(LayoutType::NetworkFocus),
            8 => Some(LayoutType::SystemHealth),
            9 => Some(LayoutType::Thermals),
            _ => None,
        }
    }
}

impl fmt::Display for LayoutType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LayoutType::Startup => write!(f, "Startup"),
            LayoutType::Full => write!(f, "Full"),
            LayoutType::Advanced => write!(f, "Advanced"),
            LayoutType::Minimal => write!(f, "Minimal"),
            LayoutType::Compact => write!(f, "Compact"),
            LayoutType::BatteryFocus => write!(f, "Battery"),
            LayoutType::GpuFocus => write!(f, "GPU"),
            LayoutType::NetworkFocus => write!(f, "Network"),
            LayoutType::SystemHealth => write!(f, "Health"),
            LayoutType::Thermals => write!(f, "Thermals"),
        }
    }
}

impl FromStr for LayoutType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "startup" | "start" | "home" => Ok(LayoutType::Startup),
            "full" => Ok(LayoutType::Full),
            "advanced" | "adv" => Ok(LayoutType::Advanced),
            "minimal" | "min" => Ok(LayoutType::Minimal),
            "compact" => Ok(LayoutType::Compact),
            "battery" | "batt" => Ok(LayoutType::BatteryFocus),
            "gpu" => Ok(LayoutType::GpuFocus),
            "network" | "net" => Ok(LayoutType::NetworkFocus),
            "health" | "system_health" => Ok(LayoutType::SystemHealth),
            "thermals" | "thermal" | "fan" => Ok(LayoutType::Thermals),
            _ => Err(format!("Unknown layout type: {}", s)),
        }
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
        assert_eq!(LayoutType::all().len(), 10);
    }

    #[test]
    fn test_layout_type_display() {
        assert_eq!(LayoutType::Startup.to_string(), "Startup");
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
        assert_eq!(
            LayoutType::from_str("startup").unwrap(),
            LayoutType::Startup
        );
        assert_eq!(LayoutType::from_str("full").unwrap(), LayoutType::Full);
        assert_eq!(
            LayoutType::from_str("minimal").unwrap(),
            LayoutType::Minimal
        );
        assert_eq!(LayoutType::from_str("min").unwrap(), LayoutType::Minimal);
        assert_eq!(
            LayoutType::from_str("compact").unwrap(),
            LayoutType::Compact
        );
        assert_eq!(
            LayoutType::from_str("battery").unwrap(),
            LayoutType::BatteryFocus
        );
        assert_eq!(
            LayoutType::from_str("batt").unwrap(),
            LayoutType::BatteryFocus
        );
        assert_eq!(LayoutType::from_str("gpu").unwrap(), LayoutType::GpuFocus);
        assert_eq!(
            LayoutType::from_str("network").unwrap(),
            LayoutType::NetworkFocus
        );
        assert_eq!(
            LayoutType::from_str("net").unwrap(),
            LayoutType::NetworkFocus
        );
        assert_eq!(
            LayoutType::from_str("health").unwrap(),
            LayoutType::SystemHealth
        );
        assert!(LayoutType::from_str("invalid").is_err());
    }

    #[test]
    fn test_layout_type_from_key_number() {
        assert_eq!(LayoutType::from_key_number(0), Some(LayoutType::Startup));
        assert_eq!(LayoutType::from_key_number(1), Some(LayoutType::Full));
        assert_eq!(LayoutType::from_key_number(9), Some(LayoutType::Thermals));
        assert_eq!(LayoutType::from_key_number(10), None);
    }

    #[test]
    fn test_layout_type_key_number() {
        assert_eq!(LayoutType::Startup.key_number(), 0);
        assert_eq!(LayoutType::Full.key_number(), 1);
        assert_eq!(LayoutType::SystemHealth.key_number(), 8);
    }
}

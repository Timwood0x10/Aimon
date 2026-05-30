use serde::{Deserialize, Serialize};

/// Unique identifier for each achievement
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AchievementId {
    IceCold,
    PowerSaver,
    Marathon,
    FireHazard,
    MemoryHog,
    NetworkFlood,
    ThermalThrottle,
    FullCharge,
    Centurion,
    Balanced,
    Overclocker,
    NightOwl,
}

/// Definition of an achievement
#[derive(Debug, Clone)]
pub struct AchievementDef {
    pub id: AchievementId,
    pub name: &'static str,
    pub description: &'static str,
    pub icon: &'static str,
}

/// Get all achievement definitions
pub fn all_achievements() -> Vec<AchievementDef> {
    vec![
        AchievementDef {
            id: AchievementId::IceCold,
            name: "Ice Cold",
            description: "CPU temperature below 40C",
            icon: "SNOWFLAKE",
        },
        AchievementDef {
            id: AchievementId::PowerSaver,
            name: "Power Saver",
            description: "Battery above 80% with low power draw",
            icon: "BATTERY",
        },
        AchievementDef {
            id: AchievementId::Marathon,
            name: "Marathon",
            description: "System uptime exceeds 24 hours",
            icon: "RUNNER",
        },
        AchievementDef {
            id: AchievementId::FireHazard,
            name: "Fire Hazard",
            description: "CPU temperature above 90C",
            icon: "FIRE",
        },
        AchievementDef {
            id: AchievementId::MemoryHog,
            name: "Memory Hog",
            description: "Memory usage above 90%",
            icon: "PIG",
        },
        AchievementDef {
            id: AchievementId::NetworkFlood,
            name: "Network Flood",
            description: "Network throughput exceeds 100 MB/s",
            icon: "WAVE",
        },
        AchievementDef {
            id: AchievementId::ThermalThrottle,
            name: "Thermal Throttle",
            description: "System is thermal throttling",
            icon: "THERMOMETER",
        },
        AchievementDef {
            id: AchievementId::FullCharge,
            name: "Full Charge",
            description: "Battery at 100%",
            icon: "LIGHTNING",
        },
        AchievementDef {
            id: AchievementId::Centurion,
            name: "Centurion",
            description: "CPU usage at 100% on any core",
            icon: "HUNDRED",
        },
        AchievementDef {
            id: AchievementId::Balanced,
            name: "Balanced",
            description: "CPU and memory both between 40-60%",
            icon: "BALANCE",
        },
        AchievementDef {
            id: AchievementId::Overclocker,
            name: "Overclocker",
            description: "P-Cluster frequency above 3500 MHz",
            icon: "ROCKET",
        },
        AchievementDef {
            id: AchievementId::NightOwl,
            name: "Night Owl",
            description: "Active between 12 AM and 5 AM",
            icon: "OWL",
        },
    ]
}

/// Get definition by ID
pub fn get_definition(id: AchievementId) -> Option<AchievementDef> {
    all_achievements().into_iter().find(|a| a.id == id)
}

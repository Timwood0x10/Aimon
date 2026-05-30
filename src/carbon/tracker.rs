use serde::{Deserialize, Serialize};

/// Carbon dioxide equivalent constants
const CO2_PER_KWH: f64 = 0.5; // kg CO2 per kWh (average grid)
const PHONE_CHARGE_WH: f64 = 12.0; // Wh per phone charge
const LIGHTBULB_60W_HOURS: f64 = 60.0; // Wh per hour of 60W lightbulb

/// Environmental equivalents for carbon footprint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CarbonEquivalent {
    pub phone_charges: f64,
    pub lightbulb_hours: f64,
    pub description: String,
}

/// Tracks energy consumption and carbon footprint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CarbonTracker {
    /// Total energy consumed in watt-hours
    pub total_energy_wh: f64,
    /// Total carbon emissions in kg CO2
    pub carbon_kg: f64,
    /// Number of recordings
    pub recording_count: u64,
}

impl CarbonTracker {
    /// Create a new carbon tracker
    pub fn new() -> Self {
        Self {
            total_energy_wh: 0.0,
            carbon_kg: 0.0,
            recording_count: 0,
        }
    }

    /// Record a power measurement
    /// - watts: current power draw in watts
    /// - elapsed_secs: time period in seconds for this measurement
    pub fn record(&mut self, watts: f64, elapsed_secs: f64) {
        if watts <= 0.0 || elapsed_secs <= 0.0 {
            return;
        }

        // Convert watts * seconds to watt-hours
        let energy_wh = watts * elapsed_secs / 3600.0;
        self.total_energy_wh += energy_wh;

        // Calculate carbon: wh -> kwh -> kg CO2
        self.carbon_kg = self.total_energy_wh / 1000.0 * CO2_PER_KWH;

        self.recording_count += 1;
    }

    /// Get total energy in kilowatt-hours
    pub fn total_energy_kwh(&self) -> f64 {
        self.total_energy_wh / 1000.0
    }

    /// Get carbon equivalents for display
    pub fn get_equivalents(&self) -> CarbonEquivalent {
        let phone_charges = self.total_energy_wh / PHONE_CHARGE_WH;
        let lightbulb_hours = self.total_energy_wh / LIGHTBULB_60W_HOURS;

        let description = if self.carbon_kg < 0.001 {
            "Minimal impact".to_string()
        } else if self.carbon_kg < 0.01 {
            format!(
                "Like charging your phone {:.0} times",
                phone_charges
            )
        } else if self.carbon_kg < 0.1 {
            format!(
                "Like a 60W bulb on for {:.1} hours",
                lightbulb_hours
            )
        } else {
            format!(
                "{:.1} phone charges or {:.1} hours of light",
                phone_charges, lightbulb_hours
            )
        };

        CarbonEquivalent {
            phone_charges,
            lightbulb_hours,
            description,
        }
    }

    /// Reset all tracking data
    pub fn reset(&mut self) {
        self.total_energy_wh = 0.0;
        self.carbon_kg = 0.0;
        self.recording_count = 0;
    }
}

impl Default for CarbonTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_calculation() {
        let mut tracker = CarbonTracker::new();

        // 100W for 1 hour = 100 Wh = 0.1 kWh
        tracker.record(100.0, 3600.0);

        assert!((tracker.total_energy_wh - 100.0).abs() < 0.001);
        assert!((tracker.total_energy_kwh() - 0.1).abs() < 0.001);
        // 0.1 kWh * 0.5 kg/kWh = 0.05 kg CO2
        assert!((tracker.carbon_kg - 0.05).abs() < 0.001);
    }

    #[test]
    fn test_phone_equivalent() {
        let mut tracker = CarbonTracker::new();

        // 12 Wh = 1 phone charge
        tracker.record(12.0, 3600.0);

        let equiv = tracker.get_equivalents();
        assert!((equiv.phone_charges - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_lightbulb_equivalent() {
        let mut tracker = CarbonTracker::new();

        // 60 Wh = 1 hour of 60W lightbulb
        tracker.record(60.0, 3600.0);

        let equiv = tracker.get_equivalents();
        assert!((equiv.lightbulb_hours - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_reset() {
        let mut tracker = CarbonTracker::new();

        tracker.record(100.0, 3600.0);
        assert!(tracker.total_energy_wh > 0.0);

        tracker.reset();
        assert!((tracker.total_energy_wh).abs() < f64::EPSILON);
        assert!((tracker.carbon_kg).abs() < f64::EPSILON);
        assert_eq!(tracker.recording_count, 0);
    }

    #[test]
    fn test_zero_power() {
        let mut tracker = CarbonTracker::new();

        // Zero power should not accumulate
        tracker.record(0.0, 3600.0);
        assert!((tracker.total_energy_wh).abs() < f64::EPSILON);

        // Zero time should not accumulate
        tracker.record(100.0, 0.0);
        assert!((tracker.total_energy_wh).abs() < f64::EPSILON);
    }
}

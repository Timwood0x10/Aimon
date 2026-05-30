use std::collections::VecDeque;
use super::engine::{Anomaly, AnomalySeverity};

/// Configurable anomaly detector with rolling window statistics
pub struct AnomalyDetector {
    /// Number of standard deviations to trigger anomaly (default 2.0)
    pub sensitivity: f32,
    /// Rolling window size for statistics
    window_size: usize,
    /// Number of recent points to check for anomalies
    check_count: usize,
}

impl AnomalyDetector {
    /// Create a new detector with default sensitivity (2.0 std devs)
    pub fn new() -> Self {
        Self {
            sensitivity: 2.0,
            window_size: 30,
            check_count: 5,
        }
    }

    /// Create a detector with custom sensitivity
    pub fn with_sensitivity(sensitivity: f32) -> Self {
        Self {
            sensitivity: sensitivity.max(0.5),
            window_size: 30,
            check_count: 5,
        }
    }

    /// Set custom window size
    pub fn with_window_size(mut self, size: usize) -> Self {
        self.window_size = size.max(5);
        self
    }

    /// Analyze a history buffer for anomalies
    pub fn detect(&self, history: &VecDeque<f32>, metric_name: &str) -> Vec<Anomaly> {
        let mut anomalies = Vec::new();
        let values: Vec<f32> = history.iter().copied().collect();

        if values.len() < self.window_size.min(10) {
            return anomalies;
        }

        let check = self.check_count.min(values.len());
        let window = self.window_size.min(values.len());

        for i in (values.len() - check)..values.len() {
            let window_start = if i >= window { i - window } else { 0 };
            let slice = &values[window_start..i];

            if slice.len() < 3 {
                continue;
            }

            let mean: f32 = slice.iter().sum::<f32>() / slice.len() as f32;
            let variance: f32 = slice
                .iter()
                .map(|v| (v - mean).powi(2))
                .sum::<f32>()
                / slice.len() as f32;
            let std_dev = variance.sqrt();

            if std_dev < f32::EPSILON {
                continue;
            }

            let z_score = (values[i] - mean).abs() / std_dev;

            if z_score >= self.sensitivity {
                let severity = if z_score >= self.sensitivity * 2.0 {
                    AnomalySeverity::High
                } else if z_score >= self.sensitivity * 1.5 {
                    AnomalySeverity::Medium
                } else {
                    AnomalySeverity::Low
                };

                anomalies.push(Anomaly {
                    metric_name: metric_name.to_string(),
                    value: values[i],
                    mean,
                    std_dev,
                    z_score,
                    severity,
                });
            }
        }

        anomalies
    }

    /// Detect anomalies across multiple metrics at once
    pub fn detect_all(
        &self,
        metrics: &[(&str, &VecDeque<f32>)],
    ) -> Vec<Anomaly> {
        let mut all_anomalies = Vec::new();
        for (name, history) in metrics {
            all_anomalies.extend(self.detect(history, name));
        }
        all_anomalies
    }
}

impl Default for AnomalyDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_sensitivity() {
        let detector = AnomalyDetector::with_sensitivity(1.5);
        assert!((detector.sensitivity - 1.5).abs() < f32::EPSILON);

        // Sensitivity clamped to minimum 0.5
        let detector = AnomalyDetector::with_sensitivity(0.1);
        assert!((detector.sensitivity - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_detect_with_spike() {
        let detector = AnomalyDetector::new();
        let mut history = VecDeque::new();

        for i in 0..20 {
            history.push_back(50.0 + (i as f32 * 0.1).sin());
        }
        history.push_back(120.0); // spike

        let anomalies = detector.detect(&history, "cpu");
        assert!(!anomalies.is_empty());
    }

    #[test]
    fn test_detect_all_multiple_metrics() {
        let detector = AnomalyDetector::new();

        let mut cpu = VecDeque::new();
        let mut mem = VecDeque::new();

        for i in 0..20 {
            cpu.push_back(50.0 + (i as f32 * 0.1).sin());
            mem.push_back(60.0 + (i as f32 * 0.05).sin());
        }
        cpu.push_back(150.0);
        mem.push_back(200.0);

        let metrics: Vec<(&str, &VecDeque<f32>)> = vec![("cpu", &cpu), ("mem", &mem)];
        let anomalies = detector.detect_all(&metrics);
        assert!(anomalies.len() >= 2);
    }
}

use std::collections::VecDeque;
use serde::{Deserialize, Serialize};

/// Battery drain prediction result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatteryPrediction {
    pub drain_rate: f32,        // percentage per minute
    pub minutes_remaining: f32, // estimated minutes until empty
    pub confidence: f32,        // 0.0 to 1.0
}

/// Memory exhaustion prediction result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryPrediction {
    pub growth_rate: f32,        // percentage per minute
    pub minutes_until_full: f32, // estimated minutes until 100%
    pub current_pct: f32,        // current memory usage percentage
    pub confidence: f32,         // 0.0 to 1.0
}

/// Detected anomaly in system metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anomaly {
    pub metric_name: String,
    pub value: f32,
    pub mean: f32,
    pub std_dev: f32,
    pub z_score: f32,
    pub severity: AnomalySeverity,
}

/// Severity levels for anomalies
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnomalySeverity {
    Low,      // z-score 2-3
    Medium,   // z-score 3-4
    High,     // z-score > 4
}

/// Prediction engine for system metrics
pub struct PredictionEngine;

impl PredictionEngine {
    /// Perform linear regression on a slice of values
    /// Returns (slope, intercept) or None if insufficient data
    pub fn linear_regression(y: &[f32]) -> Option<(f32, f32)> {
        let n = y.len();
        if n < 2 {
            return None;
        }

        let n_f = n as f32;
        let x_mean = (n_f - 1.0) / 2.0;
        let y_mean: f32 = y.iter().sum::<f32>() / n_f;

        let mut numerator = 0.0;
        let mut denominator = 0.0;

        for (i, &val) in y.iter().enumerate() {
            let x_diff = i as f32 - x_mean;
            numerator += x_diff * (val - y_mean);
            denominator += x_diff * x_diff;
        }

        if denominator.abs() < f32::EPSILON {
            return None;
        }

        let slope = numerator / denominator;
        let intercept = y_mean - slope * x_mean;
        Some((slope, intercept))
    }

    /// Predict battery drain based on history
    /// history should contain battery percentage values (most recent last)
    pub fn predict_battery_drain(
        history: &VecDeque<f32>,
        current_battery_pct: f32,
    ) -> Option<BatteryPrediction> {
        if history.len() < 3 || current_battery_pct <= 0.0 {
            return None;
        }

        let values: Vec<f32> = history.iter().copied().collect();
        let (slope, intercept) = Self::linear_regression(&values)?;

        // slope is percentage per sample; negative slope means draining
        let drain_rate = -slope;

        if drain_rate <= 0.0 {
            // Battery is charging or stable
            return Some(BatteryPrediction {
                drain_rate: 0.0,
                minutes_remaining: f32::INFINITY,
                confidence: 0.3,
            });
        }

        let minutes_remaining = current_battery_pct / drain_rate;

        // Confidence based on data consistency (R-squared approximation)
        let confidence = Self::calculate_r_squared(&values, slope, intercept);

        Some(BatteryPrediction {
            drain_rate,
            minutes_remaining,
            confidence: confidence.clamp(0.0, 1.0),
        })
    }

    /// Predict memory exhaustion based on history
    pub fn predict_memory_exhaustion(history: &VecDeque<u16>) -> Option<MemoryPrediction> {
        if history.len() < 3 {
            return None;
        }

        let values: Vec<f32> = history.iter().map(|&x| x as f32).collect();
        let (slope, intercept) = Self::linear_regression(&values)?;

        let current_pct = *values.last().unwrap();

        if slope <= 0.0 {
            // Memory usage is decreasing or stable
            return Some(MemoryPrediction {
                growth_rate: 0.0,
                minutes_until_full: f32::INFINITY,
                current_pct,
                confidence: 0.3,
            });
        }

        let remaining_pct = 100.0 - current_pct;
        let minutes_until_full = remaining_pct / slope;

        let confidence = Self::calculate_r_squared(&values, slope, intercept);

        Some(MemoryPrediction {
            growth_rate: slope,
            minutes_until_full,
            current_pct,
            confidence: confidence.clamp(0.0, 1.0),
        })
    }

    /// Detect anomalies in a history buffer
    pub fn detect_anomalies(history: &VecDeque<f32>, metric_name: &str) -> Vec<Anomaly> {
        let mut anomalies = Vec::new();
        let values: Vec<f32> = history.iter().copied().collect();

        if values.len() < 10 {
            return anomalies;
        }

        // Use rolling window for mean/stddev calculation
        let window_size = 30.min(values.len());
        let check_count = 5.min(values.len());

        for i in (values.len() - check_count)..values.len() {
            let window_start = if i >= window_size { i - window_size } else { 0 };
            let window = &values[window_start..i];

            if window.is_empty() {
                continue;
            }

            let mean: f32 = window.iter().sum::<f32>() / window.len() as f32;
            let variance: f32 = window.iter().map(|v| (v - mean).powi(2)).sum::<f32>()
                / window.len() as f32;
            let std_dev = variance.sqrt();

            if std_dev < f32::EPSILON {
                continue;
            }

            let z_score = (values[i] - mean).abs() / std_dev;

            if z_score >= 2.0 {
                let severity = if z_score >= 4.0 {
                    AnomalySeverity::High
                } else if z_score >= 3.0 {
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

    /// Calculate R-squared (coefficient of determination)
    fn calculate_r_squared(y: &[f32], slope: f32, intercept: f32) -> f32 {
        let n = y.len() as f32;
        let y_mean: f32 = y.iter().sum::<f32>() / n;

        let mut ss_res = 0.0;
        let mut ss_tot = 0.0;

        for (i, &val) in y.iter().enumerate() {
            let predicted = slope * i as f32 + intercept;
            ss_res += (val - predicted).powi(2);
            ss_tot += (val - y_mean).powi(2);
        }

        if ss_tot.abs() < f32::EPSILON {
            return 0.0;
        }

        1.0 - (ss_res / ss_tot)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_regression_basic() {
        // Perfect linear relationship: y = 2x + 1
        let y = vec![1.0, 3.0, 5.0, 7.0, 9.0];
        let (slope, intercept) = PredictionEngine::linear_regression(&y).unwrap();
        assert!((slope - 2.0).abs() < 0.001);
        assert!((intercept - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_linear_regression_insufficient_data() {
        let y = vec![5.0];
        assert!(PredictionEngine::linear_regression(&y).is_none());

        let y_empty: Vec<f32> = vec![];
        assert!(PredictionEngine::linear_regression(&y_empty).is_none());
    }

    #[test]
    fn test_battery_prediction() {
        let mut history = VecDeque::new();
        // Battery draining from 80 to 65 over 15 samples
        for i in 0..15 {
            history.push_back(80.0 - i as f32);
        }

        let result = PredictionEngine::predict_battery_drain(&history, 65.0);
        assert!(result.is_some());
        let pred = result.unwrap();
        assert!(pred.drain_rate > 0.0);
        assert!(pred.minutes_remaining > 0.0);
        assert!(pred.confidence > 0.0);
    }

    #[test]
    fn test_memory_prediction() {
        let mut history = VecDeque::new();
        // Memory growing from 40% to 55%
        for i in 0..15 {
            history.push_back((40 + i) as u16);
        }

        let result = PredictionEngine::predict_memory_exhaustion(&history);
        assert!(result.is_some());
        let pred = result.unwrap();
        assert!(pred.growth_rate > 0.0);
        assert!(pred.minutes_until_full > 0.0);
        assert!(pred.current_pct > 0.0);
    }

    #[test]
    fn test_anomaly_spike() {
        let mut history = VecDeque::new();
        // Normal values around 50 with low variance
        for i in 0..20 {
            history.push_back(50.0 + (i as f32 * 0.1).sin());
        }
        // Add a spike
        history.push_back(100.0);

        let anomalies = PredictionEngine::detect_anomalies(&history, "test_metric");
        assert!(!anomalies.is_empty());
        assert_eq!(anomalies[0].metric_name, "test_metric");
        assert!(anomalies[0].z_score >= 2.0);
    }

    #[test]
    fn test_anomaly_no_anomalies() {
        let mut history = VecDeque::new();
        // Consistent values with no anomalies
        for i in 0..20 {
            history.push_back(50.0 + (i as f32 * 0.01).sin());
        }

        let anomalies = PredictionEngine::detect_anomalies(&history, "test_metric");
        assert!(anomalies.is_empty());
    }
}

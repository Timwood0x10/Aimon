//! Performance metrics collector
//! Handles collection of performance and efficiency metrics

use crate::types::*;

/// Collect performance metrics
pub async fn collect_performance_metrics(
    cpu_info: &CpuInfo,
    total_power: f64,
) -> PerformanceMetrics {
    let mut metrics = PerformanceMetrics::default();

    // Calculate instructions per watt (estimated)
    if total_power > 0.0 {
        let estimated_instructions = cpu_info.average_usage as f64 * 1000000.0; // Simplified
        metrics.instructions_per_watt = estimated_instructions / total_power;
    }

    // Calculate performance per watt
    if total_power > 0.0 {
        metrics.performance_per_watt = cpu_info.average_usage as f64 / total_power;
    }

    // Calculate frequency efficiency
    let avg_freq = (cpu_info.power_metrics.e_cluster_freq_mhz
        + cpu_info.power_metrics.p_cluster_freq_mhz) as f64
        / 2.0;
    if avg_freq > 0.0 {
        metrics.frequency_efficiency = cpu_info.average_usage as f64 / avg_freq * 1000.0;
    }

    // Determine workload type
    metrics.workload_type = if cpu_info.average_usage < 10.0 {
        "idle".to_string()
    } else if cpu_info.power_metrics.gpu_w > cpu_info.power_metrics.cpu_w {
        "graphics".to_string()
    } else if cpu_info.average_usage > 70.0 {
        "compute".to_string()
    } else {
        "mixed".to_string()
    };

    metrics
}

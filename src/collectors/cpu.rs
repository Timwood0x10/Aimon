//! CPU data collector
//! Handles collection of CPU metrics and power information
//! Uses the PowermetricsSampler backend for power data parsing
//! and sysinfo for CPU usage.

use crate::collectors::backends::powermetrics::PowermetricsSampler;
use crate::types::*;
use sysinfo::System;

/// Collect CPU information including usage and power metrics
pub async fn collect_cpu_info(
    system: &System,
    sampler: &mut PowermetricsSampler,
) -> Result<CpuInfo, Box<dyn std::error::Error>> {
    let cpu_usages: Vec<f32> = system.cpus().iter().map(|cpu| cpu.cpu_usage()).collect();

    let average_usage = if !cpu_usages.is_empty() {
        cpu_usages.iter().sum::<f32>() / cpu_usages.len() as f32
    } else {
        0.0
    };

    // Use PowermetricsSampler for power metrics (handles caching internally)
    let power_metrics = match sampler.sample().await {
        Ok(m) => m,
        Err(e) => {
            log::warn!("Powermetrics failed, using fallback: {}", e);
            get_fallback_cpu_metrics(system)
        }
    };

    let (usage_meta, power_meta) = if power_metrics.package_w > 0.0 || power_metrics.cpu_w > 0.0 {
        (
            MetricMeta::measured(MetricSource::Sysinfo),
            MetricMeta::measured(MetricSource::Powermetrics),
        )
    } else {
        (
            MetricMeta::measured(MetricSource::Sysinfo),
            MetricMeta::estimated(
                MetricSource::Estimate,
                "powermetrics unavailable, computed fallback",
            ),
        )
    };

    Ok(CpuInfo {
        core_usages: cpu_usages,
        average_usage,
        power_metrics,
        usage_meta,
        power_meta,
    })
}

/// Get fallback CPU metrics based on real CPU usage
pub(crate) fn get_fallback_cpu_metrics(system: &System) -> CPUMetrics {
    let avg_usage =
        system.cpus().iter().map(|cpu| cpu.cpu_usage()).sum::<f32>() / system.cpus().len() as f32;

    let estimated_power = (avg_usage / 100.0) * 15.0;

    CPUMetrics {
        e_cluster_active: (avg_usage * 0.6) as i32,
        p_cluster_active: (avg_usage * 0.4) as i32,
        e_cluster_freq_mhz: if avg_usage > 50.0 { 2400 } else { 1800 },
        p_cluster_freq_mhz: if avg_usage > 70.0 { 3200 } else { 2400 },
        ane_w: (estimated_power * 0.05) as f64,
        cpu_w: (estimated_power * 0.6) as f64,
        gpu_w: (estimated_power * 0.2) as f64,
        dram_w: (estimated_power * 0.1) as f64,
        package_w: estimated_power as f64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collectors::backends::powermetrics::parse_powermetrics_output;

    #[test]
    fn test_cpumetrics_default() {
        let metrics = CPUMetrics::default();
        assert_eq!(metrics.e_cluster_active, 0);
        assert_eq!(metrics.p_cluster_active, 0);
        assert_eq!(metrics.e_cluster_freq_mhz, 0);
        assert_eq!(metrics.p_cluster_freq_mhz, 0);
        assert_eq!(metrics.cpu_w, 0.0);
        assert_eq!(metrics.gpu_w, 0.0);
        assert_eq!(metrics.ane_w, 0.0);
        assert_eq!(metrics.package_w, 0.0);
    }

    #[test]
    fn test_cpumetrics_display() {
        let metrics = CPUMetrics {
            e_cluster_active: 50,
            p_cluster_active: 75,
            e_cluster_freq_mhz: 2400,
            p_cluster_freq_mhz: 3200,
            cpu_w: 10.5,
            gpu_w: 5.2,
            ane_w: 1.1,
            dram_w: 2.0,
            package_w: 16.8,
        };

        let display = format!("{}", metrics);
        assert!(display.contains("E-Cluster Active: 50%"));
        assert!(display.contains("P-Cluster Active: 75%"));
        assert!(display.contains("E-Cluster Freq: 2400 MHz"));
        assert!(display.contains("P-Cluster Freq: 3200 MHz"));
        assert!(display.contains("CPU Power: 10.50W"));
        assert!(display.contains("GPU Power: 5.20W"));
        assert!(display.contains("ANE Power: 1.10W"));
        assert!(display.contains("Package Power: 16.80W"));
    }

    #[tokio::test]
    async fn test_parse_powermetrics_empty_input() {
        let result = parse_powermetrics_output("").unwrap();
        assert_eq!(result.e_cluster_active, 0);
        assert_eq!(result.p_cluster_active, 0);
    }

    #[tokio::test]
    async fn test_parse_powermetrics_with_power_data() {
        let output = "ANE Power: 150mW\nCPU Power: 5000mW\nGPU Power: 2000mW\nCombined Power (CPU + GPU + ANE): 7150mW\n";
        let metrics = parse_powermetrics_output(output).unwrap();
        assert_eq!(metrics.ane_w, 0.15);
        assert_eq!(metrics.cpu_w, 5.0);
        assert_eq!(metrics.gpu_w, 2.0);
        assert_eq!(metrics.package_w, 7.15);
    }

    #[tokio::test]
    async fn test_parse_powermetrics_with_dram() {
        let output = "DRAM Power: 800mW\n";
        let metrics = parse_powermetrics_output(output).unwrap();
        assert!((metrics.dram_w - 0.8).abs() < 1e-10);
    }

    #[tokio::test]
    async fn test_parse_powermetrics_mixed_data() {
        let output = r#"
CPU 0 active residency: 45.2%
CPU 1 active residency: 55.8%
CPU 4 active residency: 85.6%
CPU 5 active residency: 92.4%
CPU 0 frequency: 1800 MHz
CPU 1 frequency: 2000 MHz
CPU 4 frequency: 3000 MHz
CPU 5 frequency: 3200 MHz
ANE Power: 100mW
CPU Power: 4500mW
GPU Power: 1500mW
Combined Power (CPU + GPU + ANE): 6100mW
"#;
        let metrics = parse_powermetrics_output(output).unwrap();
        assert_eq!(metrics.e_cluster_active, 50); // (45.2+55.8)/2
        assert_eq!(metrics.p_cluster_active, 89); // (85.6+92.4)/2
        assert_eq!(metrics.e_cluster_freq_mhz, 1900); // (1800+2000)/2
        assert_eq!(metrics.p_cluster_freq_mhz, 3100); // (3000+3200)/2
        assert_eq!(metrics.ane_w, 0.1);
        assert_eq!(metrics.cpu_w, 4.5);
        assert_eq!(metrics.gpu_w, 1.5);
        assert_eq!(metrics.package_w, 6.1);
    }

    #[test]
    fn test_get_fallback_cpu_metrics() {
        let system = System::new_all();
        let metrics = get_fallback_cpu_metrics(&system);

        assert!(metrics.e_cluster_active >= 0);
        assert!(metrics.p_cluster_active >= 0);
        assert!(metrics.e_cluster_freq_mhz >= 0);
        assert!(metrics.p_cluster_freq_mhz >= 0);
        assert!(metrics.cpu_w >= 0.0);
        assert!(metrics.gpu_w >= 0.0);
        assert!(metrics.ane_w >= 0.0);
        assert!(metrics.package_w >= 0.0);
    }
}

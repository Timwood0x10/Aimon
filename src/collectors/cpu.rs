//! CPU data collector
//! Handles collection of CPU metrics and power information

use crate::cli::get_powermetrics_output;
use crate::types::*;
use regex::Regex;
use lazy_static::lazy_static;
use std::time::{Duration, Instant};
use sysinfo::System;

/// Collect CPU information including usage and power metrics
pub async fn collect_cpu_info(
    system: &System,
    last_powermetrics: &mut Option<Instant>,
    cached_cpu_metrics: &mut Option<CPUMetrics>,
    powermetrics_cache_duration: Duration,
) -> Result<CpuInfo, Box<dyn std::error::Error>> {
    let cpu_usages: Vec<f32> = system
        .cpus()
        .iter()
        .map(|cpu| cpu.cpu_usage())
        .collect();

    let average_usage = if !cpu_usages.is_empty() {
        cpu_usages.iter().sum::<f32>() / cpu_usages.len() as f32
    } else {
        0.0
    };

    let power_metrics = if let Some(last_time) = last_powermetrics {
        if last_time.elapsed() < powermetrics_cache_duration {
            cached_cpu_metrics.clone().unwrap_or_default()
        } else {
            fetch_fresh_powermetrics(system, last_powermetrics, cached_cpu_metrics).await?
        }
    } else {
        fetch_fresh_powermetrics(system, last_powermetrics, cached_cpu_metrics).await?
    };

    Ok(CpuInfo {
        core_usages: cpu_usages,
        average_usage,
        power_metrics,
    })
}

/// Fetch fresh powermetrics data from the system
async fn fetch_fresh_powermetrics(
    system: &System,
    last_powermetrics: &mut Option<Instant>,
    cached_cpu_metrics: &mut Option<CPUMetrics>,
) -> Result<CPUMetrics, Box<dyn std::error::Error>> {
    match get_powermetrics_output().await {
        Ok(output) => {
            let metrics = parse_cpu_metrics(output).await?;
            *cached_cpu_metrics = Some(metrics.clone());
            *last_powermetrics = Some(Instant::now());
            Ok(metrics)
        }
        Err(e) => {
            // If powermetrics fails, use fallback metrics
            log::warn!("Powermetrics failed, using fallback: {}", e);
            let fallback_metrics = get_fallback_cpu_metrics(system);
            *cached_cpu_metrics = Some(fallback_metrics.clone());
            *last_powermetrics = Some(Instant::now());
            Ok(fallback_metrics)
        }
    }
}

/// Get fallback CPU metrics based on real CPU usage
fn get_fallback_cpu_metrics(system: &System) -> CPUMetrics {
    // Use real CPU usage as basis for fallback metrics
    let avg_usage = system.cpus().iter().map(|cpu| cpu.cpu_usage()).sum::<f32>() / system.cpus().len() as f32;
    
    // Estimate based on actual CPU usage
    let estimated_power = (avg_usage / 100.0) * 15.0; // Scale with usage
    
    CPUMetrics {
        e_cluster_active: (avg_usage * 0.6) as i32, // E-cores typically more active
        p_cluster_active: (avg_usage * 0.4) as i32, // P-cores less active
        e_cluster_freq_mhz: if avg_usage > 50.0 { 2400 } else { 1800 },
        p_cluster_freq_mhz: if avg_usage > 70.0 { 3200 } else { 2400 },
        ane_w: (estimated_power * 0.05) as f64,
        cpu_w: (estimated_power * 0.6) as f64,
        gpu_w: (estimated_power * 0.2) as f64,
        package_w: estimated_power as f64,
    }
}

/// Parse CPU metrics from powermetrics output
async fn parse_cpu_metrics(
    powermetrics_output: String,
) -> Result<CPUMetrics, Box<dyn std::error::Error>> {
    lazy_static! {
        static ref ACTIVE_RESIDENCY_REGEX: Regex = Regex::new(r"CPU (\d+) active residency:\s+(\d+\.\d+)%").unwrap();
        static ref FREQUENCY_REGEX: Regex = Regex::new(r"CPU\s+(\d+)\s+frequency:\s+(\d+)\s+MHz").unwrap();
    }

    let lines: Vec<&str> = powermetrics_output.lines().collect();
    let mut cpu_metrics = CPUMetrics::default();

    let mut e_cluster_active_sum = 0.0;
    let mut p_cluster_active_sum = 0.0;
    let mut e_cluster_freq_sum = 0.0;
    let mut p_cluster_freq_sum = 0.0;
    let mut e_cluster_count = 0;
    let mut p_cluster_count = 0;
    let mut e_cluster_freq_count = 0;
    let mut p_cluster_freq_count = 0;

    for line in &lines {
        if let Some(caps) = ACTIVE_RESIDENCY_REGEX.captures(line) {
            if let (Ok(core_id), Ok(active_residency)) = (caps[1].parse::<usize>(), caps[2].parse::<f64>()) {
                if core_id <= 3 {
                    e_cluster_active_sum += active_residency;
                    e_cluster_count += 1;
                } else {
                    p_cluster_active_sum += active_residency;
                    p_cluster_count += 1;
                }
            }
        }

        if let Some(caps) = FREQUENCY_REGEX.captures(line) {
            if let (Ok(core_id), Ok(active_freq)) = (caps[1].parse::<usize>(), caps[2].parse::<f64>()) {
                if core_id <= 3 {
                    e_cluster_freq_sum += active_freq;
                    e_cluster_freq_count += 1;
                } else {
                    p_cluster_freq_sum += active_freq;
                    p_cluster_freq_count += 1;
                }
            }
        }

        if line.contains("ANE Power") {
            if let Some(power_str) = line.split_whitespace().nth(2) {
                if let Ok(power) = power_str.trim_end_matches("mW").parse::<f64>() {
                    cpu_metrics.ane_w = power / 1000.0;
                }
            }
        } else if line.contains("CPU Power") {
            if let Some(power_str) = line.split_whitespace().nth(2) {
                if let Ok(power) = power_str.trim_end_matches("mW").parse::<f64>() {
                    cpu_metrics.cpu_w = power / 1000.0;
                }
            }
        } else if line.contains("GPU Power") {
            if let Some(power_str) = line.split_whitespace().nth(2) {
                if let Ok(power) = power_str.trim_end_matches("mW").parse::<f64>() {
                    cpu_metrics.gpu_w = power / 1000.0;
                }
            }
        } else if line.contains("Combined Power (CPU + GPU + ANE)") {
            if let Some(power_str) = line.split_whitespace().nth(7) {
                if let Ok(power) = power_str.trim_end_matches("mW").parse::<f64>() {
                    cpu_metrics.package_w = power / 1000.0;
                }
            }
        }
    }

    if e_cluster_count > 0 {
        cpu_metrics.e_cluster_active = (e_cluster_active_sum / e_cluster_count as f64) as i32;
    }
    if e_cluster_freq_count > 0 {
        cpu_metrics.e_cluster_freq_mhz = (e_cluster_freq_sum / e_cluster_freq_count as f64) as i32;
    }

    if p_cluster_count > 0 {
        cpu_metrics.p_cluster_active = (p_cluster_active_sum / p_cluster_count as f64) as i32;
    }
    if p_cluster_freq_count > 0 {
        cpu_metrics.p_cluster_freq_mhz = (p_cluster_freq_sum / p_cluster_freq_count as f64) as i32;
    }

    Ok(cpu_metrics)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

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
    async fn test_parse_cpu_metrics_empty_input() {
        let output = String::new();
        let result = parse_cpu_metrics(output).await;
        assert!(result.is_ok());

        let metrics = result.unwrap();
        assert_eq!(metrics.e_cluster_active, 0);
        assert_eq!(metrics.p_cluster_active, 0);
    }

    #[tokio::test]
    async fn test_parse_cpu_metrics_with_active_residency() {
        let output = r#"
CPU 0 active residency: 50.5%
CPU 1 active residency: 60.3%
CPU 2 active residency: 70.1%
CPU 3 active residency: 80.2%
CPU 4 active residency: 90.5%
CPU 5 active residency: 95.2%
"#;
        let result = parse_cpu_metrics(output.to_string()).await;
        assert!(result.is_ok());

        let metrics = result.unwrap();
        // E-cores (0-3) average: (50.5 + 60.3 + 70.1 + 80.2) / 4 = 65.275
        assert_eq!(metrics.e_cluster_active, 65);
        // P-cores (4-5) average: (90.5 + 95.2) / 2 = 92.85
        assert_eq!(metrics.p_cluster_active, 92);
    }

    #[tokio::test]
    async fn test_parse_cpu_metrics_with_frequency() {
        let output = "CPU 0 frequency: 1800 MHz\nCPU 1 frequency: 2000 MHz\nCPU 2 frequency: 2200 MHz\nCPU 3 frequency: 2400 MHz\nCPU 4 frequency: 3000 MHz\nCPU 5 frequency: 3200 MHz\n";
        let result = parse_cpu_metrics(output.to_string()).await;
        assert!(result.is_ok());

        let metrics = result.unwrap();
        // E-cores (0-3) average: (1800 + 2000 + 2200 + 2400) / 4 = 2100
        assert_eq!(metrics.e_cluster_freq_mhz, 2100);
        // P-cores (4-5) average: (3000 + 3200) / 2 = 3100
        assert_eq!(metrics.p_cluster_freq_mhz, 3100);
    }

    #[test]
    fn test_frequency_regex_directly() {
        use regex::Regex;
        let re = Regex::new(r"CPU\s+(\d+)\s+frequency:\s+(\d+)\s+MHz").unwrap();
        let line = "CPU 0 frequency: 1800 MHz";
        let caps = re.captures(line);
        assert!(caps.is_some(), "Regex should match '{}'", line);
        let caps = caps.unwrap();
        assert_eq!(&caps[1], "0");
        assert_eq!(&caps[2], "1800");
    }

    #[tokio::test]
    async fn test_parse_cpu_metrics_with_frequency_no_colon() {
        let output = "CPU 0 frequency 1800 MHz\nCPU 1 frequency 2000 MHz\nCPU 2 frequency 2200 MHz\nCPU 3 frequency 2400 MHz\nCPU 4 frequency 3000 MHz\nCPU 5 frequency 3200 MHz\n";
        let result = parse_cpu_metrics(output.to_string()).await;
        assert!(result.is_ok());

        let metrics = result.unwrap();
        // Without colon, regex won't match, so frequencies should be 0
        assert_eq!(metrics.e_cluster_freq_mhz, 0);
        assert_eq!(metrics.p_cluster_freq_mhz, 0);
    }

    #[tokio::test]
    async fn test_parse_cpu_metrics_with_power_data() {
        let output = r#"
ANE Power: 150mW
CPU Power: 5000mW
GPU Power: 2000mW
Combined Power (CPU + GPU + ANE): 7150mW
"#;
        let result = parse_cpu_metrics(output.to_string()).await;
        assert!(result.is_ok());

        let metrics = result.unwrap();
        assert_eq!(metrics.ane_w, 0.15);
        assert_eq!(metrics.cpu_w, 5.0);
        assert_eq!(metrics.gpu_w, 2.0);
        assert_eq!(metrics.package_w, 7.15);
    }

    #[tokio::test]
    async fn test_parse_cpu_metrics_mixed_data() {
        let output = r#"
CPU 0 active residency: 45.2%
CPU 1 active residency: 55.8%
CPU 2 active residency: 65.3%
CPU 3 active residency: 75.1%
CPU 4 active residency: 85.6%
CPU 5 active residency: 92.4%
CPU 0 frequency: 1800 MHz
CPU 1 frequency: 2000 MHz
CPU 2 frequency: 2200 MHz
CPU 3 frequency: 2400 MHz
CPU 4 frequency: 3000 MHz
CPU 5 frequency: 3200 MHz
ANE Power: 100mW
CPU Power: 4500mW
GPU Power: 1500mW
Combined Power (CPU + GPU + ANE): 6100mW
"#;
        let result = parse_cpu_metrics(output.to_string()).await;
        assert!(result.is_ok());

        let metrics = result.unwrap();
        // E-cores active: (45.2 + 55.8 + 65.3 + 75.1) / 4 = 60.35
        assert_eq!(metrics.e_cluster_active, 60);
        // P-cores active: (85.6 + 92.4) / 2 = 89.0
        assert_eq!(metrics.p_cluster_active, 89);
        // E-cores freq: (1800 + 2000 + 2200 + 2400) / 4 = 2100
        assert_eq!(metrics.e_cluster_freq_mhz, 2100);
        // P-cores freq: (3000 + 3200) / 2 = 3100
        assert_eq!(metrics.p_cluster_freq_mhz, 3100);
        assert_eq!(metrics.ane_w, 0.1);
        assert_eq!(metrics.cpu_w, 4.5);
        assert_eq!(metrics.gpu_w, 1.5);
        assert_eq!(metrics.package_w, 6.1);
    }

    #[tokio::test]
    async fn test_parse_cpu_metrics_invalid_data() {
        let output = r#"
This is not valid powermetrics output
Some random text
Another line
"#;
        let result = parse_cpu_metrics(output.to_string()).await;
        assert!(result.is_ok());

        // Should return default metrics
        let metrics = result.unwrap();
        assert_eq!(metrics.e_cluster_active, 0);
        assert_eq!(metrics.p_cluster_active, 0);
    }

    #[tokio::test]
    async fn test_parse_cpu_metrics_partial_data() {
        let output = r#"
CPU 0 active residency: 50.0%
CPU Power: 3000mW
"#;
        let result = parse_cpu_metrics(output.to_string()).await;
        assert!(result.is_ok());

        let metrics = result.unwrap();
        // Only one E-core, so average is 50
        assert_eq!(metrics.e_cluster_active, 50);
        assert_eq!(metrics.cpu_w, 3.0);
        // P-cores should be 0
        assert_eq!(metrics.p_cluster_active, 0);
    }

    #[test]
    fn test_get_fallback_cpu_metrics() {
        let system = System::new_all();
        let metrics = get_fallback_cpu_metrics(&system);

        // Fallback metrics should have reasonable values
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
//! Powermetrics data collection backend
//!
//! Handles running `powermetrics` (macOS) and parsing its output for
//! CPU/GPU/ANE/DRAM/package power, CPU residency, and frequency data.
//! Uses caching to avoid spawning the process on every refresh cycle.

use crate::types::CPUMetrics;
use lazy_static::lazy_static;
use regex::Regex;
use std::time::{Duration, Instant};
use tokio::process::Command;
use tokio::time::timeout;

/// Cached state for powermetrics sampling
#[derive(Clone)]
pub struct PowermetricsSampler {
    pub last_sample: Option<Instant>,
    pub cached_metrics: Option<CPUMetrics>,
    pub cached_output: Option<String>,
    pub cache_duration: Duration,
}

impl PowermetricsSampler {
    pub fn new(cache_duration: Duration) -> Self {
        Self {
            last_sample: None,
            cached_metrics: None,
            cached_output: None,
            cache_duration,
        }
    }

    /// Returns cached metrics if still fresh, otherwise runs powermetrics
    pub async fn sample(&mut self) -> Result<CPUMetrics, Box<dyn std::error::Error>> {
        if let Some(t) = self.last_sample {
            if t.elapsed() < self.cache_duration {
                // Cache hit
                return Ok(self.cached_metrics.clone().unwrap_or_default());
            }
        }
        // Cache miss — fetch fresh data
        match fetch_powermetrics_output().await {
            Ok(output) => {
                let metrics = parse_powermetrics_output(&output)?;
                self.cached_metrics = Some(metrics.clone());
                self.cached_output = Some(output);
                self.last_sample = Some(Instant::now());
                Ok(metrics)
            }
            Err(e) => {
                // On error, return stale cache if available
                if let Some(stale) = self.cached_metrics.clone() {
                    Ok(stale)
                } else {
                    Err(e)
                }
            }
        }
    }

    pub fn get_cached_output(&self) -> Option<&str> {
        self.cached_output.as_deref()
    }
}

/// Run `powermetrics -n 1` with a 5-second timeout
async fn fetch_powermetrics_output() -> Result<String, Box<dyn std::error::Error>> {
    let cmd = Command::new("powermetrics")
        .arg("-n")
        .arg("1")
        .arg("--samplers")
        .arg("cpu_power,gpu_power")
        .output();

    match timeout(Duration::from_secs(5), cmd).await {
        Ok(Ok(output)) => {
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(format!("powermetrics failed: {}", stderr).into());
            }
            Ok(String::from_utf8_lossy(&output.stdout).into_owned())
        }
        Ok(Err(e)) => Err(format!("powermetrics exec error: {}", e).into()),
        Err(_) => Err("powermetrics timed out after 5s".into()),
    }
}

/// Parse powermetrics output into CPUMetrics
pub fn parse_powermetrics_output(output: &str) -> Result<CPUMetrics, Box<dyn std::error::Error>> {
    lazy_static! {
        static ref RESIDENCY_RE: Regex =
            Regex::new(r"CPU (\d+) active residency:\s+(\d+\.\d+)%").unwrap();
        static ref FREQ_RE: Regex =
            Regex::new(r"CPU\s+(\d+)\s+frequency:\s+(\d+)\s+MHz").unwrap();
        static ref POWER_RE: Regex = Regex::new(
            r"^\s*(?P<comp>ANE|CPU|GPU|DRAM)\s+Power:\s+(?P<val>[\d.]+)\s*(?P<unit>mW|W)$"
        )
        .unwrap();
        static ref COMBINED_POWER_RE: Regex = Regex::new(
            r"^\s*Combined\s+Power\s+\(CPU\s+\+\s+GPU\s+\+\s+ANE\):\s+(?P<val>[\d.]+)\s*(?P<unit>mW|W)$"
        )
        .unwrap();
    }

    let mut m = CPUMetrics::default();
    let mut e_active_sum = 0.0;
    let mut p_active_sum = 0.0;
    let mut e_freq_sum = 0.0;
    let mut p_freq_sum = 0.0;
    let mut e_active_n = 0;
    let mut p_active_n = 0;
    let mut e_freq_n = 0;
    let mut p_freq_n = 0;

    for line in output.lines() {
        if let Some(caps) = RESIDENCY_RE.captures(line) {
            let id: usize = caps[1].parse().unwrap_or(99);
            let val: f64 = caps[2].parse().unwrap_or(0.0);
            if id <= 3 {
                e_active_sum += val;
                e_active_n += 1;
            } else {
                p_active_sum += val;
                p_active_n += 1;
            }
        }
        if let Some(caps) = FREQ_RE.captures(line) {
            let id: usize = caps[1].parse().unwrap_or(99);
            let val: f64 = caps[2].parse().unwrap_or(0.0);
            if id <= 3 {
                e_freq_sum += val;
                e_freq_n += 1;
            } else {
                p_freq_sum += val;
                p_freq_n += 1;
            }
        }
        if let Some(caps) = POWER_RE.captures(line) {
            let val: f64 = caps["val"].parse().unwrap_or(0.0);
            let w = match &caps["unit"] {
                "mW" => val / 1000.0,
                _ => val,
            };
            match &caps["comp"] {
                "ANE" => m.ane_w = w,
                "CPU" => m.cpu_w = w,
                "GPU" => m.gpu_w = w,
                "DRAM" => m.dram_w = w,
                _ => {}
            }
        }
        if let Some(caps) = COMBINED_POWER_RE.captures(line) {
            let val: f64 = caps["val"].parse().unwrap_or(0.0);
            m.package_w = match &caps["unit"] {
                "mW" => val / 1000.0,
                _ => val,
            };
        }
    }

    if e_active_n > 0 {
        m.e_cluster_active = (e_active_sum / e_active_n as f64) as i32;
    }
    if e_freq_n > 0 {
        m.e_cluster_freq_mhz = (e_freq_sum / e_freq_n as f64) as i32;
    }
    if p_active_n > 0 {
        m.p_cluster_active = (p_active_sum / p_active_n as f64) as i32;
    }
    if p_freq_n > 0 {
        m.p_cluster_freq_mhz = (p_freq_sum / p_freq_n as f64) as i32;
    }

    Ok(m)
}

/// Extract DRAM power in watts from raw powermetrics output
pub fn extract_dram_power(output: &str) -> Option<f64> {
    lazy_static! {
        static ref RE: Regex =
            Regex::new(r"^\s*DRAM\s+Power:\s+(?P<val>[\d.]+)\s*(?P<unit>mW|W)$").unwrap();
    }
    for line in output.lines() {
        if let Some(caps) = RE.captures(line) {
            let val: f64 = caps["val"].parse().ok()?;
            return Some(match &caps["unit"] {
                "mW" => val / 1000.0,
                _ => val,
            });
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parse_empty() {
        let m = parse_powermetrics_output("").unwrap();
        assert_eq!(m.e_cluster_active, 0);
        assert_eq!(m.p_cluster_active, 0);
    }

    #[tokio::test]
    async fn test_parse_residency() {
        let out = "CPU 0 active residency: 50.5%\nCPU 3 active residency: 60.3%\nCPU 4 active residency: 90.5%\nCPU 5 active residency: 95.2%\n";
        let m = parse_powermetrics_output(out).unwrap();
        assert_eq!(m.e_cluster_active, 55); // (50.5+60.3)/2
        assert_eq!(m.p_cluster_active, 92); // (90.5+95.2)/2
    }

    #[tokio::test]
    async fn test_parse_power_mw() {
        let out = "ANE Power: 150mW\nCPU Power: 5000mW\nGPU Power: 2000mW\nDRAM Power: 800mW\nCombined Power (CPU + GPU + ANE): 7150mW\n";
        let m = parse_powermetrics_output(out).unwrap();
        assert!((m.ane_w - 0.15).abs() < 1e-10);
        assert!((m.cpu_w - 5.0).abs() < 1e-10);
        assert!((m.gpu_w - 2.0).abs() < 1e-10);
        assert!((m.dram_w - 0.8).abs() < 1e-10);
        assert!((m.package_w - 7.15).abs() < 1e-10);
    }

    #[tokio::test]
    async fn test_parse_power_w() {
        let out = "ANE Power: 0.15W\nCPU Power: 5.0W\nGPU Power: 2.0W\nDRAM Power: 0.8W\nCombined Power (CPU + GPU + ANE): 7.15W\n";
        let m = parse_powermetrics_output(out).unwrap();
        assert!((m.ane_w - 0.15).abs() < 1e-10);
        assert!((m.cpu_w - 5.0).abs() < 1e-10);
        assert!((m.gpu_w - 2.0).abs() < 1e-10);
        assert!((m.dram_w - 0.8).abs() < 1e-10);
        assert!((m.package_w - 7.15).abs() < 1e-10);
    }

    #[test]
    fn test_extract_dram_mw() {
        let out = "DRAM Power: 800mW\n";
        assert!((extract_dram_power(out).unwrap() - 0.8).abs() < 1e-10);
    }

    #[test]
    fn test_extract_dram_w() {
        let out = "DRAM Power: 0.8W\n";
        assert!((extract_dram_power(out).unwrap() - 0.8).abs() < 1e-10);
    }

    #[test]
    fn test_extract_dram_none() {
        assert!(extract_dram_power("no DRAM line").is_none());
    }
}

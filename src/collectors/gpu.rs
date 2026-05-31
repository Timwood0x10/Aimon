//! GPU data collector
//! Collects GPU usage, frequency, and core information using IOReport/IOKit

use crate::cli::get_powermetrics_output;
use crate::types::GpuInfo;
use lazy_static::lazy_static;
use regex::Regex;

/// Collect GPU information from powermetrics and system_profiler
pub async fn collect_gpu_info() -> GpuInfo {
    let mut gpu_info = GpuInfo::default();

    // Try to get GPU info from powermetrics
    match get_powermetrics_output().await {
        Ok(output) => {
            parse_gpu_from_powermetrics(&output, &mut gpu_info);
        }
        Err(_) => {
            // Fallback: try system_profiler
            if let Ok(output) = get_system_profiler_gpu().await {
                parse_gpu_from_profiler(&output, &mut gpu_info);
            }
        }
    }

    // Detect GPU core count from chip model
    if gpu_info.core_count == 0 {
        gpu_info.core_count = detect_gpu_core_count();
    }

    // Calculate TFLOPs
    if gpu_info.freq_mhz > 0 && gpu_info.core_count > 0 {
        // Approximate: 2 FLOPs per clock per GPU core (FP32)
        gpu_info.tflops =
            (gpu_info.core_count as f64 * gpu_info.freq_mhz as f64 * 1e6 * 2.0) / 1e12;
    }

    gpu_info
}

/// Parse GPU metrics from powermetrics output
fn parse_gpu_from_powermetrics(output: &str, gpu_info: &mut GpuInfo) {
    lazy_static! {
        static ref GPU_FREQ_REGEX: Regex = Regex::new(r"GPU\s+frequency:\s+(\d+)\s+MHz").unwrap();
        static ref GPU_ACTIVE_REGEX: Regex =
            Regex::new(r"GPU\s+active\s+residency:\s+(\d+\.?\d*)%").unwrap();
        static ref GPU_SRAM_REGEX: Regex =
            Regex::new(r"GPU SRAM Power:\s+(\d+\.?\d*)\s*mW").unwrap();
    }

    for line in output.lines() {
        if let Some(caps) = GPU_FREQ_REGEX.captures(line) {
            if let Ok(freq) = caps[1].parse::<i32>() {
                gpu_info.freq_mhz = freq;
                if gpu_info.max_freq_mhz == 0 || freq > gpu_info.max_freq_mhz {
                    gpu_info.max_freq_mhz = freq;
                }
            }
        }

        if let Some(caps) = GPU_ACTIVE_REGEX.captures(line) {
            if let Ok(usage) = caps[1].parse::<f32>() {
                gpu_info.usage_percentage = usage;
            }
        }

        if let Some(caps) = GPU_SRAM_REGEX.captures(line) {
            if let Ok(power) = caps[1].parse::<f64>() {
                gpu_info.sram_power_w = power / 1000.0;
            }
        }

        // Also parse GPU frequency table entries
        if line.contains("GPU freq") && line.contains("MHz") {
            if let Some(freq_str) = line.split_whitespace().find(|s| s.ends_with("MHz")) {
                if let Ok(freq) = freq_str.trim_end_matches("MHz").parse::<i32>() {
                    if freq > gpu_info.max_freq_mhz {
                        gpu_info.max_freq_mhz = freq;
                    }
                }
            }
        }
    }
}

/// Parse GPU info from system_profiler
async fn get_system_profiler_gpu() -> Result<String, Box<dyn std::error::Error>> {
    let output = tokio::process::Command::new("system_profiler")
        .args(["SPDisplaysDataType"])
        .output()
        .await?;

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn parse_gpu_from_profiler(output: &str, gpu_info: &mut GpuInfo) {
    for line in output.lines() {
        if line.contains("Total Number of Cores") {
            if let Some(num_str) = line.split(':').nth(1) {
                if let Ok(count) = num_str.trim().parse::<usize>() {
                    gpu_info.core_count = count;
                }
            }
        }
        if line.contains("Chipset Model") && line.contains("Apple") {
            // Extract the Apple GPU model
        }
    }
}

/// Detect GPU core count based on the chip model string
fn detect_gpu_core_count() -> usize {
    // Try reading from sysctl or ioreg
    if let Ok(output) = std::process::Command::new("sysctl")
        .args(["-n", "machdep.cpu.brand_string"])
        .output()
    {
        let brand = String::from_utf8_lossy(&output.stdout);
        return guess_gpu_cores_from_brand(&brand);
    }
    0
}

/// Guess GPU core count from CPU brand string
fn guess_gpu_cores_from_brand(brand: &str) -> usize {
    let brand_lower = brand.to_lowercase();
    if brand_lower.contains("m1 ultra") {
        64
    } else if brand_lower.contains("m1 max") {
        32
    } else if brand_lower.contains("m1 pro") {
        16
    } else if brand_lower.contains("m2 ultra") {
        76
    } else if brand_lower.contains("m2 max") {
        38
    } else if brand_lower.contains("m2 pro") {
        19
    } else if brand_lower.contains("m3 ultra") {
        76
    } else if brand_lower.contains("m3 max") {
        40
    } else if brand_lower.contains("m3 pro") {
        18
    } else if brand_lower.contains("m4 max") {
        40
    } else if brand_lower.contains("m4 pro") {
        20
    } else if brand_lower.contains("m4") {
        10
    } else if brand_lower.contains("m2") {
        10
    } else if brand_lower.contains("m1") {
        8
    } else {
        8 // Default fallback
    }
}

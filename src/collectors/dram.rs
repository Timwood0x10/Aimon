//! DRAM bandwidth data collector
//! Uses powermetrics for DRAM power estimation

use crate::cli::get_powermetrics_output;
use crate::types::DramInfo;

/// Collect DRAM bandwidth information
/// Note: Without sudo, we can only estimate from DRAM power
pub async fn collect_dram_info() -> DramInfo {
    let mut dram_info = DramInfo::default();

    if let Ok(output) = get_powermetrics_output().await {
        for line in output.lines() {
            if line.contains("DRAM Power") || line.contains("DRAM power") {
                if let Some(power_str) = line.split_whitespace().nth(2) {
                    if let Ok(power) = power_str.trim_end_matches("mW").parse::<f64>() {
                        dram_info.power_w = power / 1000.0;
                    }
                }
            }
        }
    }

    dram_info
}

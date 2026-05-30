//! Data formatters for various output formats
//! Converts SystemData into JSON, CSV, and Prometheus text formats

use crate::types::SystemData;
use serde::Serialize;

/// Wrapper that adds a wall-clock timestamp to SystemData for serialization
#[derive(Serialize)]
struct TimestampedData<'a> {
    #[serde(flatten)]
    data: &'a SystemData,
    timestamp: String,
}

/// Format SystemData as pretty-printed JSON
pub fn format_json(data: &SystemData) -> String {
    let output = TimestampedData {
        data,
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    serde_json::to_string_pretty(&output).unwrap_or_else(|_| "{}".to_string())
}

/// Format SystemData as a compact JSON with key metrics only
pub fn format_compact_json(data: &SystemData) -> String {
    let compact = serde_json::json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "cpu": {
            "average_usage": data.cpu_info.average_usage,
            "e_cluster_active": data.cpu_info.power_metrics.e_cluster_active,
            "p_cluster_active": data.cpu_info.power_metrics.p_cluster_active,
        },
        "memory": {
            "total_bytes": data.memory_info.total_memory,
            "used_bytes": data.memory_info.used_memory,
            "usage_percent": data.memory_info.usage_percentage,
        },
        "battery": {
            "percent": data.battery_info.percentage,
            "is_charging": data.battery_info.is_charging,
            "health_percent": data.battery_info.health_percentage,
        },
        "power": {
            "cpu_watts": data.cpu_info.power_metrics.cpu_w,
            "gpu_watts": data.cpu_info.power_metrics.gpu_w,
            "package_watts": data.cpu_info.power_metrics.package_w,
        },
        "thermal": {
            "pressure": data.thermal_info.thermal_pressure,
            "throttling": data.thermal_info.thermal_throttling,
        },
        "uptime_seconds": data.system_health.uptime_seconds,
    });
    serde_json::to_string_pretty(&compact).unwrap_or_else(|_| "{}".to_string())
}

/// Format SystemData as CSV (one row of key metrics)
pub fn format_csv(data: &SystemData) -> String {
    let mut wtr = csv::Writer::from_writer(vec![]);

    // Write header
    let headers = csv_headers();
    wtr.write_record(&headers).ok();

    // Write data row
    let row = csv_values(data);
    wtr.write_record(&row).ok();

    String::from_utf8(wtr.into_inner().unwrap_or_default()).unwrap_or_default()
}

/// CSV column headers
fn csv_headers() -> Vec<&'static str> {
    vec![
        "timestamp",
        "hostname",
        "cpu_avg_usage",
        "cpu_e_cluster_active",
        "cpu_p_cluster_active",
        "memory_total_bytes",
        "memory_used_bytes",
        "memory_usage_percent",
        "battery_percent",
        "battery_is_charging",
        "battery_health_percent",
        "power_cpu_watts",
        "power_gpu_watts",
        "power_package_watts",
        "thermal_pressure",
        "thermal_throttling",
        "uptime_seconds",
        "network_rx_bytes",
        "network_tx_bytes",
    ]
}

/// CSV data values matching headers
fn csv_values(data: &SystemData) -> Vec<String> {
    let total_rx: u64 = data.network_info.iter().map(|n| n.bytes_received).sum();
    let total_tx: u64 = data.network_info.iter().map(|n| n.bytes_transmitted).sum();

    vec![
        chrono::Utc::now().to_rfc3339(),
        data.system_info.host_name.clone(),
        data.cpu_info.average_usage.to_string(),
        data.cpu_info.power_metrics.e_cluster_active.to_string(),
        data.cpu_info.power_metrics.p_cluster_active.to_string(),
        data.memory_info.total_memory.to_string(),
        data.memory_info.used_memory.to_string(),
        data.memory_info.usage_percentage.to_string(),
        data.battery_info.percentage.to_string(),
        data.battery_info.is_charging.to_string(),
        data.battery_info.health_percentage.to_string(),
        data.cpu_info.power_metrics.cpu_w.to_string(),
        data.cpu_info.power_metrics.gpu_w.to_string(),
        data.cpu_info.power_metrics.package_w.to_string(),
        data.thermal_info.thermal_pressure.to_string(),
        data.thermal_info.thermal_throttling.to_string(),
        data.system_health.uptime_seconds.to_string(),
        total_rx.to_string(),
        total_tx.to_string(),
    ]
}

/// Format SystemData as Prometheus exposition text
pub fn format_prometheus(data: &SystemData) -> String {
    crate::api::prometheus::PrometheusExporter::export(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use std::time::Instant;

    fn create_test_data() -> SystemData {
        SystemData {
            system_info: SystemInfo {
                name: "macOS".to_string(),
                kernel_version: "24.0.0".to_string(),
                os_version: "15.0".to_string(),
                host_name: "test-host".to_string(),
                cpu_arch: "arm64".to_string(),
                cpu_brand: "Apple M1".to_string(),
                cpu_core_count: 8,
                e_core_count: 4,
                p_core_count: 4,
                gpu_core_count: 8,
                chip_name: "Apple M1".to_string(),
            },
            cpu_info: CpuInfo {
                core_usages: vec![10.0, 20.0, 30.0],
                average_usage: 20.0,
                power_metrics: CPUMetrics {
                    e_cluster_active: 15,
                    p_cluster_active: 25,
                    e_cluster_freq_mhz: 600,
                    p_cluster_freq_mhz: 3200,
                    cpu_w: 1.5,
                    gpu_w: 0.5,
                    ane_w: 0.1,
                    dram_w: 0.3,
                    package_w: 2.1,
                },
            },
            gpu_info: GpuInfo::default(),
            ane_info: AneInfo::default(),
            dram_info: DramInfo::default(),
            thunderbolt_info: ThunderboltInfo::default(),
            disk_io_info: DiskIoInfo::default(),
            memory_info: MemoryInfo {
                total_memory: 16_000_000_000,
                used_memory: 8_000_000_000,
                available_memory: 8_000_000_000,
                total_swap: 0,
                used_swap: 0,
                usage_percentage: 50,
            },
            network_info: vec![NetworkInterface {
                name: "en0".to_string(),
                bytes_received: 1000,
                bytes_transmitted: 2000,
                packets_received: 10,
                packets_transmitted: 20,
            }],
            temperature_info: vec![TemperatureInfo {
                label: "CPU".to_string(),
                temperature: 45.0,
                critical_temperature: 100.0,
            }],
            process_info: vec![],
            battery_info: BatteryInfo {
                percentage: 85.0,
                is_charging: true,
                is_plugged: true,
                health_percentage: 95.0,
                cycle_count: 100,
                time_remaining: Some(120),
                power_adapter_wattage: 67.0,
                current_capacity: 4000,
                design_capacity: 5000,
                voltage: 12.0,
                amperage: 2.0,
                temperature: 30.0,
            },
            thermal_info: ThermalInfo {
                fans: vec![],
                thermal_state: ThermalState::Nominal,
                fan_speeds: vec![1200, 1300],
                thermal_throttling: false,
                heat_dissipation_rate: 5.0,
                thermal_pressure: 10,
            },
            performance_metrics: PerformanceMetrics::default(),
            system_health: SystemHealthInfo {
                uptime_seconds: 3600,
                sleep_wake_efficiency: 98.0,
                power_quality_score: 90,
                system_load_1min: 1.5,
                system_load_5min: 1.2,
                system_load_15min: 1.0,
            },
            timestamp: Instant::now(),
            terminal_info: TerminalInfo::default(),
        }
    }

    #[test]
    fn test_format_json_output() {
        let data = create_test_data();
        let json_str = format_json(&data);
        let parsed: serde_json::Value =
            serde_json::from_str(&json_str).expect("JSON output should be valid JSON");
        assert!(parsed.is_object());
        assert!(parsed.get("timestamp").is_some());
        assert!(parsed.get("cpu_info").is_some());
        assert!(parsed.get("memory_info").is_some());
    }

    #[test]
    fn test_format_csv_output() {
        let data = create_test_data();
        let csv_str = format_csv(&data);
        let lines: Vec<&str> = csv_str.trim().lines().collect();
        assert!(lines.len() >= 2, "CSV should have header + data row");
        // Verify header contains expected columns
        assert!(lines[0].contains("timestamp"));
        assert!(lines[0].contains("cpu_avg_usage"));
        assert!(lines[0].contains("memory_total_bytes"));
        assert!(lines[0].contains("battery_percent"));
    }

    #[test]
    fn test_format_compact_json() {
        let data = create_test_data();
        let json_str = format_compact_json(&data);
        let parsed: serde_json::Value =
            serde_json::from_str(&json_str).expect("Compact JSON should be valid JSON");
        assert!(parsed.get("cpu").is_some());
        assert!(parsed.get("memory").is_some());
        assert!(parsed.get("battery").is_some());
        assert!(parsed.get("power").is_some());
        assert!(parsed.get("thermal").is_some());
        assert!(parsed.get("uptime_seconds").is_some());
    }

    #[test]
    fn test_format_prometheus() {
        let data = create_test_data();
        let prom = format_prometheus(&data);
        assert!(
            prom.contains("# HELP"),
            "Prometheus output should contain HELP lines"
        );
        assert!(
            prom.contains("# TYPE"),
            "Prometheus output should contain TYPE lines"
        );
    }

    #[test]
    fn test_prometheus_metric_names() {
        let data = create_test_data();
        let prom = format_prometheus(&data);
        let expected_metrics = [
            "system_cpu_usage_percent",
            "system_memory_usage_percent",
            "system_memory_total_bytes",
            "system_battery_percent",
            "system_battery_health_percent",
            "system_power_watts",
            "system_temperature_celsius",
            "system_network_rx_bytes",
            "system_network_tx_bytes",
            "system_thermal_pressure",
            "system_uptime_seconds",
        ];
        for metric in &expected_metrics {
            assert!(
                prom.contains(metric),
                "Prometheus output missing metric: {}",
                metric
            );
        }
    }

    #[test]
    fn test_csv_column_count() {
        let data = create_test_data();
        let csv_str = format_csv(&data);
        let mut rdr = csv::Reader::from_reader(csv_str.as_bytes());
        let headers = rdr.headers().expect("CSV should have headers");
        let header_count = headers.len();

        for result in rdr.records() {
            let record = result.expect("CSV record should be valid");
            assert_eq!(
                record.len(),
                header_count,
                "Data row column count ({}) should match header count ({})",
                record.len(),
                header_count
            );
        }
    }
}

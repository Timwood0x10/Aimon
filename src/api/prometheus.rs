//! Prometheus metrics exporter
//! Formats SystemData into Prometheus exposition text format

use crate::types::SystemData;

/// Prometheus exporter for system metrics
pub struct PrometheusExporter;

impl PrometheusExporter {
    /// Export SystemData as Prometheus exposition text format
    pub fn export(data: &SystemData) -> String {
        let mut lines = Vec::new();

        // CPU usage
        Self::add_gauge(
            &mut lines,
            "system_cpu_usage_percent",
            "Overall CPU usage percentage",
            data.cpu_info.average_usage as f64,
        );

        // Memory metrics
        Self::add_gauge(
            &mut lines,
            "system_memory_usage_percent",
            "Memory usage percentage",
            data.memory_info.usage_percentage as f64,
        );
        Self::add_gauge(
            &mut lines,
            "system_memory_total_bytes",
            "Total system memory in bytes",
            data.memory_info.total_memory as f64,
        );

        // Battery metrics
        Self::add_gauge(
            &mut lines,
            "system_battery_percent",
            "Battery charge percentage",
            data.battery_info.percentage as f64,
        );
        Self::add_gauge(
            &mut lines,
            "system_battery_health_percent",
            "Battery health percentage",
            data.battery_info.health_percentage as f64,
        );

        // Power metrics with component labels
        Self::add_gauge_with_labels(
            &mut lines,
            "system_power_watts",
            "Power consumption in watts",
            &[
                ("component", "cpu"),
            ],
            data.cpu_info.power_metrics.cpu_w,
        );
        Self::add_gauge_with_labels(
            &mut lines,
            "system_power_watts",
            "Power consumption in watts",
            &[
                ("component", "gpu"),
            ],
            data.cpu_info.power_metrics.gpu_w,
        );
        Self::add_gauge_with_labels(
            &mut lines,
            "system_power_watts",
            "Power consumption in watts",
            &[
                ("component", "ane"),
            ],
            data.cpu_info.power_metrics.ane_w,
        );
        Self::add_gauge_with_labels(
            &mut lines,
            "system_power_watts",
            "Power consumption in watts",
            &[
                ("component", "package"),
            ],
            data.cpu_info.power_metrics.package_w,
        );

        // Temperature metrics with sensor labels
        for temp in &data.temperature_info {
            Self::add_gauge_with_labels(
                &mut lines,
                "system_temperature_celsius",
                "Temperature sensor reading in Celsius",
                &[("sensor", &temp.label)],
                temp.temperature as f64,
            );
        }

        // Network metrics
        let total_rx: u64 = data.network_info.iter().map(|n| n.bytes_received).sum();
        let total_tx: u64 = data.network_info.iter().map(|n| n.bytes_transmitted).sum();

        Self::add_gauge(
            &mut lines,
            "system_network_rx_bytes",
            "Total bytes received across all interfaces",
            total_rx as f64,
        );
        Self::add_gauge(
            &mut lines,
            "system_network_tx_bytes",
            "Total bytes transmitted across all interfaces",
            total_tx as f64,
        );

        // Thermal pressure
        Self::add_gauge(
            &mut lines,
            "system_thermal_pressure",
            "Thermal pressure level (0-100)",
            data.thermal_info.thermal_pressure as f64,
        );

        // Uptime
        Self::add_gauge(
            &mut lines,
            "system_uptime_seconds",
            "System uptime in seconds",
            data.system_health.uptime_seconds as f64,
        );

        lines.join("\n")
    }

    /// Add a simple gauge metric line
    fn add_gauge(lines: &mut Vec<String>, name: &str, help: &str, value: f64) {
        lines.push(format!("# HELP {} {}", name, help));
        lines.push(format!("# TYPE {} gauge", name));
        lines.push(format!("{} {}", name, value));
    }

    /// Add a gauge metric with labels
    fn add_gauge_with_labels(
        lines: &mut Vec<String>,
        name: &str,
        help: &str,
        labels: &[(&str, &str)],
        value: f64,
    ) {
        // Only add HELP/TYPE once per metric name - check if already added
        let help_key = format!("# HELP {}", name);
        if !lines.iter().any(|l| l.starts_with(&help_key)) {
            lines.push(format!("# HELP {} {}", name, help));
            lines.push(format!("# TYPE {} gauge", name));
        }

        let label_str = labels
            .iter()
            .map(|(k, v)| format!("{}=\"{}\"", k, v))
            .collect::<Vec<_>>()
            .join(",");
        lines.push(format!("{}{{{}}} {}", name, label_str, value));
    }
}

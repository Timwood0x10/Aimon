//! Headless exporter for one-shot and continuous metric output
//! Supports JSON, CSV, and Prometheus output formats

use crate::api::formatters::{format_compact_json, format_csv, format_json, format_prometheus};
use crate::collectors::DataCollector;
use std::time::Duration;
use tokio::io::AsyncWriteExt;

/// Supported output formats for headless export
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Json,
    Csv,
    Prometheus,
}

impl OutputFormat {
    /// Parse a format string (case-insensitive)
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "json" => Some(Self::Json),
            "csv" => Some(Self::Csv),
            "prometheus" | "prom" => Some(Self::Prometheus),
            _ => None,
        }
    }
}

/// Headless exporter for one-shot and continuous metric output
pub struct HeadlessExporter {
    collector: DataCollector,
}

impl HeadlessExporter {
    /// Create a new headless exporter
    pub fn new() -> Self {
        Self {
            collector: DataCollector::new_fast(),
        }
    }

    /// Collect data and output as JSON once
    pub async fn export_json(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        let data = self.collector.collect_all_data().await?;
        Ok(format_json(&data))
    }

    /// Collect data and output as CSV once
    pub async fn export_csv(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        let data = self.collector.collect_all_data().await?;
        Ok(format_csv(&data))
    }

    /// Collect data and output as compact JSON once
    pub async fn export_compact_json(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        let data = self.collector.collect_all_data().await?;
        Ok(format_compact_json(&data))
    }

    /// Collect data and output as Prometheus text once
    pub async fn export_prometheus(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        let data = self.collector.collect_all_data().await?;
        Ok(format_prometheus(&data))
    }

    /// Run continuous output stream at the given interval in the specified format.
    /// Writes to stdout until interrupted.
    pub async fn export_stream(
        interval_secs: u64,
        format: OutputFormat,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut exporter = Self::new();
        let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));
        let mut stdout = tokio::io::stdout();

        // For CSV, emit the header once at the start
        if format == OutputFormat::Csv {
            let data = exporter.collector.collect_all_data().await?;
            let header = format_csv(&data);
            // Write only the header line (first line)
            if let Some(first_line) = header.lines().next() {
                stdout.write_all(first_line.as_bytes()).await?;
                stdout.write_all(b"\n").await?;
                stdout.flush().await?;
            }
        }

        loop {
            interval.tick().await;

            let output = match format {
                OutputFormat::Json => exporter.export_json().await,
                OutputFormat::Csv => {
                    // For streaming CSV, output data rows without the header
                    let data = exporter.collector.collect_all_data().await?;
                    let csv = format_csv(&data);
                    // Skip the header line, output only the data row
                    Ok(csv
                        .lines()
                        .skip(1)
                        .collect::<Vec<_>>()
                        .join("\n"))
                }
                OutputFormat::Prometheus => exporter.export_prometheus().await,
            };

            match output {
                Ok(text) => {
                    stdout.write_all(text.as_bytes()).await?;
                    stdout.write_all(b"\n").await?;
                    stdout.flush().await?;
                }
                Err(e) => {
                    log::error!("Export error: {}", e);
                }
            }
        }
    }
}

impl Default for HeadlessExporter {
    fn default() -> Self {
        Self::new()
    }
}

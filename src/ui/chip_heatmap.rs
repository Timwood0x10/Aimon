//! ASCII chip visualization showing Apple Silicon die layout with temperature overlay
//! Maps temperature sensors to die regions and color-codes by temperature

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::theme::Theme;
use crate::types::SystemData;

/// Chip heatmap visualization for Apple Silicon die
pub struct ChipHeatmap;

/// Die regions on an Apple Silicon chip
#[derive(Debug, Clone, Copy, PartialEq)]
enum DieRegion {
    ECore,
    PCore,
    Gpu,
    Ane,
    Dram,
}

/// Get color for a given temperature in Celsius
fn temperature_color(temp: f32) -> Color {
    if temp < 40.0 {
        Color::Rgb(50, 100, 255) // Blue - cool
    } else if temp < 60.0 {
        Color::Rgb(50, 200, 50) // Green - warm
    } else if temp < 80.0 {
        Color::Rgb(255, 220, 50) // Yellow - hot
    } else if temp < 90.0 {
        Color::Rgb(255, 140, 30) // Orange - very hot
    } else {
        Color::Rgb(255, 40, 40) // Red - critical
    }
}

/// Map a sensor label to a die region
fn map_sensor_to_region(label: &str) -> Option<DieRegion> {
    let lower = label.to_lowercase();
    if lower.contains("e-core") || lower.contains("ecore") || lower.contains("efficiency") {
        Some(DieRegion::ECore)
    } else if lower.contains("p-core") || lower.contains("pcore") || lower.contains("performance") {
        Some(DieRegion::PCore)
    } else if lower.contains("gpu") || lower.contains("graphics") {
        Some(DieRegion::Gpu)
    } else if lower.contains("ane") || lower.contains("neural") {
        Some(DieRegion::Ane)
    } else if lower.contains("dram") || lower.contains("memory") || lower.contains("ram") {
        Some(DieRegion::Dram)
    } else {
        None
    }
}

/// Compute average temperature for a set of sensors matching a region
fn region_temperature(data: &SystemData, region: DieRegion) -> Option<f32> {
    let matching: Vec<f32> = data
        .temperature_info
        .iter()
        .filter(|t| map_sensor_to_region(&t.label) == Some(region))
        .map(|t| t.temperature)
        .collect();

    if matching.is_empty() {
        None
    } else {
        Some(matching.iter().sum::<f32>() / matching.len() as f32)
    }
}

/// Get average temperature across all sensors
fn average_temperature(data: &SystemData) -> Option<f32> {
    if data.temperature_info.is_empty() {
        return None;
    }
    let sum: f32 = data.temperature_info.iter().map(|t| t.temperature).sum();
    Some(sum / data.temperature_info.len() as f32)
}

/// Build a colored block span for a region label
fn region_block(label: &str, temp: Option<f32>, width: usize) -> Span<'static> {
    match temp {
        Some(t) => {
            let color = temperature_color(t);
            let text = format!("{:<width$}", format!("{} {:.0}C", label, t), width = width);
            Span::styled(
                text,
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            )
        }
        None => {
            let text = format!("{:<width$}", format!("{} --", label), width = width);
            Span::styled(text, Style::default().fg(Color::DarkGray))
        }
    }
}

impl ChipHeatmap {
    /// Render the chip heatmap visualization
    pub fn render(f: &mut Frame, area: Rect, data: &SystemData, theme: &Theme) {
        let ecore_temp = region_temperature(data, DieRegion::ECore);
        let pcore_temp = region_temperature(data, DieRegion::PCore);
        let gpu_temp = region_temperature(data, DieRegion::Gpu);
        let ane_temp = region_temperature(data, DieRegion::Ane);
        let dram_temp = region_temperature(data, DieRegion::Dram);
        let avg_temp = average_temperature(data);

        let mut lines: Vec<Line> = vec![
            Line::from(Span::styled(
                "  +------------------------------------------+",
                Style::default().fg(theme.accent),
            )),
            Line::from(Span::styled(
                "  |          APPLE SILICON DIE MAP            |",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                "  +------------------------------------------+",
                Style::default().fg(theme.accent),
            )),
            Line::from(vec![
                Span::styled("  | ", Style::default().fg(theme.accent)),
                region_block("E-CLUSTER", ecore_temp, 36),
                Span::styled(" |", Style::default().fg(theme.accent)),
            ]),
            Line::from(vec![
                Span::styled("  | ", Style::default().fg(theme.accent)),
                region_block("P-CLUSTER", pcore_temp, 36),
                Span::styled(" |", Style::default().fg(theme.accent)),
            ]),
            Line::from(vec![
                Span::styled("  | ", Style::default().fg(theme.accent)),
                region_block("GPU", gpu_temp, 36),
                Span::styled(" |", Style::default().fg(theme.accent)),
            ]),
            Line::from(vec![
                Span::styled("  | ", Style::default().fg(theme.accent)),
                region_block("ANE (Neural Engine)", ane_temp, 36),
                Span::styled(" |", Style::default().fg(theme.accent)),
            ]),
            Line::from(vec![
                Span::styled("  | ", Style::default().fg(theme.accent)),
                region_block("DRAM", dram_temp, 36),
                Span::styled(" |", Style::default().fg(theme.accent)),
            ]),
            Line::from(Span::styled(
                "  +------------------------------------------+",
                Style::default().fg(theme.accent),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Legend: ", Style::default().fg(Color::DarkGray)),
                Span::styled("<40C ", Style::default().fg(Color::Rgb(50, 100, 255))),
                Span::styled("40-60C ", Style::default().fg(Color::Rgb(50, 200, 50))),
                Span::styled("60-80C ", Style::default().fg(Color::Rgb(255, 220, 50))),
                Span::styled("80-90C ", Style::default().fg(Color::Rgb(255, 140, 30))),
                Span::styled(">90C", Style::default().fg(Color::Rgb(255, 40, 40))),
            ]),
        ];

        // Average temp or no data message
        match avg_temp {
            Some(t) => {
                let color = temperature_color(t);
                lines.push(Line::from(vec![
                    Span::styled("  Avg: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("{:.1}C", t),
                        Style::default().fg(color).add_modifier(Modifier::BOLD),
                    ),
                ]));
            }
            None => {
                lines.push(Line::from(Span::styled(
                    "  No sensor data available",
                    Style::default().fg(Color::DarkGray),
                )));
            }
        }

        let block = Paragraph::new(lines).block(
            Block::default()
                .title(" CHIP HEATMAP ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.accent)),
        );

        f.render_widget(block, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temperature_color_mapping() {
        // Blue for < 40
        assert_eq!(temperature_color(20.0), Color::Rgb(50, 100, 255));
        assert_eq!(temperature_color(39.9), Color::Rgb(50, 100, 255));

        // Green for 40-60
        assert_eq!(temperature_color(40.0), Color::Rgb(50, 200, 50));
        assert_eq!(temperature_color(59.9), Color::Rgb(50, 200, 50));

        // Yellow for 60-80
        assert_eq!(temperature_color(60.0), Color::Rgb(255, 220, 50));
        assert_eq!(temperature_color(79.9), Color::Rgb(255, 220, 50));

        // Orange for 80-90
        assert_eq!(temperature_color(80.0), Color::Rgb(255, 140, 30));
        assert_eq!(temperature_color(89.9), Color::Rgb(255, 140, 30));

        // Red for >= 90
        assert_eq!(temperature_color(90.0), Color::Rgb(255, 40, 40));
        assert_eq!(temperature_color(100.0), Color::Rgb(255, 40, 40));
    }

    #[test]
    fn test_chip_heatmap_no_data() {
        use crate::types::*;
        use std::time::Instant;

        let data = SystemData {
            system_info: SystemInfo {
                name: "Test".into(),
                kernel_version: "1.0".into(),
                os_version: "14.0".into(),
                host_name: "test".into(),
                cpu_arch: "arm64".into(),
                cpu_brand: "Apple M1".into(),
                cpu_core_count: 8,
                e_core_count: 4,
                p_core_count: 4,
                gpu_core_count: 8,
                chip_name: "Apple M1".into(),
            },
            cpu_info: CpuInfo {
                core_usages: vec![],
                average_usage: 0.0,
                power_metrics: CPUMetrics::default(),
            },
            gpu_info: GpuInfo::default(),
            ane_info: AneInfo::default(),
            dram_info: DramInfo::default(),
            thunderbolt_info: ThunderboltInfo::default(),
            disk_io_info: DiskIoInfo::default(),
            memory_info: MemoryInfo {
                total_memory: 0,
                used_memory: 0,
                available_memory: 0,
                total_swap: 0,
                used_swap: 0,
                usage_percentage: 0,
            },
            network_info: vec![],
            temperature_info: vec![],
            process_info: vec![],
            battery_info: BatteryInfo::default(),
            thermal_info: ThermalInfo::default(),
            performance_metrics: PerformanceMetrics::default(),
            system_health: SystemHealthInfo::default(),
            timestamp: Instant::now(),
            terminal_info: TerminalInfo::default(),
        };

        // With no temperature data, average should be None
        assert!(average_temperature(&data).is_none());

        // Region temperatures should also be None
        assert!(region_temperature(&data, DieRegion::ECore).is_none());
        assert!(region_temperature(&data, DieRegion::PCore).is_none());
        assert!(region_temperature(&data, DieRegion::Gpu).is_none());
        assert!(region_temperature(&data, DieRegion::Ane).is_none());
        assert!(region_temperature(&data, DieRegion::Dram).is_none());
    }
}

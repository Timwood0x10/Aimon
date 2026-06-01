//! System sonification panel
//! Maps system metrics to audio-like parameters for visualization:
//! CPU -> pitch, Memory -> volume, Network -> beat rhythm, Temperature -> tone

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::theme::Theme;
use crate::types::SystemData;

/// State for the sonification display
#[derive(Debug, Clone)]
pub struct SonificationState {
    pub pitch: f32,  // 0.0 - 1.0, mapped from CPU usage
    pub volume: f32, // 0.0 - 1.0, mapped from memory usage
    pub beat: f32,   // 0.0 - 1.0, mapped from network activity
    pub tone: f32,   // 0.0 - 1.0, mapped from temperature
}

impl Default for SonificationState {
    fn default() -> Self {
        Self {
            pitch: 0.0,
            volume: 0.0,
            beat: 0.0,
            tone: 0.0,
        }
    }
}

impl SonificationState {
    /// Create state from current system data
    pub fn from_system_data(data: &SystemData) -> Self {
        let pitch = (data.cpu_info.average_usage / 100.0).clamp(0.0, 1.0);
        let volume = (data.memory_info.usage_percentage as f32 / 100.0).clamp(0.0, 1.0);

        // Network beat: normalize total throughput
        let total_net: u64 = data
            .network_info
            .iter()
            .map(|n| n.bytes_received + n.bytes_transmitted)
            .sum();
        // Use log-scale for network to avoid extreme values dominating
        let beat = if total_net > 0 {
            ((total_net as f64).log10() / 12.0).min(1.0) as f32
        } else {
            0.0
        };

        // Temperature tone: average temp normalized to 0-100 range
        let tone = if !data.temperature_info.is_empty() {
            let avg: f32 = data
                .temperature_info
                .iter()
                .map(|t| t.temperature)
                .sum::<f32>()
                / data.temperature_info.len() as f32;
            (avg / 100.0).clamp(0.0, 1.0)
        } else {
            0.0
        };

        Self {
            pitch,
            volume,
            beat,
            tone,
        }
    }
}

/// Generate a text description of the system "sound"
pub fn describe_sonification(data: &SystemData) -> String {
    let state = SonificationState::from_system_data(data);

    let pitch_desc = if state.pitch > 0.8 {
        "high-pitched whine"
    } else if state.pitch > 0.5 {
        "moderate tone"
    } else if state.pitch > 0.2 {
        "low hum"
    } else {
        "near-silent"
    };

    let volume_desc = if state.volume > 0.9 {
        "deafening"
    } else if state.volume > 0.7 {
        "loud"
    } else if state.volume > 0.4 {
        "moderate"
    } else {
        "quiet"
    };

    let beat_desc = if state.beat > 0.7 {
        "rapid-fire rhythm"
    } else if state.beat > 0.4 {
        "steady pulse"
    } else if state.beat > 0.1 {
        "slow beat"
    } else {
        "silence"
    };

    let tone_desc = if state.tone > 0.8 {
        "blazing hot"
    } else if state.tone > 0.6 {
        "warm"
    } else if state.tone > 0.3 {
        "cool"
    } else {
        "cold"
    };

    format!(
        "{} {} with {} and {} tone",
        volume_desc, pitch_desc, beat_desc, tone_desc
    )
}

/// Render a bar visualization for a metric
fn render_bar(label: &str, value: f32, color: Color, width: usize) -> Line<'static> {
    let bar_width = (value * width as f32) as usize;
    let bar_width = bar_width.min(width);
    let empty_width = width - bar_width;

    let bar_char = "\u{2588}"; // full block
    let empty_char = "\u{2591}"; // light shade

    Line::from(vec![
        Span::styled(
            format!("{:>6}: ", label),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(
            bar_char.repeat(bar_width),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            empty_char.repeat(empty_width),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(
            format!(" {:.0}%", value * 100.0),
            Style::default().fg(color),
        ),
    ])
}

/// Render the sonification panel
pub fn render(f: &mut Frame, area: Rect, data: &SystemData, _active: bool, theme: &Theme) {
    let state = SonificationState::from_system_data(data);
    let bar_width = 16;

    let lines: Vec<Line> = vec![
        Line::from(Span::styled(
            describe_sonification(data),
            Style::default().fg(theme.fg),
        )),
        Line::from(""),
        render_bar("Pitch", state.pitch, theme.cpu_color, bar_width),
        render_bar("Volume", state.volume, theme.mem_color, bar_width),
        render_bar("Beat", state.beat, theme.net_rx_color, bar_width),
        render_bar("Tone", state.tone, theme.temp_color, bar_width),
    ];

    let block = Paragraph::new(lines).block(
        Block::default()
            .title(" SONIFICATION ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border_color)),
    );

    f.render_widget(block, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use std::time::Instant;

    fn make_test_system_data() -> SystemData {
        SystemData {
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
                core_usages: vec![50.0, 60.0, 70.0],
                average_usage: 60.0,
                power_metrics: CPUMetrics::default(),
                usage_meta: MetricMeta::unavailable(),
                power_meta: MetricMeta::unavailable(),
            },
            gpu_info: GpuInfo::default(),
            ane_info: AneInfo::default(),
            dram_info: DramInfo::default(),
            thunderbolt_info: ThunderboltInfo::default(),
            disk_io_info: DiskIoInfo::default(),
            disk_usage_info: Vec::new(),
            directory_usage_info: Vec::new(),
            memory_info: MemoryInfo {
                total_memory: 16 * 1024 * 1024 * 1024,
                used_memory: 8 * 1024 * 1024 * 1024,
                available_memory: 8 * 1024 * 1024 * 1024,
                total_swap: 0,
                used_swap: 0,
                usage_percentage: 50,
            },
            network_info: vec![NetworkInterface {
                name: "en0".into(),
                bytes_received: 1024 * 1024,
                bytes_transmitted: 512 * 1024,
                packets_received: 100,
                packets_transmitted: 50,
            }],
            temperature_info: vec![TemperatureInfo {
                label: "CPU".into(),
                temperature: 55.0,
                critical_temperature: 100.0,
            }],
            process_info: vec![],
            battery_info: BatteryInfo::default(),
            thermal_info: ThermalInfo::default(),
            performance_metrics: PerformanceMetrics::default(),
            system_health: SystemHealthInfo::default(),
            timestamp: Instant::now(),
            terminal_info: TerminalInfo::default(),
            carbon_info: crate::carbon::tracker::CarbonTracker::default(),
            capabilities: crate::collectors::capabilities::CollectorCapabilities::default(),
        }
    }

    #[test]
    fn test_sonification_cpu_pitch() {
        let data = make_test_system_data();
        let state = SonificationState::from_system_data(&data);

        // CPU is 60%, pitch should be 0.6
        assert!((state.pitch - 0.6).abs() < 0.01);

        // Memory is 50%, volume should be 0.5
        assert!((state.volume - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_sonification_not_empty() {
        let data = make_test_system_data();
        let description = describe_sonification(&data);

        // Description should be non-empty and contain meaningful words
        assert!(!description.is_empty());
        assert!(description.contains("with"));
        assert!(description.contains("tone"));
    }

    #[test]
    fn test_sonification_state_clamping() {
        let mut data = make_test_system_data();

        // Extreme CPU usage should clamp to 1.0
        data.cpu_info.average_usage = 150.0;
        let state = SonificationState::from_system_data(&data);
        assert!((state.pitch - 1.0).abs() < 0.01);

        // Zero usage should give 0.0
        data.cpu_info.average_usage = 0.0;
        data.memory_info.usage_percentage = 0;
        data.temperature_info = vec![];
        data.network_info = vec![];
        let state = SonificationState::from_system_data(&data);
        assert!((state.pitch).abs() < 0.01);
        assert!((state.volume).abs() < 0.01);
        assert!((state.tone).abs() < 0.01);
    }
}

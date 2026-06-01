//! System health layout - uptime, load averages, temperatures, fans
//! Comprehensive system wellness overview

use crate::history::HistoryData;
use crate::types::SystemData;
use crate::ui::chart;
use crate::ui::components;
use crate::ui::theme::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Create layout for system health view
fn create_system_health_layout(area: Rect) -> Vec<Rect> {
    let main = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Length(10), // Uptime + Load averages
            Constraint::Length(8),  // Temperatures + Fans
            Constraint::Length(8),  // Disk usage
            Constraint::Min(0),     // Temperature history chart
        ])
        .split(area);

    let top_row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main[1]);

    let mid_row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main[2]);

    vec![
        main[0], top_row[0], top_row[1], mid_row[0], mid_row[1], main[3], main[4],
    ]
}

/// Format uptime seconds to human-readable string
fn format_uptime(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;

    if days > 0 {
        format!("{}d {}h {}m", days, hours, minutes)
    } else if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}

/// Draw the system health layout
pub fn draw(f: &mut Frame, data: &SystemData, history: &HistoryData, theme: &Theme) {
    let areas = create_system_health_layout(f.size());

    // Header
    components::render_header(f, areas[0], data, theme);

    // Uptime and system info
    let health = &data.system_health;
    let uptime_lines = vec![
        Line::from(vec![
            Span::styled("Uptime: ", Style::default().fg(theme.fg)),
            Span::styled(
                format_uptime(health.uptime_seconds),
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Sleep/Wake: ", Style::default().fg(theme.fg)),
            Span::styled(
                format!("{:.0}%", health.sleep_wake_efficiency),
                Style::default().fg(theme.fg),
            ),
        ]),
        Line::from(vec![
            Span::styled("Power Quality: ", Style::default().fg(theme.fg)),
            Span::styled(
                format!("{}/100", health.power_quality_score),
                Style::default().fg(theme.fg),
            ),
        ]),
        Line::from(vec![
            Span::styled("Kernel: ", Style::default().fg(theme.fg)),
            Span::styled(
                data.system_info.kernel_version.clone(),
                Style::default().fg(Color::Gray),
            ),
        ]),
    ];

    let uptime_block = Paragraph::new(uptime_lines)
        .style(Style::default().fg(theme.fg).bg(theme.bg))
        .block(
            Block::default()
                .title(" SYSTEM UPTIME ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border_color))
                .style(Style::default().bg(theme.bg)),
        );
    f.render_widget(uptime_block, areas[1]);

    // Load averages
    let load_lines = vec![
        Line::from(vec![
            Span::styled("1 min:  ", Style::default().fg(theme.fg)),
            Span::styled(
                format!("{:.2}", health.system_load_1min),
                Style::default().fg(get_load_color(health.system_load_1min, theme)),
            ),
        ]),
        Line::from(vec![
            Span::styled("5 min:  ", Style::default().fg(theme.fg)),
            Span::styled(
                format!("{:.2}", health.system_load_5min),
                Style::default().fg(get_load_color(health.system_load_5min, theme)),
            ),
        ]),
        Line::from(vec![
            Span::styled("15 min: ", Style::default().fg(theme.fg)),
            Span::styled(
                format!("{:.2}", health.system_load_15min),
                Style::default().fg(get_load_color(health.system_load_15min, theme)),
            ),
        ]),
        Line::from(vec![
            Span::styled("CPU Cores: ", Style::default().fg(theme.fg)),
            Span::styled(
                format!("{}", data.cpu_info.core_usages.len()),
                Style::default().fg(theme.fg),
            ),
        ]),
    ];

    let load_block = Paragraph::new(load_lines)
        .style(Style::default().fg(theme.fg).bg(theme.bg))
        .block(
            Block::default()
                .title(" LOAD AVERAGES ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border_color))
                .style(Style::default().bg(theme.bg)),
        );
    f.render_widget(load_block, areas[2]);

    // Temperatures
    let thermal = &data.thermal_info;
    let mut temp_lines = vec![
        Line::from(vec![
            Span::styled("Thermal Pressure: ", Style::default().fg(theme.fg)),
            Span::styled(
                format!("{}%", thermal.thermal_pressure),
                Style::default().fg(components::thermal_pressure_color(
                    thermal.thermal_pressure,
                    theme,
                )),
            ),
        ]),
        Line::from(vec![
            Span::styled("Throttling: ", Style::default().fg(theme.fg)),
            Span::styled(
                if thermal.thermal_throttling {
                    "YES"
                } else {
                    "NO"
                },
                Style::default().fg(if thermal.thermal_throttling {
                    theme.critical_color
                } else {
                    theme.fg
                }),
            ),
        ]),
    ];

    // Add up to 3 temperature sensors
    for temp in data.temperature_info.iter().take(3) {
        temp_lines.push(Line::from(vec![
            Span::styled(format!("{}: ", temp.label), Style::default().fg(theme.fg)),
            Span::styled(
                format!("{:.0}C", temp.temperature),
                Style::default().fg(get_temp_color(temp.temperature, theme)),
            ),
        ]));
    }

    let temp_block = Paragraph::new(temp_lines)
        .style(Style::default().fg(theme.fg).bg(theme.bg))
        .block(
            Block::default()
                .title(" TEMPERATURES ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border_color))
                .style(Style::default().bg(theme.bg)),
        );
    f.render_widget(temp_block, areas[3]);

    // Fan speeds
    let fan_lines: Vec<Line> = thermal
        .fans
        .iter()
        .map(|fan| {
            Line::from(vec![
                Span::styled(
                    format!("Fan {}: ", fan.id + 1),
                    Style::default().fg(theme.fg),
                ),
                Span::styled(
                    format!("{} RPM {}", fan.current_rpm, fan.mode),
                    Style::default().fg(theme.fg),
                ),
            ])
        })
        .collect();

    let fan_block = Paragraph::new(if fan_lines.is_empty() {
        vec![
            Line::from(Span::styled(
                "No fan RPM exposed",
                Style::default().fg(Color::Gray),
            )),
            Line::from(Span::styled(
                "Fanless Mac or SMC requires sudo",
                Style::default().fg(Color::DarkGray),
            )),
        ]
    } else {
        fan_lines
    })
    .style(Style::default().fg(theme.fg).bg(theme.bg))
    .block(
        Block::default()
            .title(" FANS ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border_color))
            .style(Style::default().bg(theme.bg)),
    );
    f.render_widget(fan_block, areas[4]);

    // Disk usage
    components::render_disk_usage_stats(f, areas[5], data, theme);

    // Temperature history chart
    let chart_config = chart::ChartConfig::new("TEMPERATURE HISTORY", 0.0, 100.0, theme.fg)
        .with_bg(theme.bg)
        .with_border_color(theme.border_color);
    chart::render_chart(f, areas[6], &history.temperature_history, &chart_config);
}

use ratatui::style::Color;

/// Get color for load average value
fn get_load_color(load: f64, theme: &Theme) -> Color {
    if load > 4.0 {
        theme.critical_color
    } else if load > 2.0 {
        theme.warning_color
    } else {
        theme.fg
    }
}

/// Get color for temperature value
fn get_temp_color(temp: f32, theme: &Theme) -> Color {
    if temp > 80.0 {
        theme.critical_color
    } else if temp > 60.0 {
        theme.warning_color
    } else {
        theme.fg
    }
}

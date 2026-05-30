//! Battery focus layout - detailed battery and power monitoring
//! Large battery display with power breakdown and trends

use crate::history::HistoryData;
use crate::types::SystemData;
use crate::ui::chart;
use crate::ui::components;
use crate::ui::theme::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

/// Create layout for battery focus view
fn create_battery_focus_layout(area: Rect) -> Vec<Rect> {
    let main = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Length(10), // Large battery display
            Constraint::Length(8),  // Power breakdown
            Constraint::Min(0),     // Power trend chart
        ])
        .split(area);

    let top_row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main[1]);

    vec![main[0], top_row[0], top_row[1], main[2], main[3]]
}

/// Draw the battery focus layout
pub fn draw(f: &mut Frame, data: &SystemData, history: &HistoryData, theme: &Theme) {
    let areas = create_battery_focus_layout(f.size());

    // Header
    components::render_header(f, areas[0], data, theme);

    // Large battery gauge
    let batt = &data.battery_info;
    let batt_color = if batt.percentage < 20.0 {
        theme.critical_color
    } else if batt.percentage < 50.0 {
        theme.warning_color
    } else {
        theme.battery_color
    };

    let gauge = Gauge::default()
        .block(
            Block::default()
                .title(" BATTERY LEVEL ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(batt_color))
                .style(Style::default().bg(theme.bg)),
        )
        .gauge_style(Style::default().fg(batt_color).bg(theme.bg))
        .ratio(batt.percentage as f64 / 100.0)
        .label(format!("{:.0}%", batt.percentage));
    f.render_widget(gauge, areas[1]);

    // Battery details
    let status_icon = if batt.is_charging {
        "CHARGING"
    } else {
        "DISCHARGING"
    };
    let time_str = match batt.time_remaining {
        Some(mins) => format!("{}h {:02}m remaining", mins / 60, mins % 60),
        None => "Calculating...".to_string(),
    };

    let detail_lines = vec![
        Line::from(vec![
            Span::styled("Status: ", Style::default().fg(theme.fg)),
            Span::styled(
                status_icon,
                Style::default().fg(batt_color).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Health: ", Style::default().fg(theme.fg)),
            Span::styled(
                format!("{:.0}%", batt.health_percentage),
                Style::default().fg(theme.fg),
            ),
        ]),
        Line::from(vec![
            Span::styled("Cycles: ", Style::default().fg(theme.fg)),
            Span::styled(
                format!("{}", batt.cycle_count),
                Style::default().fg(theme.fg),
            ),
        ]),
        Line::from(vec![
            Span::styled("Time: ", Style::default().fg(theme.fg)),
            Span::styled(time_str, Style::default().fg(theme.fg)),
        ]),
        Line::from(vec![
            Span::styled("Adapter: ", Style::default().fg(theme.fg)),
            Span::styled(
                format!("{}W", batt.power_adapter_wattage),
                Style::default().fg(theme.accent),
            ),
        ]),
    ];

    let detail_block = Paragraph::new(detail_lines)
        .style(Style::default().fg(theme.fg).bg(theme.bg))
        .block(
            Block::default()
                .title(" BATTERY DETAILS ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(batt_color))
                .style(Style::default().bg(theme.bg)),
        );
    f.render_widget(detail_block, areas[2]);

    // Power breakdown
    let power = &data.cpu_info.power_metrics;
    let power_lines = vec![
        Line::from(vec![
            Span::styled("Package: ", Style::default().fg(theme.fg)),
            Span::styled(
                format!("{:.1}W", power.package_w),
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("CPU:     ", Style::default().fg(theme.fg)),
            Span::styled(
                format!("{:.1}W", power.cpu_w),
                Style::default().fg(theme.cpu_color),
            ),
        ]),
        Line::from(vec![
            Span::styled("GPU:     ", Style::default().fg(theme.fg)),
            Span::styled(
                format!("{:.1}W", power.gpu_w),
                Style::default().fg(theme.warning_color),
            ),
        ]),
        Line::from(vec![
            Span::styled("ANE:     ", Style::default().fg(theme.fg)),
            Span::styled(
                format!("{:.1}W", power.ane_w),
                Style::default().fg(theme.mem_color),
            ),
        ]),
        Line::from(vec![
            Span::styled("Voltage: ", Style::default().fg(theme.fg)),
            Span::styled(
                format!("{:.2}V", batt.voltage),
                Style::default().fg(theme.fg),
            ),
        ]),
        Line::from(vec![
            Span::styled("Current: ", Style::default().fg(theme.fg)),
            Span::styled(
                format!("{:.2}A", batt.amperage),
                Style::default().fg(theme.fg),
            ),
        ]),
    ];

    let power_block = Paragraph::new(power_lines)
        .style(Style::default().fg(theme.fg).bg(theme.bg))
        .block(
            Block::default()
                .title(" POWER BREAKDOWN ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.accent))
                .style(Style::default().bg(theme.bg)),
        );
    f.render_widget(power_block, areas[3]);

    // CPU history as proxy for power trend
    let chart_config = chart::ChartConfig::new("CPU POWER TREND", 0.0, 100.0, theme.battery_color);
    chart::render_chart(f, areas[4], &history.cpu_history, &chart_config);
}

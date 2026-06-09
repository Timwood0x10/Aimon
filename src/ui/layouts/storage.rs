//! Storage layout - mounted volume usage, directory consumers, disk I/O, trends.

use crate::history::HistoryData;
use crate::types::{DiskUsageInfo, SystemData};
use crate::ui::theme::Theme;
use crate::ui::{chart, components};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Draw the storage focused layout.
pub fn draw(f: &mut Frame, data: &SystemData, history: &HistoryData, theme: &Theme) {
    let areas = create_storage_layout(f.area());

    components::render_header(f, areas[0], data, theme);
    render_mounts(f, areas[1], data, theme);
    render_top_directories(f, areas[2], data, theme);
    components::render_disk_io_stats(f, areas[3], data, theme);

    let usage_chart = chart::ChartConfig::new("PRIMARY DISK USAGE %", 0.0, 100.0, theme.mem_color)
        .with_bg(theme.bg)
        .with_border_color(theme.border_color)
        .with_marker(chart::ChartMarker::HalfBlock)
        .with_shadow(chart::ChartShadow::MediumShade)
        .as_area(0.0);
    chart::render_chart(f, areas[4], &history.disk_usage_history, &usage_chart);

    let io_max = history.get_disk_io_rate().max(1.0) * 1.25;
    let io_chart = chart::ChartConfig::new("DISK I/O TREND B/S", 0.0, io_max, theme.net_tx_color)
        .with_bg(theme.bg)
        .with_border_color(theme.border_color)
        .with_marker(chart::ChartMarker::Bar)
        .with_shadow(chart::ChartShadow::LightShade);
    chart::render_chart(f, areas[5], &history.disk_io_rate_history, &io_chart);

    let trend = history
        .get_disk_usage_trend()
        .map(|value| format!("usage trend {:+.2}%", value))
        .unwrap_or_else(|| "usage trend pending".to_string());
    components::render_status_bar(f, areas[6], theme, "Storage", &trend);
}

fn create_storage_layout(area: Rect) -> Vec<Rect> {
    let main = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(12),
            Constraint::Length(9),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(1),
        ])
        .split(area);

    let top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
        .split(main[1]);

    vec![main[0], top[0], top[1], main[2], main[3], main[4], main[5]]
}

fn render_mounts(f: &mut Frame, area: Rect, data: &SystemData, theme: &Theme) {
    let mut disks = data.disk_usage_info.clone();
    disks.sort_by(|left, right| right.usage_percentage.total_cmp(&left.usage_percentage));

    let lines = if disks.is_empty() {
        vec![Line::from(Span::styled(
            "  No mounted volumes available",
            Style::default().fg(Color::DarkGray),
        ))]
    } else {
        disks
            .iter()
            .take(7)
            .map(|disk| mount_line(disk, theme))
            .collect()
    };

    let block = Paragraph::new(lines)
        .style(Style::default().fg(theme.fg).bg(theme.bg))
        .block(panel_block("MOUNTS", theme));
    f.render_widget(block, area);
}

fn mount_line(disk: &DiskUsageInfo, theme: &Theme) -> Line<'static> {
    let color = usage_color(disk.usage_percentage, theme);
    let filled = (disk.usage_percentage / 10.0).round().clamp(0.0, 10.0) as usize;
    let bar = format!("{}{}", "█".repeat(filled), "░".repeat(10 - filled));
    let removable = if disk.is_removable { "USB" } else { "fixed" };

    Line::from(vec![
        Span::styled(
            format!(" {:<12}", truncate_field(&disk.mount_point, 12)),
            Style::default().fg(theme.fg),
        ),
        Span::styled(bar, Style::default().fg(color)),
        Span::styled(
            format!(" {:>5.1}% ", disk.usage_percentage),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(
                "{} / {} {} {}",
                components::format_storage_bytes(disk.used_bytes),
                components::format_storage_bytes(disk.total_bytes),
                truncate_field(&disk.file_system, 8),
                removable
            ),
            Style::default().fg(Color::DarkGray),
        ),
    ])
}

fn render_top_directories(f: &mut Frame, area: Rect, data: &SystemData, theme: &Theme) {
    let lines = if data.directory_usage_info.is_empty() {
        vec![
            Line::from(Span::styled(
                "  Directory scan pending or no home folders found",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(Span::styled(
                "  Scans are bounded to protect UI responsiveness",
                Style::default().fg(Color::DarkGray),
            )),
        ]
    } else {
        data.directory_usage_info
            .iter()
            .take(8)
            .map(|entry| {
                let partial = if entry.is_partial { " partial" } else { "" };
                Line::from(vec![
                    Span::styled(
                        format!(" {:<28}", truncate_field(&entry.path, 28)),
                        Style::default().fg(theme.fg),
                    ),
                    Span::styled(
                        format!("{:>9}", components::format_storage_bytes(entry.size_bytes)),
                        Style::default()
                            .fg(theme.mem_color)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!(" files {}{}", entry.file_count, partial),
                        Style::default().fg(Color::DarkGray),
                    ),
                ])
            })
            .collect()
    };

    let block = Paragraph::new(lines)
        .style(Style::default().fg(theme.fg).bg(theme.bg))
        .block(panel_block("TOP DIRECTORIES", theme));
    f.render_widget(block, area);
}

fn usage_color(usage_percentage: f32, theme: &Theme) -> Color {
    if usage_percentage > 90.0 {
        theme.critical_color
    } else if usage_percentage > 75.0 {
        theme.warning_color
    } else {
        theme.mem_color
    }
}

fn panel_block(title: &str, theme: &Theme) -> Block<'static> {
    Block::default()
        .title(format!(" {title} "))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border_color))
        .style(Style::default().bg(theme.bg))
}

fn truncate_field(value: &str, max_len: usize) -> String {
    if value.chars().count() <= max_len {
        return value.to_string();
    }
    let mut truncated = value
        .chars()
        .take(max_len.saturating_sub(1))
        .collect::<String>();
    truncated.push('…');
    truncated
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Objective: Verify storage labels are shortened without overflowing.
    /// Invariants: Long labels end with an ellipsis and fit the target width.
    #[test]
    fn test_truncate_field_bounds_long_values() {
        let value = truncate_field("/very/long/storage/path", 8);
        assert!(
            value.ends_with('…'),
            "truncated value should end with ellipsis"
        );
        assert!(
            value.chars().count() <= 8,
            "truncated value should not exceed requested width"
        );
    }
}

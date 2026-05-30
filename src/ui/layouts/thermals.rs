//! Thermals/Fan layout - comprehensive temperature and fan monitoring
//! Matches mactop's Fan & Thermals layout

use crate::config::Config;
use crate::history::HistoryData;
use crate::types::SystemData;
use crate::ui::components;
use crate::ui::theme::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

/// Draw the thermals/fan focused layout
pub fn draw(
    f: &mut Frame,
    data: &SystemData,
    history: &HistoryData,
    config: &Config,
    theme: &Theme,
) {
    let size = f.size();

    let main = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(4),
            Constraint::Min(10),
            Constraint::Length(1),
        ])
        .split(size);

    components::render_header(f, main[0], data, theme);

    let row1 = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main[1]);

    components::render_cpu_gauge(f, row1[0], data.cpu_info.average_usage, theme);
    components::render_gpu_gauge(f, row1[1], data, theme);

    let row2 = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(main[2]);

    components::render_detailed_temperatures(f, row2[0], data, theme);

    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Percentage(30),
            Constraint::Percentage(30),
        ])
        .split(row2[1]);

    components::render_power_breakdown(f, right[0], data, theme);
    components::render_ane_stats(f, right[1], data, theme);
    components::render_dram_stats(f, right[2], data, theme);

    let uptime_str = format_uptime(data.system_health.uptime_seconds);
    components::render_status_bar(f, main[3], theme, "Thermals", &uptime_str);
}

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
//! Compact layout - dense single-screen overview
//! Shows key metrics in a 2x3 grid for quick scanning

use crate::history::HistoryData;
use crate::types::SystemData;
use crate::ui::components;
use crate::ui::theme::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

/// Create a 2x3 grid layout for compact view
fn create_compact_grid(area: Rect) -> Vec<Rect> {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let top_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(rows[0]);

    let bottom_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(rows[1]);

    vec![
        top_cols[0],
        top_cols[1],
        top_cols[2],
        bottom_cols[0],
        bottom_cols[1],
        bottom_cols[2],
    ]
}

/// Draw the compact layout with key metrics in a grid
pub fn draw(f: &mut Frame, data: &SystemData, history: &HistoryData, theme: &Theme) {
    let areas = create_compact_grid(f.area());

    // Top row: CPU history, Memory history, Battery %
    components::render_utilization_history_chart(
        f,
        areas[0],
        "CPU UTIL",
        &history.cpu_history,
        data.cpu_info.average_usage as f64,
        theme,
    );
    components::render_utilization_history_chart(
        f,
        areas[1],
        "MEM UTIL",
        &history.memory_history,
        data.memory_info.usage_percentage as f64,
        theme,
    );
    components::render_battery_stats(f, areas[2], data, theme);

    // Bottom row: Network rates, Top 3 processes, Thermal
    components::render_network_stats(f, areas[3], data, history, theme);
    components::render_compact_process_list(f, areas[4], data, 3, theme);
    components::render_thermal_stats(f, areas[5], data, theme);
}

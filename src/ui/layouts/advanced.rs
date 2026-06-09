//! Advanced layout - comprehensive view matching mactop's full feature set
//! Shows CPU+GPU+ANE+Power+Memory+Network+Disk+Temperature+Fan+Process+Thunderbolt

use crate::carbon::render::render_efficiency_advisor;
use crate::config::Config;
use crate::history::HistoryData;
use crate::types::SystemData;
use crate::ui::components;
use crate::ui::theme::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

/// Draw the advanced layout with all features visible
pub fn draw(
    f: &mut Frame,
    data: &SystemData,
    history: &HistoryData,
    config: &Config,
    theme: &Theme,
) {
    let size = f.area();

    // Main vertical layout: header + content + status
    let main = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Length(4), // CPU + GPU gauges
            Constraint::Length(4), // Memory + Power
            Constraint::Length(7), // CPU cores + ANE/DRAM
            Constraint::Min(8),    // Process list + Network + Disk + Thunderbolt + Thermals
            Constraint::Length(1), // Status bar
        ])
        .split(size);

    // Header
    components::render_header(f, main[0], data, theme);

    // Row 1: CPU gauge + GPU gauge
    let row1 = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main[1]);

    components::render_utilization_history_chart(
        f,
        row1[0],
        "CPU UTIL",
        &history.cpu_history,
        data.cpu_info.average_usage as f64,
        theme,
    );
    components::render_gpu_gauge(f, row1[1], data, theme);

    // Row 2: Memory gauge + Power breakdown
    let row2 = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main[2]);

    components::render_utilization_history_chart(
        f,
        row2[0],
        "MEM UTIL",
        &history.memory_history,
        data.memory_info.usage_percentage as f64,
        theme,
    );
    render_efficiency_advisor(f, row2[1], data, theme);

    // Row 3: CPU cores + ANE + DRAM
    let row3 = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(main[3]);

    components::render_cpu_cores_bar_chart(f, row3[0], data, theme);
    components::render_ane_stats(f, row3[1], data, theme);
    components::render_dram_stats(f, row3[2], data, theme);

    // Row 4: Process list + right side panels
    let row4 = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(35), // Process list
            Constraint::Percentage(65), // Right panels
        ])
        .split(main[4]);

    // Process list
    components::render_process_list(f, row4[0], data, config, theme);

    // Right side: Network + Disk + Storage + Thermals
    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(25), // Network
            Constraint::Percentage(25), // Disk I/O
            Constraint::Percentage(25), // Thunderbolt
            Constraint::Percentage(25), // Thermals
        ])
        .split(row4[1]);

    components::render_network_stats(f, right[0], data, history, theme);
    components::render_disk_io_stats(f, right[1], data, theme);
    components::render_disk_usage_stats(f, right[2], data, theme);
    components::render_detailed_temperatures(f, right[3], data, theme);

    // Status bar
    let uptime_str = format_uptime(data.system_health.uptime_seconds);
    components::render_status_bar(f, main[5], theme, "Advanced", &uptime_str);
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

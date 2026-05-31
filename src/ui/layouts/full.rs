//! Full layout - comprehensive system overview
//! Shows all major system metrics in a multi-panel view

use crate::carbon::render::{render_carbon_panel, render_efficiency_advisor};
use crate::config::Config;
use crate::history::HistoryData;
use crate::types::SystemData;
use crate::ui::components;
use crate::ui::layout;
use crate::ui::theme::Theme;
use ratatui::Frame;

/// Draw the full layout with all system metrics
pub fn draw(
    f: &mut Frame,
    data: &SystemData,
    history: &HistoryData,
    config: &Config,
    theme: &Theme,
) {
    let main = layout::create_full_layout(f.size());

    // Header with system info
    components::render_header(f, main[0], data, theme);

    // Top section: CPU, GPU, Memory, Power
    let top = layout::create_top_stats_layout(main[1]);

    components::render_utilization_history_chart(
        f,
        top[0],
        "CPU UTIL",
        &history.cpu_history,
        data.cpu_info.average_usage as f64,
        theme,
    );
    components::render_gpu_gauge(f, top[1], data, theme);
    components::render_utilization_history_chart(
        f,
        top[2],
        "MEM UTIL",
        &history.memory_history,
        data.memory_info.usage_percentage as f64,
        theme,
    );
    render_efficiency_advisor(f, top[3], data, theme);

    // Middle section: Charts + CPU Cores
    let mid = layout::create_charts_layout(main[2]);

    // Left side: CPU history chart
    let chart_config =
        super::super::chart::ChartConfig::new("CPU HISTORY", 0.0, 100.0, theme.cpu_color)
            .with_bg(theme.bg)
            .with_border_color(theme.border_color);
    super::super::chart::render_chart(f, mid[0], &history.cpu_history, &chart_config);

    // Right side: CPU cores bar chart
    components::render_cpu_cores_bar_chart(f, mid[1], data, theme);

    // Bottom section: Processes, Network, Thermal, Disk I/O, ANE
    let bottom = layout::create_bottom_stats_layout(main[3]);

    components::render_process_list(f, bottom[0], data, config, theme);
    components::render_network_stats(f, bottom[1], data, history, theme);
    components::render_thermal_stats(f, bottom[2], data, theme);
    components::render_disk_io_stats(f, bottom[3], data, theme);
    render_carbon_panel(f, bottom[4], &data.carbon_info, theme);

    // Status bar
    let uptime_str = format_uptime(data.system_health.uptime_seconds);
    components::render_status_bar(f, main[4], theme, "Full", &uptime_str);
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

//! Full layout - comprehensive system overview
//! Shows all major system metrics in a multi-panel view

use ratatui::Frame;
use crate::config::Config;
use crate::history::HistoryData;
use crate::types::SystemData;
use crate::ui::components;
use crate::ui::layout;
use crate::ui::theme::Theme;

/// Draw the full layout with all system metrics
pub fn draw(f: &mut Frame, data: &SystemData, history: &HistoryData, config: &Config, theme: &Theme) {
    let main = layout::create_full_layout(f.size());

    // Header with system info
    components::render_header(f, main[0], data, theme);

    // Top section: CPU, Memory, Battery, Power
    let top = layout::create_top_stats_layout(main[1]);

    components::render_cpu_gauge(f, top[0], data.cpu_info.average_usage, theme);
    components::render_mem_gauge(f, top[1], data.memory_info.usage_percentage, theme);
    components::render_battery_stats(f, top[2], data, theme);
    components::render_power_stats(f, top[3], data, theme);

    // Middle section: Charts + CPU Cores
    let mid = layout::create_charts_layout(main[2]);

    // Left side: CPU history chart
    let chart_config = super::super::chart::ChartConfig::new(
        "CPU HISTORY",
        0.0,
        100.0,
        theme.cpu_color,
    );
    super::super::chart::render_chart(f, mid[0], &history.cpu_history, &chart_config);

    // Right side: CPU cores bar chart
    components::render_cpu_cores_bar_chart(f, mid[1], data, theme);

    // Bottom section: Processes, Network, Thermal, Network Sparkline
    let bottom = layout::create_bottom_stats_layout(main[3]);

    components::render_process_list(f, bottom[0], data, config, theme);
    components::render_network_stats(f, bottom[1], data, history, theme);
    components::render_thermal_stats(f, bottom[2], data, theme);
    components::render_network_sparkline(f, bottom[3], history, theme);
}

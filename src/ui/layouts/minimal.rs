//! Minimal layout - just CPU and memory gauges
//! A clean, distraction-free view of core metrics

use crate::history::HistoryData;
use crate::types::SystemData;
use crate::ui::components;
use crate::ui::layout;
use crate::ui::theme::Theme;
use ratatui::Frame;

/// Draw the minimal layout with only CPU and memory
pub fn draw(f: &mut Frame, data: &SystemData, history: &HistoryData, theme: &Theme) {
    let areas = layout::create_minimal_layout(f.size());

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
}

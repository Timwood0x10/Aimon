//! Minimal layout - just CPU and memory gauges
//! A clean, distraction-free view of core metrics

use crate::types::SystemData;
use crate::ui::components;
use crate::ui::layout;
use crate::ui::theme::Theme;
use ratatui::Frame;

/// Draw the minimal layout with only CPU and memory
pub fn draw(f: &mut Frame, data: &SystemData, theme: &Theme) {
    let areas = layout::create_minimal_layout(f.size());

    components::render_cpu_gauge(f, areas[0], data.cpu_info.average_usage, theme);
    components::render_mem_gauge(f, areas[1], data.memory_info.usage_percentage, theme);
}

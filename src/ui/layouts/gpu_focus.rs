//! GPU focus layout - GPU/ANE power and performance metrics
//! Highlights GPU utilization, power draw, and CPU core activity

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::types::SystemData;
use crate::history::HistoryData;
use crate::ui::components;
use crate::ui::theme::Theme;

/// Create layout for GPU focus view
fn create_gpu_focus_layout(area: Rect) -> Vec<Rect> {
    let main = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Length(8),  // GPU/ANE power stats
            Constraint::Length(12), // CPU cores chart
            Constraint::Min(0),    // Performance metrics
        ])
        .split(area);

    let top_row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(main[1]);

    vec![main[0], top_row[0], top_row[1], main[2], main[3]]
}

/// Draw the GPU focus layout
pub fn draw(f: &mut Frame, data: &SystemData, _history: &HistoryData, theme: &Theme) {
    let areas = create_gpu_focus_layout(f.size());

    // Header
    components::render_header(f, areas[0], data, theme);

    // GPU Power
    let power = &data.cpu_info.power_metrics;
    let gpu_lines = vec![
        Line::from(vec![
            Span::styled("GPU Power: ", Style::default().fg(theme.fg)),
            Span::styled(
                format!("{:.1}W", power.gpu_w),
                Style::default().fg(theme.warning_color).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Package:   ", Style::default().fg(theme.fg)),
            Span::styled(format!("{:.1}W", power.package_w), Style::default().fg(theme.accent)),
        ]),
        Line::from(vec![
            Span::styled("CPU Power: ", Style::default().fg(theme.fg)),
            Span::styled(format!("{:.1}W", power.cpu_w), Style::default().fg(theme.cpu_color)),
        ]),
    ];

    let gpu_block = Paragraph::new(gpu_lines).block(
        Block::default()
            .title(" GPU POWER ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.warning_color)),
    );
    f.render_widget(gpu_block, areas[1]);

    // ANE Power
    let ane_lines = vec![
        Line::from(vec![
            Span::styled("ANE Power: ", Style::default().fg(theme.fg)),
            Span::styled(
                format!("{:.1}W", power.ane_w),
                Style::default().fg(theme.mem_color).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("E-Cluster: ", Style::default().fg(theme.fg)),
            Span::styled(format!("{}%", power.e_cluster_active), Style::default().fg(theme.fg)),
        ]),
        Line::from(vec![
            Span::styled("P-Cluster: ", Style::default().fg(theme.fg)),
            Span::styled(format!("{}%", power.p_cluster_active), Style::default().fg(theme.fg)),
        ]),
    ];

    let ane_block = Paragraph::new(ane_lines).block(
        Block::default()
            .title(" ANE / CLUSTERS ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.mem_color)),
    );
    f.render_widget(ane_block, areas[2]);

    // CPU cores bar chart
    components::render_cpu_cores_bar_chart(f, areas[3], data, theme);

    // Performance metrics
    let perf = &data.performance_metrics;
    let perf_lines = vec![
        Line::from(vec![
            Span::styled("Workload:    ", Style::default().fg(theme.fg)),
            Span::styled(
                perf.workload_type.clone(),
                Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Perf/Watt:   ", Style::default().fg(theme.fg)),
            Span::styled(format!("{:.2}", perf.performance_per_watt), Style::default().fg(theme.fg)),
        ]),
        Line::from(vec![
            Span::styled("Freq Eff:    ", Style::default().fg(theme.fg)),
            Span::styled(format!("{:.2}", perf.frequency_efficiency), Style::default().fg(theme.fg)),
        ]),
        Line::from(vec![
            Span::styled("E-Core Freq: ", Style::default().fg(theme.fg)),
            Span::styled(format!("{} MHz", power.e_cluster_freq_mhz), Style::default().fg(theme.fg)),
        ]),
        Line::from(vec![
            Span::styled("P-Core Freq: ", Style::default().fg(theme.fg)),
            Span::styled(format!("{} MHz", power.p_cluster_freq_mhz), Style::default().fg(theme.fg)),
        ]),
    ];

    let perf_block = Paragraph::new(perf_lines).block(
        Block::default()
            .title(" PERFORMANCE METRICS ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.accent)),
    );
    f.render_widget(perf_block, areas[4]);
}

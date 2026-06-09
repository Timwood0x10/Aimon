//! GPU focus layout - Comprehensive GPU/ANE/DRAM performance monitoring
//! Highlights GPU utilization, frequency, TFLOPs, ANE usage, DRAM bandwidth, and power breakdown

use crate::history::HistoryData;
use crate::types::SystemData;
use crate::ui::components;
use crate::ui::theme::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Create layout for GPU focus view
fn create_gpu_focus_layout(area: Rect) -> Vec<Rect> {
    let main = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // 0: Header
            Constraint::Length(3),  // 1: CPU gauge
            Constraint::Length(3),  // 2: GPU gauge
            Constraint::Length(7),  // 3: GPU stats + ANE stats
            Constraint::Length(7),  // 4: DRAM + Power breakdown
            Constraint::Length(12), // 5: CPU cores bar chart
            Constraint::Min(0),     // 6: Performance metrics / Disk IO
        ])
        .split(area);

    let row3 = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main[3]);

    let row4 = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main[4]);

    vec![
        main[0], // Header
        main[1], // CPU gauge
        main[2], // GPU gauge
        row3[0], // GPU stats
        row3[1], // ANE stats
        row4[0], // DRAM stats
        row4[1], // Power breakdown
        main[5], // CPU cores
        main[6], // Performance / Disk IO
    ]
}

/// Draw the GPU focus layout
pub fn draw(f: &mut Frame, data: &SystemData, history: &HistoryData, theme: &Theme) {
    let areas = create_gpu_focus_layout(f.area());

    // Header
    components::render_header(f, areas[0], data, theme);

    components::render_utilization_history_chart(
        f,
        areas[1],
        "CPU UTIL",
        &history.cpu_history,
        data.cpu_info.average_usage as f64,
        theme,
    );

    // GPU gauge
    components::render_gpu_gauge(f, areas[2], data, theme);

    // GPU stats detail
    components::render_gpu_stats(f, areas[3], data, theme);

    // ANE stats
    components::render_ane_stats(f, areas[4], data, theme);

    // DRAM stats
    components::render_dram_stats(f, areas[5], data, theme);

    // Power breakdown
    components::render_power_breakdown(f, areas[6], data, theme);

    // CPU cores bar chart
    components::render_cpu_cores_bar_chart(f, areas[7], data, theme);

    // Performance metrics + Disk IO in bottom row
    let bottom_split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(areas[8]);

    // Performance metrics
    let power = &data.cpu_info.power_metrics;
    let perf = &data.performance_metrics;
    let perf_lines = vec![
        Line::from(vec![
            Span::styled("  Workload:    ", Style::default().fg(theme.fg)),
            Span::styled(
                perf.workload_type.clone(),
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Perf/Watt:   ", Style::default().fg(theme.fg)),
            Span::styled(
                format!("{:.2}", perf.performance_per_watt),
                Style::default().fg(theme.fg),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Freq Eff:    ", Style::default().fg(theme.fg)),
            Span::styled(
                format!("{:.2}", perf.frequency_efficiency),
                Style::default().fg(theme.fg),
            ),
        ]),
        Line::from(vec![
            Span::styled("  E-Core Freq: ", Style::default().fg(theme.fg)),
            Span::styled(
                format!("{} MHz", power.e_cluster_freq_mhz),
                Style::default().fg(theme.cpu_color),
            ),
        ]),
        Line::from(vec![
            Span::styled("  P-Core Freq: ", Style::default().fg(theme.fg)),
            Span::styled(
                format!("{} MHz", power.p_cluster_freq_mhz),
                Style::default().fg(theme.cpu_color),
            ),
        ]),
    ];

    let perf_block = Paragraph::new(perf_lines)
        .style(Style::default().fg(theme.fg).bg(theme.bg))
        .block(
            Block::default()
                .title(" ◈ PERFORMANCE ")
                .borders(Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Rounded)
                .border_style(
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                )
                .style(Style::default().bg(theme.bg)),
        );
    f.render_widget(perf_block, bottom_split[0]);

    // Disk IO
    components::render_disk_io_stats(f, bottom_split[1], data, theme);
}

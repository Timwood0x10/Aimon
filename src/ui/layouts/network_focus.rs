//! Network focus layout - detailed network interface monitoring
//! Shows all interfaces with large sparklines and real-time rates

use crate::history::HistoryData;
use crate::types::SystemData;
use crate::ui::chart;
use crate::ui::components;
use crate::ui::theme::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Create layout for network focus view
fn create_network_focus_layout(area: Rect) -> Vec<Rect> {
    let main = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Length(10), // Interface stats
            Constraint::Length(8),  // RX sparkline
            Constraint::Min(0),     // TX sparkline
        ])
        .split(area);

    let interface_row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(main[1]);

    vec![
        main[0],
        interface_row[0],
        interface_row[1],
        interface_row[2],
        main[2],
        main[3],
    ]
}

/// Format bytes per second to human-readable string
fn format_rate(rate: f64) -> String {
    if rate >= 1024.0 * 1024.0 {
        format!("{:.2} MB/s", rate / 1024.0 / 1024.0)
    } else if rate >= 1024.0 {
        format!("{:.2} KB/s", rate / 1024.0)
    } else {
        format!("{:.0} B/s", rate)
    }
}

/// Draw the network focus layout
pub fn draw(f: &mut Frame, data: &SystemData, history: &HistoryData, theme: &Theme) {
    let areas = create_network_focus_layout(f.size());

    // Header
    components::render_header(f, areas[0], data, theme);

    // Show up to 3 network interfaces
    let interfaces: Vec<_> = data.network_info.iter().take(3).collect();
    for (i, iface) in interfaces.iter().enumerate() {
        let area = areas[1 + i];
        let rx_gb = iface.bytes_received as f64 / 1024.0 / 1024.0 / 1024.0;
        let tx_gb = iface.bytes_transmitted as f64 / 1024.0 / 1024.0 / 1024.0;

        let lines = vec![
            Line::from(Span::styled(
                iface.name.clone(),
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(vec![
                Span::styled("RX: ", Style::default().fg(theme.net_rx_color)),
                Span::styled(format!("{:.2} GB", rx_gb), Style::default().fg(theme.fg)),
            ]),
            Line::from(vec![
                Span::styled("TX: ", Style::default().fg(theme.net_tx_color)),
                Span::styled(format!("{:.2} GB", tx_gb), Style::default().fg(theme.fg)),
            ]),
            Line::from(vec![
                Span::styled("Pkts RX: ", Style::default().fg(theme.fg)),
                Span::styled(
                    format!("{}", iface.packets_received),
                    Style::default().fg(theme.fg),
                ),
            ]),
            Line::from(vec![
                Span::styled("Pkts TX: ", Style::default().fg(theme.fg)),
                Span::styled(
                    format!("{}", iface.packets_transmitted),
                    Style::default().fg(theme.fg),
                ),
            ]),
        ];

        let block = Paragraph::new(lines)
            .style(Style::default().fg(theme.fg).bg(theme.bg))
            .block(
                Block::default()
                    .title(format!(" {} ", iface.name))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.net_rx_color))
                    .style(Style::default().bg(theme.bg)),
            );
        f.render_widget(block, area);
    }

    // If fewer than 3 interfaces, fill remaining slots with placeholder
    for i in interfaces.len()..3 {
        let area = areas[1 + i];
        let block = Paragraph::new("No interface")
            .style(Style::default().fg(theme.fg).bg(theme.bg))
            .block(
                Block::default()
                    .title(" N/A ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.fg))
                    .style(Style::default().bg(theme.bg)),
            );
        f.render_widget(block, area);
    }

    // Real-time rates
    let rx_rate = history.get_network_rx_rate();
    let tx_rate = history.get_network_tx_rate();
    let rate_lines = vec![
        Line::from(vec![
            Span::styled("RX Rate: ", Style::default().fg(theme.net_rx_color)),
            Span::styled(
                format_rate(rx_rate),
                Style::default()
                    .fg(theme.net_rx_color)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("TX Rate: ", Style::default().fg(theme.net_tx_color)),
            Span::styled(
                format_rate(tx_rate),
                Style::default()
                    .fg(theme.net_tx_color)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
    ];

    let rate_block = Paragraph::new(rate_lines)
        .style(Style::default().fg(theme.fg).bg(theme.bg))
        .block(
            Block::default()
                .title(" REAL-TIME RATES ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.accent))
                .style(Style::default().bg(theme.bg)),
        );
    f.render_widget(rate_block, areas[4]);

    // RX sparkline (large)
    let rx_config = chart::SparklineConfig::new("NETWORK RX", theme.net_rx_color);
    let rx_data: std::collections::VecDeque<u64> = history
        .network_rx_rate_history
        .iter()
        .map(|&v| v as u64)
        .collect();
    chart::render_sparkline(f, areas[5], &rx_data, &rx_config);
}

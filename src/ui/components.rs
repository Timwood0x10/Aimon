//! UI components for rendering specific data types
//! Contains reusable widgets and rendering functions

use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

use super::chart;
use super::theme::Theme;
use crate::config::{Config, ProcessSortBy};
use crate::history::HistoryData;
use crate::types::*;

/// Base style with theme foreground and background applied to all content
fn base_style(theme: &Theme) -> Style {
    Style::default().fg(theme.fg).bg(theme.bg)
}

/// Render header with system info
pub fn render_header(f: &mut Frame, area: Rect, data: &SystemData, theme: &Theme) {
    let header_line = Line::from(vec![
        Span::styled(
            " ◆ SYSTEM ALERT ",
            Style::default()
                .fg(theme.bg)
                .bg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" {}{}", " ", data.system_info.host_name),
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{}{}", " │ ", data.system_info.os_version),
            Style::default().fg(theme.fg),
        ),
        Span::styled(
            format!("{}{}", " │ ", data.system_info.cpu_brand),
            Style::default().fg(theme.cpu_color),
        ),
        Span::styled(
            format!("{}{}", " │ ", data.system_info.cpu_arch),
            Style::default().fg(Color::Gray),
        ),
    ]);

    let header = Paragraph::new(header_line)
        .style(base_style(theme))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Double)
                .border_style(
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                )
                .style(Style::default().bg(theme.bg)),
        );

    f.render_widget(header, area);
}

/// Render a CPU usage gauge with gradient bar
pub fn render_cpu_gauge(f: &mut Frame, area: Rect, usage: f32, theme: &Theme) {
    let color = if usage > 90.0 {
        theme.critical_color
    } else if usage > 70.0 {
        theme.warning_color
    } else {
        theme.cpu_color
    };

    let gauge = Gauge::default()
        .block(
            Block::default()
                .title(format!(" ◈ CPU {:.1}% ", usage))
                .borders(Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Rounded)
                .border_style(Style::default().fg(color).add_modifier(Modifier::BOLD))
                .style(Style::default().bg(theme.bg).fg(theme.fg)),
        )
        .gauge_style(
            Style::default()
                .fg(color)
                .bg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .ratio(usage as f64 / 100.0)
        .label(Span::styled(
            format!("{:.1}%", usage),
            Style::default()
                .fg(Color::White)
                .bg(color)
                .add_modifier(Modifier::BOLD),
        ));

    f.render_widget(gauge, area);
}

/// Render a memory usage gauge with gradient bar
pub fn render_mem_gauge(f: &mut Frame, area: Rect, usage: u16, theme: &Theme) {
    let color = if usage > 90 {
        theme.critical_color
    } else if usage > 70 {
        theme.warning_color
    } else {
        theme.mem_color
    };

    let gauge = Gauge::default()
        .block(
            Block::default()
                .title(format!(" ◈ MEMORY {}% ", usage))
                .borders(Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Rounded)
                .border_style(Style::default().fg(color).add_modifier(Modifier::BOLD))
                .style(Style::default().bg(theme.bg).fg(theme.fg)),
        )
        .gauge_style(
            Style::default()
                .fg(color)
                .bg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .ratio(usage as f64 / 100.0)
        .label(Span::styled(
            format!("{}%", usage),
            Style::default()
                .fg(Color::White)
                .bg(color)
                .add_modifier(Modifier::BOLD),
        ));

    f.render_widget(gauge, area);
}

/// Render battery statistics panel
pub fn render_battery_stats(f: &mut Frame, area: Rect, data: &SystemData, theme: &Theme) {
    let batt = &data.battery_info;
    let color = if batt.percentage < 20.0 {
        theme.critical_color
    } else if batt.percentage < 50.0 {
        theme.warning_color
    } else {
        theme.battery_color
    };

    let status = if batt.is_charging { "⚡" } else { "🔋" };
    let lines = vec![
        Line::from(Span::styled(
            format!("{} {:.0}%", status, batt.percentage),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("  Health: {:.0}%", batt.health_percentage),
            Style::default().fg(theme.fg),
        )),
        Line::from(Span::styled(
            format!("  Cycles: {}", batt.cycle_count),
            Style::default().fg(theme.fg),
        )),
        Line::from(Span::styled(
            format!("  Power: {}W", batt.power_adapter_wattage),
            Style::default().fg(theme.fg),
        )),
    ];

    let block = Paragraph::new(lines).style(base_style(theme)).block(
        Block::default()
            .title(" ◈ BATTERY ".to_string())
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(
                Style::default()
                    .fg(theme.battery_color)
                    .add_modifier(Modifier::BOLD),
            )
            .style(Style::default().bg(theme.bg)),
    );

    f.render_widget(block, area);
}

/// Render power consumption stats
pub fn render_power_stats(f: &mut Frame, area: Rect, data: &SystemData, theme: &Theme) {
    let power = &data.cpu_info.power_metrics;
    let lines = vec![
        Line::from(Span::styled(
            format!("  ⚡ Total: {:.1}W", power.package_w),
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("  ◆ CPU: {:.1}W", power.cpu_w),
            Style::default().fg(theme.cpu_color),
        )),
        Line::from(Span::styled(
            format!("  ◆ GPU: {:.1}W", power.gpu_w),
            Style::default().fg(theme.warning_color),
        )),
        Line::from(Span::styled(
            format!("  ◆ ANE: {:.1}W", power.ane_w),
            Style::default().fg(theme.mem_color),
        )),
    ];

    let block = Paragraph::new(lines).style(base_style(theme)).block(
        Block::default()
            .title(" ◈ POWER ".to_string())
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )
            .style(Style::default().bg(theme.bg)),
    );

    f.render_widget(block, area);
}

/// Render process list
pub fn render_process_list(
    f: &mut Frame,
    area: Rect,
    data: &SystemData,
    config: &Config,
    theme: &Theme,
) {
    // Use indices to avoid cloning the entire process list on every render
    let mut indices: Vec<usize> = (0..data.process_info.len()).collect();

    // Sort by configured criteria
    match config.process_sort_by {
        ProcessSortBy::Cpu => {
            indices.sort_by(|&a, &b| {
                data.process_info[b]
                    .cpu_usage
                    .partial_cmp(&data.process_info[a].cpu_usage)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }
        ProcessSortBy::Memory => {
            indices.sort_by(|&a, &b| {
                data.process_info[b]
                    .memory_usage
                    .cmp(&data.process_info[a].memory_usage)
            });
        }
        ProcessSortBy::Pid => {
            indices.sort_by(|&a, &b| {
                data.process_info[a]
                    .pid
                    .as_u32()
                    .cmp(&data.process_info[b].pid.as_u32())
            });
        }
        ProcessSortBy::Name => {
            indices.sort_by(|&a, &b| data.process_info[a].name.cmp(&data.process_info[b].name));
        }
    }

    // Only show top 8 processes
    indices.truncate(8);

    let header = Line::from(vec![
        Span::styled(
            "  PID  ",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " CPU%  ",
            Style::default()
                .fg(theme.cpu_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "  MEM  ",
            Style::default()
                .fg(theme.mem_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "NAME",
            Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
        ),
    ]);

    let mut all_lines = vec![header];

    for (row, &i) in indices.iter().enumerate() {
        let p = &data.process_info[i];
        let cpu_color = if p.cpu_usage > 50.0 {
            theme.critical_color
        } else if p.cpu_usage > 20.0 {
            theme.warning_color
        } else {
            theme.fg
        };

        let line = Line::from(vec![
            Span::styled(format!("{:>6} ", p.pid), Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:>5.1}% ", p.cpu_usage),
                Style::default().fg(cpu_color),
            ),
            Span::styled(
                format!("{:>6.0}M ", p.memory_usage as f64 / 1024.0 / 1024.0),
                Style::default().fg(theme.mem_color),
            ),
            Span::styled(
                p.name.chars().take(20).collect::<String>(),
                Style::default().fg(if row % 2 == 0 { theme.fg } else { Color::Gray }),
            ),
        ]);
        all_lines.push(line);
    }

    let block = Paragraph::new(all_lines).style(base_style(theme)).block(
        Block::default()
            .title(" ◈ PROCESSES ".to_string())
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(
                Style::default()
                    .fg(Color::Gray)
                    .add_modifier(Modifier::BOLD),
            )
            .style(Style::default().bg(theme.bg)),
    );

    f.render_widget(block, area);
}

/// Render network statistics
pub fn render_network_stats(
    f: &mut Frame,
    area: Rect,
    data: &SystemData,
    history: &HistoryData,
    theme: &Theme,
) {
    let total_rx: u64 = data.network_info.iter().map(|n| n.bytes_received).sum();
    let total_tx: u64 = data.network_info.iter().map(|n| n.bytes_transmitted).sum();

    // Get real-time rates
    let rx_rate = history.get_network_rx_rate();
    let tx_rate = history.get_network_tx_rate();

    // Format rate display
    let format_rate = |rate: f64| -> String {
        if rate >= 1024.0 * 1024.0 {
            format!("{:.2} MB/s", rate / 1024.0 / 1024.0)
        } else if rate >= 1024.0 {
            format!("{:.2} KB/s", rate / 1024.0)
        } else {
            format!("{:.0} B/s", rate)
        }
    };

    let lines = vec![
        Line::from(vec![
            Span::styled(
                "↓ ",
                Style::default()
                    .fg(theme.net_rx_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{:.2} GB", total_rx as f64 / 1024.0 / 1024.0 / 1024.0),
                Style::default().fg(theme.net_rx_color),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "↑ ",
                Style::default()
                    .fg(theme.net_tx_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{:.2} GB", total_tx as f64 / 1024.0 / 1024.0 / 1024.0),
                Style::default().fg(theme.net_tx_color),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "↓ ",
                Style::default()
                    .fg(theme.net_rx_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format_rate(rx_rate),
                Style::default().fg(theme.net_rx_color),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "↑ ",
                Style::default()
                    .fg(theme.net_tx_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format_rate(tx_rate),
                Style::default().fg(theme.net_tx_color),
            ),
        ]),
    ];

    let block = Paragraph::new(lines).style(base_style(theme)).block(
        Block::default()
            .title(" ◈ NETWORK ".to_string())
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(
                Style::default()
                    .fg(theme.net_rx_color)
                    .add_modifier(Modifier::BOLD),
            )
            .style(Style::default().bg(theme.bg)),
    );

    f.render_widget(block, area);
}

/// Render thermal statistics
pub fn render_thermal_stats(f: &mut Frame, area: Rect, data: &SystemData, theme: &Theme) {
    let thermal = &data.thermal_info;
    let pressure_color = if thermal.thermal_pressure > 80 {
        theme.critical_color
    } else if thermal.thermal_pressure > 50 {
        theme.warning_color
    } else {
        theme.temp_color
    };

    let lines = vec![
        Line::from(vec![
            Span::styled("Pressure: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{}%", thermal.thermal_pressure),
                Style::default()
                    .fg(pressure_color)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Throttle: ", Style::default().fg(Color::Gray)),
            Span::styled(
                if thermal.thermal_throttling {
                    "YES"
                } else {
                    "NO"
                },
                Style::default()
                    .fg(if thermal.thermal_throttling {
                        theme.critical_color
                    } else {
                        theme.mem_color
                    })
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Fans: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{} RPM", thermal.fan_speeds.iter().max().unwrap_or(&0)),
                Style::default().fg(theme.fg),
            ),
        ]),
    ];

    let block = Paragraph::new(lines).style(base_style(theme)).block(
        Block::default()
            .title(" ◈ THERMAL ".to_string())
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(
                Style::default()
                    .fg(theme.temp_color)
                    .add_modifier(Modifier::BOLD),
            )
            .style(Style::default().bg(theme.bg)),
    );

    f.render_widget(block, area);
}

/// Get color based on CPU usage percentage
fn get_cpu_usage_color(usage: f32, theme: &Theme) -> Color {
    if usage > 90.0 {
        theme.critical_color
    } else if usage > 70.0 {
        theme.warning_color
    } else if usage > 50.0 {
        Color::Yellow
    } else {
        Color::Green
    }
}

/// Render CPU core usage bar chart with gradient bars
pub fn render_cpu_cores_bar_chart(f: &mut Frame, area: Rect, data: &SystemData, theme: &Theme) {
    let core_usages = &data.cpu_info.core_usages;
    if core_usages.is_empty() {
        return;
    }

    let num_cores = core_usages.len();
    let max_cores_per_row = 4;
    let _rows = num_cores.div_ceil(max_cores_per_row);

    // Create lines for each core
    let mut lines = Vec::new();

    for (i, &usage) in core_usages.iter().enumerate() {
        let core_color = get_cpu_usage_color(usage, theme);
        let bar_length = (usage / 100.0 * 20.0) as usize;
        let bar = "█".repeat(bar_length);
        let empty = "░".repeat(20 - bar_length);

        let line = Line::from(vec![
            Span::styled(format!("C{:02} ", i), Style::default().fg(Color::Gray)),
            Span::styled(format!("{}{}", bar, empty), Style::default().fg(core_color)),
            Span::styled(format!(" {:5.1}%", usage), Style::default().fg(theme.fg)),
        ]);
        lines.push(line);
    }

    let block = Paragraph::new(lines).style(base_style(theme)).block(
        Block::default()
            .title(" ◈ CPU CORES ".to_string())
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(
                Style::default()
                    .fg(theme.cpu_color)
                    .add_modifier(Modifier::BOLD),
            )
            .style(Style::default().bg(theme.bg)),
    );

    f.render_widget(block, area);
}

/// Render network sparkline
pub fn render_network_sparkline(f: &mut Frame, area: Rect, history: &HistoryData, theme: &Theme) {
    let config = chart::SparklineConfig::new("NET RX", theme.net_rx_color);
    chart::render_sparkline(f, area, &history.network_rx_history, &config);
}

/// Get color for thermal pressure value
pub fn thermal_pressure_color(pressure: u8, theme: &Theme) -> Color {
    if pressure > 80 {
        theme.critical_color
    } else if pressure > 50 {
        theme.warning_color
    } else {
        theme.temp_color
    }
}

/// Render a compact process list showing only top N processes
pub fn render_compact_process_list(
    f: &mut Frame,
    area: Rect,
    data: &SystemData,
    max_count: usize,
    theme: &Theme,
) {
    let mut indices: Vec<usize> = (0..data.process_info.len()).collect();
    indices.sort_by(|&a, &b| {
        data.process_info[b]
            .cpu_usage
            .partial_cmp(&data.process_info[a].cpu_usage)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    indices.truncate(max_count);

    let lines: Vec<Line> = indices
        .iter()
        .map(|&i| {
            let p = &data.process_info[i];
            let cpu_color = if p.cpu_usage > 50.0 {
                theme.critical_color
            } else if p.cpu_usage > 20.0 {
                theme.warning_color
            } else {
                theme.fg
            };

            Line::from(vec![
                Span::styled(
                    format!("{:>5.1}% ", p.cpu_usage),
                    Style::default().fg(cpu_color),
                ),
                Span::styled(
                    p.name.chars().take(15).collect::<String>(),
                    Style::default().fg(theme.fg),
                ),
            ])
        })
        .collect();

    let block = Paragraph::new(lines).style(base_style(theme)).block(
        Block::default()
            .title(" ◈ TOP PROCS ".to_string())
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(
                Style::default()
                    .fg(Color::Gray)
                    .add_modifier(Modifier::BOLD),
            )
            .style(Style::default().bg(theme.bg)),
    );

    f.render_widget(block, area);
}

/// Render a centered help overlay with all keybindings
pub fn render_help_overlay(f: &mut Frame, theme: &Theme) {
    use ratatui::layout::{Constraint, Direction, Layout};

    // Create a centered area for the help popup
    let area = f.size();
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(15),
            Constraint::Percentage(70),
            Constraint::Percentage(15),
        ])
        .split(area);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(60),
            Constraint::Percentage(20),
        ])
        .split(vertical[1]);

    let popup_area = horizontal[1];

    let help_lines = vec![
        Line::from(Span::styled(
            " KEYBINDINGS ",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            " NAVIGATION",
            Style::default()
                .fg(theme.cpu_color)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled("  h / Left    ", Style::default().fg(theme.accent)),
            Span::styled("Previous layout", Style::default().fg(theme.fg)),
        ]),
        Line::from(vec![
            Span::styled("  l / Right   ", Style::default().fg(theme.accent)),
            Span::styled("Next layout", Style::default().fg(theme.fg)),
        ]),
        Line::from(vec![
            Span::styled("  j / Down    ", Style::default().fg(theme.accent)),
            Span::styled("Scroll down", Style::default().fg(theme.fg)),
        ]),
        Line::from(vec![
            Span::styled("  k / Up      ", Style::default().fg(theme.accent)),
            Span::styled("Scroll up", Style::default().fg(theme.fg)),
        ]),
        Line::from(vec![
            Span::styled("  g           ", Style::default().fg(theme.accent)),
            Span::styled("Go to top", Style::default().fg(theme.fg)),
        ]),
        Line::from(vec![
            Span::styled("  G           ", Style::default().fg(theme.accent)),
            Span::styled("Go to bottom", Style::default().fg(theme.fg)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            " LAYOUTS",
            Style::default()
                .fg(theme.cpu_color)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled("  1-7         ", Style::default().fg(theme.accent)),
            Span::styled(
                "Jump to layout (Full/Minimal/Compact/Battery/GPU/Network/Health)",
                Style::default().fg(theme.fg),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            " ACTIONS",
            Style::default()
                .fg(theme.cpu_color)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled("  s / S       ", Style::default().fg(theme.accent)),
            Span::styled(
                "Cycle process sort forward/backward",
                Style::default().fg(theme.fg),
            ),
        ]),
        Line::from(vec![
            Span::styled("  /           ", Style::default().fg(theme.accent)),
            Span::styled("Search processes", Style::default().fg(theme.fg)),
        ]),
        Line::from(vec![
            Span::styled("  F9          ", Style::default().fg(theme.accent)),
            Span::styled("Kill selected process", Style::default().fg(theme.fg)),
        ]),
        Line::from(vec![
            Span::styled("  t           ", Style::default().fg(theme.accent)),
            Span::styled("Cycle theme", Style::default().fg(theme.fg)),
        ]),
        Line::from(vec![
            Span::styled("  n           ", Style::default().fg(theme.accent)),
            Span::styled("Toggle notifications", Style::default().fg(theme.fg)),
        ]),
        Line::from(vec![
            Span::styled("  r           ", Style::default().fg(theme.accent)),
            Span::styled("Force refresh", Style::default().fg(theme.fg)),
        ]),
        Line::from(vec![
            Span::styled("  ?           ", Style::default().fg(theme.accent)),
            Span::styled("Toggle this help", Style::default().fg(theme.fg)),
        ]),
        Line::from(vec![
            Span::styled("  q / Ctrl+C  ", Style::default().fg(theme.accent)),
            Span::styled("Quit", Style::default().fg(theme.fg)),
        ]),
    ];

    let help_popup = Paragraph::new(help_lines)
        .block(
            Block::default()
                .title(" HELP (? to close) ")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_style(
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
        )
        .alignment(Alignment::Left);

    // Clear the background area with a dim overlay
    let clear_block = Block::default().style(Style::default().bg(Color::Black));
    f.render_widget(clear_block, area);

    f.render_widget(help_popup, popup_area);
}

/// Render terminal/PTMX information panel
pub fn render_terminal_info(f: &mut Frame, area: Rect, data: &SystemData, theme: &Theme) {
    let term = &data.terminal_info;

    let mut lines = vec![
        Line::from(Span::styled(
            format!("  ◈ Total PTMX: {}", term.total_ptmx_count),
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!("  ◈ Local Terminals: {}", term.local_ptmx_count),
            Style::default().fg(theme.cpu_color),
        )),
        Line::from(""),
    ];

    // Show top terminal processes
    if !term.terminal_processes.is_empty() {
        lines.push(Line::from(Span::styled(
            "  Top Terminal Processes:",
            Style::default().fg(Color::Gray),
        )));

        for proc in term.terminal_processes.iter().take(5) {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {:>6} ", proc.pid),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    format!("{:<20} ", proc.name.chars().take(20).collect::<String>()),
                    Style::default().fg(theme.fg),
                ),
                Span::styled(
                    format!("{} PTY", proc.pty_count),
                    Style::default().fg(theme.mem_color),
                ),
            ]));
        }
    } else {
        lines.push(Line::from(Span::styled(
            "  No active terminals detected",
            Style::default().fg(Color::DarkGray),
        )));
    }

    let block = Paragraph::new(lines).style(base_style(theme)).block(
        Block::default()
            .title(" ◈ TERMINALS ".to_string())
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )
            .style(Style::default().bg(theme.bg)),
    );

    f.render_widget(block, area);
}

/// Render a status bar at the bottom of the screen
pub fn render_status_bar(
    f: &mut Frame,
    area: Rect,
    theme: &Theme,
    layout_name: &str,
    uptime_str: &str,
) {
    let left = format!(" {} ", layout_name);
    let center = " h/l:Layout  j/k:Scroll  ?:Help  t:Theme  q:Quit ";
    let right = format!(" Up {} ", uptime_str);

    let bar = Paragraph::new(Line::from(vec![
        Span::styled(
            &left,
            Style::default()
                .fg(theme.bg)
                .bg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(center, Style::default().fg(theme.fg).bg(theme.bg)),
        Span::raw(" ".to_string()),
        Span::styled(&right, Style::default().fg(theme.bg).bg(theme.cpu_color)),
    ]))
    .style(Style::default().bg(theme.bg));

    f.render_widget(bar, area);
}

/// Render detailed temperature sensors panel (like mactop's Fan & Thermals layout)
pub fn render_detailed_temperatures(f: &mut Frame, area: Rect, data: &SystemData, theme: &Theme) {
    let temps = &data.temperature_info;
    let thermal = &data.thermal_info;

    let mut lines = vec![
        Line::from(Span::styled(
            format!("  ◈ Thermal Pressure: {}%", thermal.thermal_pressure),
            Style::default()
                .fg(if thermal.thermal_pressure > 80 {
                    theme.critical_color
                } else if thermal.thermal_pressure > 50 {
                    theme.warning_color
                } else {
                    theme.temp_color
                })
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!(
                "  ◈ Throttling: {}",
                if thermal.thermal_throttling {
                    "YES ⚠️"
                } else {
                    "NO ✅"
                }
            ),
            Style::default().fg(if thermal.thermal_throttling {
                theme.critical_color
            } else {
                theme.fg
            }),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "  Temperature Sensors:",
            Style::default()
                .fg(Color::Gray)
                .add_modifier(Modifier::BOLD),
        )),
    ];

    // Show all temperature sensors
    if temps.is_empty() {
        lines.push(Line::from(Span::styled(
            "    No temperature sensors available",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        for temp in temps.iter().take(12) {
            let color = if temp.temperature > temp.critical_temperature * 0.9 {
                theme.critical_color
            } else if temp.temperature > temp.critical_temperature * 0.7 {
                theme.warning_color
            } else {
                theme.temp_color
            };

            lines.push(Line::from(vec![
                Span::styled(
                    format!("    {:<20} ", temp.label),
                    Style::default().fg(theme.fg),
                ),
                Span::styled(
                    format!("{:.1}°C", temp.temperature),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" / {:.1}°C", temp.critical_temperature),
                    Style::default().fg(Color::DarkGray),
                ),
            ]));
        }
    }

    // Fan information
    if !thermal.fan_speeds.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  Fans:",
            Style::default()
                .fg(Color::Gray)
                .add_modifier(Modifier::BOLD),
        )));

        for (i, speed) in thermal.fan_speeds.iter().enumerate() {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("    Fan {}: ", i + 1),
                    Style::default().fg(theme.fg),
                ),
                Span::styled(
                    format!("{} RPM", speed),
                    Style::default()
                        .fg(theme.cpu_color)
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
        }
    }

    let block = Paragraph::new(lines).style(base_style(theme)).block(
        Block::default()
            .title(" ◈ THERMALS ")
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(
                Style::default()
                    .fg(theme.temp_color)
                    .add_modifier(Modifier::BOLD),
            )
            .style(Style::default().bg(theme.bg)),
    );

    f.render_widget(block, area);
}

/// Render disk I/O statistics panel
pub fn render_disk_io(f: &mut Frame, area: Rect, data: &SystemData, theme: &Theme) {
    let mut total_read: u64 = 0;
    let mut total_write: u64 = 0;

    for proc in data.process_info.iter() {
        total_read += proc.disk_read_bytes;
        total_write += proc.disk_write_bytes;
    }

    fn format_bytes(bytes: u64) -> String {
        if bytes >= 1024 * 1024 * 1024 {
            format!("{:.2} GB/s", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
        } else if bytes >= 1024 * 1024 {
            format!("{:.2} MB/s", bytes as f64 / (1024.0 * 1024.0))
        } else if bytes >= 1024 {
            format!("{:.2} KB/s", bytes as f64 / 1024.0)
        } else {
            format!("{} B/s", bytes)
        }
    }

    let lines = vec![
        Line::from(Span::styled(
            format!("  ↓ Read:  {}", format_bytes(total_read)),
            Style::default()
                .fg(theme.net_rx_color)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("  ↑ Write: {}", format_bytes(total_write)),
            Style::default()
                .fg(theme.net_tx_color)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("  Total R: {}", format_bytes(total_read + total_write)),
            Style::default().fg(theme.accent),
        )),
    ];

    let block = Paragraph::new(lines).style(base_style(theme)).block(
        Block::default()
            .title(" ◈ DISK I/O ")
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(
                Style::default()
                    .fg(theme.mem_color)
                    .add_modifier(Modifier::BOLD),
            )
            .style(Style::default().bg(theme.bg)),
    );

    f.render_widget(block, area);
}

/// Render GPU usage gauge with frequency and TFLOPs
pub fn render_gpu_gauge(f: &mut Frame, area: Rect, data: &SystemData, theme: &Theme) {
    let gpu = &data.gpu_info;
    let color = if gpu.usage_percentage > 90.0 {
        theme.critical_color
    } else if gpu.usage_percentage > 70.0 {
        theme.warning_color
    } else {
        theme.cpu_color
    };

    let label = if gpu.freq_mhz > 0 {
        format!("{:.1}% @ {}MHz", gpu.usage_percentage, gpu.freq_mhz)
    } else {
        format!("{:.1}%", gpu.usage_percentage)
    };

    let gauge = Gauge::default()
        .block(
            Block::default()
                .title(format!(" ◈ GPU {} ", label))
                .borders(Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Rounded)
                .border_style(Style::default().fg(color).add_modifier(Modifier::BOLD))
                .style(Style::default().bg(theme.bg).fg(theme.fg)),
        )
        .gauge_style(
            Style::default()
                .fg(color)
                .bg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .ratio(gpu.usage_percentage as f64 / 100.0)
        .label(Span::styled(
            label,
            Style::default()
                .fg(Color::White)
                .bg(color)
                .add_modifier(Modifier::BOLD),
        ));

    f.render_widget(gauge, area);
}

/// Render GPU detailed stats panel
pub fn render_gpu_stats(f: &mut Frame, area: Rect, data: &SystemData, theme: &Theme) {
    let gpu = &data.gpu_info;
    let color = if gpu.usage_percentage > 90.0 {
        theme.critical_color
    } else if gpu.usage_percentage > 70.0 {
        theme.warning_color
    } else {
        theme.cpu_color
    };

    let mut lines = vec![
        Line::from(Span::styled(
            format!("  ◈ Usage: {:.1}%", gpu.usage_percentage),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!("  ◈ Freq:  {} MHz", gpu.freq_mhz),
            Style::default().fg(theme.fg),
        )),
    ];

    if gpu.core_count > 0 {
        lines.push(Line::from(Span::styled(
            format!("  ◈ Cores: {}", gpu.core_count),
            Style::default().fg(theme.fg),
        )));
    }

    if gpu.tflops > 0.0 {
        lines.push(Line::from(Span::styled(
            format!("  ◈ Perf:  {:.2} TFLOPs", gpu.tflops),
            Style::default().fg(theme.accent),
        )));
    }

    if gpu.sram_power_w > 0.0 {
        lines.push(Line::from(Span::styled(
            format!("  ◈ SRAM:  {:.2}W", gpu.sram_power_w),
            Style::default().fg(theme.fg),
        )));
    }

    let block = Paragraph::new(lines).style(base_style(theme)).block(
        Block::default()
            .title(" ◈ GPU ")
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(Style::default().fg(color).add_modifier(Modifier::BOLD))
            .style(Style::default().bg(theme.bg)),
    );

    f.render_widget(block, area);
}

/// Render ANE (Apple Neural Engine) usage panel
pub fn render_ane_stats(f: &mut Frame, area: Rect, data: &SystemData, theme: &Theme) {
    let ane = &data.ane_info;
    let color = if ane.usage_percentage > 80.0 {
        theme.warning_color
    } else {
        theme.mem_color
    };

    let bar_length = (ane.usage_percentage / 100.0 * 20.0) as usize;
    let bar = "█".repeat(bar_length);
    let empty = "░".repeat(20 - bar_length);

    let lines = vec![
        Line::from(Span::styled(
            format!("  ANE {}{} {:.1}%", bar, empty, ane.usage_percentage),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!("  Power: {:.2}W", ane.power_w),
            Style::default().fg(theme.fg),
        )),
    ];

    let block = Paragraph::new(lines).style(base_style(theme)).block(
        Block::default()
            .title(" ◈ ANE ")
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(Style::default().fg(color).add_modifier(Modifier::BOLD))
            .style(Style::default().bg(theme.bg)),
    );

    f.render_widget(block, area);
}

/// Render DRAM bandwidth panel
pub fn render_dram_stats(f: &mut Frame, area: Rect, data: &SystemData, theme: &Theme) {
    let dram = &data.dram_info;
    let format_bw = |bw: f64| -> String {
        if bw >= 1024.0 * 1024.0 * 1024.0 {
            format!("{:.2} GB/s", bw / (1024.0 * 1024.0 * 1024.0))
        } else if bw >= 1024.0 * 1024.0 {
            format!("{:.2} MB/s", bw / (1024.0 * 1024.0))
        } else if bw > 0.0 {
            format!("{:.2} KB/s", bw / 1024.0)
        } else {
            "N/A".to_string()
        }
    };

    let mut lines = vec![Line::from(Span::styled(
        format!("  ◈ Power: {:.2}W", dram.power_w),
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD),
    ))];

    if dram.total_bytes_per_sec > 0.0 {
        lines.push(Line::from(Span::styled(
            format!("  ↓ Read:  {}", format_bw(dram.read_bytes_per_sec)),
            Style::default().fg(theme.net_rx_color),
        )));
        lines.push(Line::from(Span::styled(
            format!("  ↑ Write: {}", format_bw(dram.write_bytes_per_sec)),
            Style::default().fg(theme.net_tx_color),
        )));
        lines.push(Line::from(Span::styled(
            format!("  ◈ Total: {}", format_bw(dram.total_bytes_per_sec)),
            Style::default().fg(theme.fg),
        )));
    }

    let block = Paragraph::new(lines).style(base_style(theme)).block(
        Block::default()
            .title(" ◈ DRAM ")
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(
                Style::default()
                    .fg(theme.mem_color)
                    .add_modifier(Modifier::BOLD),
            )
            .style(Style::default().bg(theme.bg)),
    );

    f.render_widget(block, area);
}

/// Render Thunderbolt device tree panel
pub fn render_thunderbolt_info(f: &mut Frame, area: Rect, data: &SystemData, theme: &Theme) {
    let tb = &data.thunderbolt_info;

    let mut lines = Vec::new();

    if tb.buses.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No Thunderbolt devices",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        for bus in &tb.buses {
            let status_icon = if bus.status.contains("Active") {
                "●"
            } else {
                "○"
            };
            lines.push(Line::from(Span::styled(
                format!("  {} {} - {}", status_icon, bus.name, bus.speed),
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )));

            for device in &bus.devices {
                lines.push(Line::from(Span::styled(
                    format!("    ├─ {}", device.name),
                    Style::default().fg(theme.fg),
                )));
            }
        }
    }

    let block = Paragraph::new(lines).style(base_style(theme)).block(
        Block::default()
            .title(" ◈ THUNDERBOLT ")
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )
            .style(Style::default().bg(theme.bg)),
    );

    f.render_widget(block, area);
}

/// Render detailed power breakdown panel
pub fn render_power_breakdown(f: &mut Frame, area: Rect, data: &SystemData, theme: &Theme) {
    let power = &data.cpu_info.power_metrics;
    let gpu = &data.gpu_info;
    let dram = &data.dram_info;

    // Calculate power bar widths (max 20 chars)
    let total = power.package_w.max(0.01);
    let cpu_pct = (power.cpu_w / total * 20.0) as usize;
    let gpu_pct = (power.gpu_w / total * 20.0) as usize;
    let ane_pct = (power.ane_w / total * 20.0) as usize;
    let dram_pct = (dram.power_w / total * 20.0) as usize;

    let lines = vec![
        Line::from(Span::styled(
            format!("  ◈ Total: {:.1}W", power.package_w),
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled(
                format!("  CPU {:.1}W ", power.cpu_w),
                Style::default().fg(theme.cpu_color),
            ),
            Span::styled(
                "█".repeat(cpu_pct.min(20)),
                Style::default().fg(theme.cpu_color),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                format!("  GPU {:.1}W ", power.gpu_w),
                Style::default().fg(theme.warning_color),
            ),
            Span::styled(
                "█".repeat(gpu_pct.min(20)),
                Style::default().fg(theme.warning_color),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                format!("  ANE {:.1}W ", power.ane_w),
                Style::default().fg(theme.mem_color),
            ),
            Span::styled(
                "█".repeat(ane_pct.min(20)),
                Style::default().fg(theme.mem_color),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                format!("  DRAM {:.1}W ", dram.power_w),
                Style::default().fg(theme.net_rx_color),
            ),
            Span::styled(
                "█".repeat(dram_pct.min(20)),
                Style::default().fg(theme.net_rx_color),
            ),
        ]),
    ];

    let block = Paragraph::new(lines).style(base_style(theme)).block(
        Block::default()
            .title(" ◈ POWER ")
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )
            .style(Style::default().bg(theme.bg)),
    );

    f.render_widget(block, area);
}

/// Render Disk I/O panel with real-time rates
pub fn render_disk_io_stats(f: &mut Frame, area: Rect, data: &SystemData, theme: &Theme) {
    let disk = &data.disk_io_info;
    let format_rate = |rate: f64| -> String {
        if rate >= 1024.0 * 1024.0 {
            format!("{:.2} MB/s", rate / (1024.0 * 1024.0))
        } else if rate >= 1024.0 {
            format!("{:.2} KB/s", rate / 1024.0)
        } else if rate > 0.0 {
            format!("{:.0} B/s", rate)
        } else {
            "0 B/s".to_string()
        }
    };

    let lines = vec![
        Line::from(Span::styled(
            format!("  ↓ Read:  {}", format_rate(disk.read_bytes_per_sec)),
            Style::default()
                .fg(theme.net_rx_color)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!("  ↑ Write: {}", format_rate(disk.write_bytes_per_sec)),
            Style::default()
                .fg(theme.net_tx_color)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("  R ops: {:.0}/s", disk.read_ops_per_sec),
            Style::default().fg(theme.fg),
        )),
        Line::from(Span::styled(
            format!("  W ops: {:.0}/s", disk.write_ops_per_sec),
            Style::default().fg(theme.fg),
        )),
    ];

    let block = Paragraph::new(lines).style(base_style(theme)).block(
        Block::default()
            .title(" ◈ DISK I/O ")
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(
                Style::default()
                    .fg(theme.mem_color)
                    .add_modifier(Modifier::BOLD),
            )
            .style(Style::default().bg(theme.bg)),
    );

    f.render_widget(block, area);
}

/// Render detailed system info panel (like mactop's Apple Silicon section)
pub fn render_system_details(f: &mut Frame, area: Rect, data: &SystemData, theme: &Theme) {
    let sys_info = &data.system_info;
    let mem_info = &data.memory_info;

    let mut lines = vec![
        Line::from(Span::styled(
            format!("  ◈ Chip:  {}", sys_info.chip_name),
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!(
                "  ◈ E-Cores: {}  P-Cores: {}",
                sys_info.e_core_count, sys_info.p_core_count
            ),
            Style::default().fg(theme.cpu_color),
        )),
    ];

    if sys_info.gpu_core_count > 0 {
        lines.push(Line::from(Span::styled(
            format!("  ◈ GPU Cores: {}", sys_info.gpu_core_count),
            Style::default().fg(theme.cpu_color),
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!(
            "  ◈ Memory: {:.1} / {:.1} GB",
            mem_info.used_memory as f64 / (1024.0 * 1024.0 * 1024.0),
            mem_info.total_memory as f64 / (1024.0 * 1024.0 * 1024.0)
        ),
        Style::default().fg(theme.mem_color),
    )));
    lines.push(Line::from(Span::styled(
        format!("  ◈ Host:  {}", sys_info.host_name),
        Style::default().fg(theme.fg),
    )));
    lines.push(Line::from(Span::styled(
        format!("  ◈ OS:    {}", sys_info.os_version),
        Style::default().fg(Color::DarkGray),
    )));

    let block = Paragraph::new(lines).style(base_style(theme)).block(
        Block::default()
            .title(" ◈ SYSTEM INFO ")
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )
            .style(Style::default().bg(theme.bg)),
    );

    f.render_widget(block, area);
}

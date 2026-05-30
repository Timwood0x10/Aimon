//! UI components for rendering specific data types
//! Contains reusable widgets and rendering functions

use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

use crate::types::*;
use crate::history::HistoryData;
use crate::config::{Config, ProcessSortBy};
use super::theme::Theme;
use super::chart;

/// Render header with system info
pub fn render_header(f: &mut Frame, area: Rect, data: &SystemData, theme: &Theme) {
    let header_text = format!(
        " {} | {} | {} | {} ",
        data.system_info.host_name,
        data.system_info.os_version,
        data.system_info.cpu_brand,
        data.system_info.cpu_arch
    );

    let header = Paragraph::new(header_text)
        .style(Style::default().fg(theme.fg).bg(theme.bg))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.accent)),
        );

    f.render_widget(header, area);
}

/// Render a CPU usage gauge
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
                .title(" CPU ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.cpu_color)),
        )
        .gauge_style(Style::default().fg(color))
        .ratio(usage as f64 / 100.0)
        .label(format!("{:.1}%", usage));

    f.render_widget(gauge, area);
}

/// Render a memory usage gauge
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
                .title(" MEMORY ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.mem_color)),
        )
        .gauge_style(Style::default().fg(color))
        .ratio(usage as f64 / 100.0)
        .label(format!("{}%", usage));

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
        Line::from(Span::styled(
            format!("Health: {:.0}%", batt.health_percentage),
            Style::default().fg(theme.fg),
        )),
        Line::from(Span::styled(
            format!("Cycles: {}", batt.cycle_count),
            Style::default().fg(theme.fg),
        )),
        Line::from(Span::styled(
            format!("{}W", batt.power_adapter_wattage),
            Style::default().fg(theme.fg),
        )),
    ];

    let block = Paragraph::new(lines).block(
        Block::default()
            .title(" BATTERY ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.battery_color)),
    );

    f.render_widget(block, area);
}

/// Render power consumption stats
pub fn render_power_stats(f: &mut Frame, area: Rect, data: &SystemData, theme: &Theme) {
    let power = &data.cpu_info.power_metrics;
    let lines = vec![
        Line::from(Span::styled(
            format!("Total: {:.1}W", power.package_w),
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!("CPU: {:.1}W", power.cpu_w),
            Style::default().fg(theme.cpu_color),
        )),
        Line::from(Span::styled(
            format!("GPU: {:.1}W", power.gpu_w),
            Style::default().fg(theme.warning_color),
        )),
        Line::from(Span::styled(
            format!("ANE: {:.1}W", power.ane_w),
            Style::default().fg(theme.mem_color),
        )),
    ];

    let block = Paragraph::new(lines).block(
        Block::default()
            .title(" POWER ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.accent)),
    );

    f.render_widget(block, area);
}

/// Render process list
pub fn render_process_list(f: &mut Frame, area: Rect, data: &SystemData, config: &Config, theme: &Theme) {
    // 使用引用而非克隆，避免每次渲染都克隆整个列表
    let mut indices: Vec<usize> = (0..data.process_info.len()).collect();
    
    // 根据配置排序
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
            indices.sort_by(|&a, &b| {
                data.process_info[a]
                    .name
                    .cmp(&data.process_info[b].name)
            });
        }
    }
    
    // 只取前8个进程
    indices.truncate(8);

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
                    format!("{:>6} ", p.pid),
                    Style::default().fg(Color::DarkGray),
                ),
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
                    Style::default().fg(theme.fg),
                ),
            ])
        })
        .collect();

    let header = Line::from(vec![
        Span::styled("  PID  ", Style::default().fg(Color::DarkGray)),
        Span::styled(" CPU%  ", Style::default().fg(Color::DarkGray)),
        Span::styled("  MEM  ", Style::default().fg(Color::DarkGray)),
        Span::styled("NAME", Style::default().fg(Color::DarkGray)),
    ]);

    let mut all_lines = vec![header];
    all_lines.extend(lines);

    let block = Paragraph::new(all_lines).block(
        Block::default()
            .title(" PROCESSES ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    f.render_widget(block, area);
}

/// Render network statistics
pub fn render_network_stats(f: &mut Frame, area: Rect, data: &SystemData, history: &HistoryData, theme: &Theme) {
    let total_rx: u64 = data.network_info.iter().map(|n| n.bytes_received).sum();
    let total_tx: u64 = data.network_info.iter().map(|n| n.bytes_transmitted).sum();
    
    // 获取实时速率
    let rx_rate = history.get_network_rx_rate();
    let tx_rate = history.get_network_tx_rate();

    // 格式化速率显示
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
        Line::from(Span::styled(
            format!("↓ {:.2} GB", total_rx as f64 / 1024.0 / 1024.0 / 1024.0),
            Style::default().fg(theme.net_rx_color),
        )),
        Line::from(Span::styled(
            format!("↑ {:.2} GB", total_tx as f64 / 1024.0 / 1024.0 / 1024.0),
            Style::default().fg(theme.net_tx_color),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("↓ {}", format_rate(rx_rate)),
            Style::default().fg(theme.net_rx_color),
        )),
        Line::from(Span::styled(
            format!("↑ {}", format_rate(tx_rate)),
            Style::default().fg(theme.net_tx_color),
        )),
    ];

    let block = Paragraph::new(lines).block(
        Block::default()
            .title(" NETWORK ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.net_rx_color)),
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
        Line::from(Span::styled(
            format!("Pressure: {}%", thermal.thermal_pressure),
            Style::default().fg(pressure_color),
        )),
        Line::from(Span::styled(
            format!("Throttle: {}", if thermal.thermal_throttling { "YES" } else { "NO" }),
            Style::default().fg(if thermal.thermal_throttling {
                theme.critical_color
            } else {
                theme.fg
            }),
        )),
        Line::from(Span::styled(
            format!("Fans: {} RPM", thermal.fan_speeds.iter().max().unwrap_or(&0)),
            Style::default().fg(theme.fg),
        )),
    ];

    let block = Paragraph::new(lines).block(
        Block::default()
            .title(" THERMAL ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.temp_color)),
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

/// Render CPU core usage bar chart
pub fn render_cpu_cores_bar_chart(f: &mut Frame, area: Rect, data: &SystemData, theme: &Theme) {
    let core_usages = &data.cpu_info.core_usages;
    if core_usages.is_empty() {
        return;
    }

    let num_cores = core_usages.len();
    let max_cores_per_row = 4;
    let _rows = (num_cores + max_cores_per_row - 1) / max_cores_per_row;
    
    // Create lines for each core
    let mut lines = Vec::new();
    
    for (i, &usage) in core_usages.iter().enumerate() {
        let core_color = get_cpu_usage_color(usage, theme);
        let bar_length = (usage / 100.0 * 20.0) as usize;
        let bar = "█".repeat(bar_length);
        let empty = "░".repeat(20 - bar_length);
        
        let line = Line::from(vec![
            Span::styled(
                format!("C{:02} ", i),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                format!("{}{}", bar, empty),
                Style::default().fg(core_color),
            ),
            Span::styled(
                format!(" {:5.1}%", usage),
                Style::default().fg(theme.fg),
            ),
        ]);
        lines.push(line);
    }

    let block = Paragraph::new(lines).block(
        Block::default()
            .title(" CPU CORES ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.cpu_color)),
    );

    f.render_widget(block, area);
}

/// Render network sparkline
pub fn render_network_sparkline(f: &mut Frame, area: Rect, history: &HistoryData, theme: &Theme) {
    let config = chart::SparklineConfig::new("NET RX", theme.net_rx_color);
    chart::render_sparkline(f, area, &history.network_rx_history, &config);
}
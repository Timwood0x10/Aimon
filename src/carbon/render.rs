use super::tracker::CarbonTracker;
use crate::types::{ProcessInfo, SystemData};
use crate::ui::theme::Theme;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Render the carbon tracking panel
pub fn render_carbon_panel(f: &mut Frame, area: Rect, tracker: &CarbonTracker, theme: &Theme) {
    let equivalents = tracker.get_equivalents();

    let power_w = if tracker.recording_count == 0 {
        0.0
    } else {
        tracker.total_energy_wh * 3600.0 / tracker.recording_count as f64
    };

    let lines = vec![
        Line::from(Span::styled(
            format!("Energy  {:>8.3} Wh", tracker.total_energy_wh),
            Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!("CO₂     {:>8.4} kg", tracker.carbon_kg),
            Style::default().fg(theme.fg),
        )),
        Line::from(Span::styled(
            format!("kWh     {:>8.5}", tracker.total_energy_kwh()),
            Style::default().fg(theme.fg),
        )),
        Line::from(Span::styled(
            format!("Avg W   {:>8.2}", power_w),
            Style::default().fg(Color::Gray),
        )),
        Line::from(Span::styled(
            format!("Phone   {:>8.2} charges", equivalents.phone_charges),
            Style::default().fg(theme.fg),
        )),
        Line::from(Span::styled(
            format!("Bulb    {:>8.2} h", equivalents.lightbulb_hours),
            Style::default().fg(Color::Gray),
        )),
    ];

    let block = Paragraph::new(lines)
        .style(Style::default().fg(theme.fg).bg(theme.bg))
        .block(
            Block::default()
                .title(" CARBON LEDGER ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border_color))
                .style(Style::default().bg(theme.bg)),
        );

    f.render_widget(block, area);
}

pub fn render_efficiency_advisor(f: &mut Frame, area: Rect, data: &SystemData, theme: &Theme) {
    let current_power_w = data.cpu_info.power_metrics.package_w;
    let cpu_usage = data.cpu_info.average_usage;
    let top_process = data
        .process_info
        .iter()
        .max_by(|a, b| a.cpu_usage.total_cmp(&b.cpu_usage));
    let advice = build_efficiency_advice(current_power_w, cpu_usage, top_process);

    let mut lines = vec![
        Line::from(Span::styled(
            format!("Package {:>6.2}W  CPU {:>5.1}%", current_power_w, cpu_usage),
            Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    lines.extend(advice.into_iter().map(|line| {
        Line::from(Span::styled(
            line,
            Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
        ))
    }));

    let block = Paragraph::new(lines)
        .style(Style::default().fg(theme.fg).bg(theme.bg))
        .block(
            Block::default()
                .title(" EFFICIENCY ADVISOR ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border_color))
                .style(Style::default().bg(theme.bg)),
        );

    f.render_widget(block, area);
}

fn build_efficiency_advice(
    current_power_w: f64,
    cpu_usage: f32,
    top_process: Option<&ProcessInfo>,
) -> Vec<String> {
    let mut advice = Vec::new();

    if current_power_w >= 18.0 && cpu_usage < 25.0 {
        advice.push("! High idle power detected".to_string());
        advice.push("  Check background GPU / media tasks".to_string());
    } else if current_power_w >= 25.0 {
        advice.push("! Heavy package power draw".to_string());
        advice.push("  Consider unplugging unused devices".to_string());
    } else if current_power_w > 0.0 && current_power_w <= 8.0 {
        advice.push("✓ Efficient run profile".to_string());
    } else {
        advice.push("· Building baseline...".to_string());
    }

    if let Some(process) = top_process.filter(|process| process.cpu_usage >= 15.0) {
        advice.push(format!(
            "Top CPU: {} ({:.1}%)",
            truncate_process_name(&process.name),
            process.cpu_usage
        ));
    }

    advice.truncate(5);
    advice
}

fn truncate_process_name(name: &str) -> String {
    const MAX_LEN: usize = 18;
    if name.chars().count() <= MAX_LEN {
        return name.to_string();
    }

    let mut truncated = name.chars().take(MAX_LEN - 1).collect::<String>();
    truncated.push('…');
    truncated
}

#[cfg(test)]
mod tests {
    use super::*;
    use sysinfo::Pid;

    #[test]
    fn detects_high_idle_power() {
        let advice = build_efficiency_advice(20.0, 10.0, None);
        assert!(advice[0].contains("High idle power"));
    }

    #[test]
    fn includes_top_cpu_process() {
        let process = ProcessInfo {
            pid: Pid::from_u32(42),
            name: "very-long-render-process-name".to_string(),
            cpu_usage: 42.0,
            memory_usage: 0,
            disk_read_bytes: 0,
            disk_write_bytes: 0,
        };

        let advice = build_efficiency_advice(12.0, 50.0, Some(&process));
        assert!(advice.iter().any(|line| line.contains("Top CPU")));
        assert!(advice.iter().any(|line| line.contains('…')));
    }
}

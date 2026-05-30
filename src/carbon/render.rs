use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use super::tracker::CarbonTracker;
use crate::ui::theme::Theme;

/// Render the carbon tracking panel
pub fn render_carbon_panel(
    f: &mut Frame,
    area: Rect,
    tracker: &CarbonTracker,
    theme: &Theme,
) {
    let equivalents = tracker.get_equivalents();

    let lines = vec![
        Line::from(Span::styled(
            format!("Energy: {:.3} Wh", tracker.total_energy_wh),
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!("CO2: {:.4} kg", tracker.carbon_kg),
            Style::default().fg(theme.fg),
        )),
        Line::from(Span::styled(
            format!("={:.2} kWh", tracker.total_energy_kwh()),
            Style::default().fg(theme.fg),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Equivalents:",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!("Phone charges: {:.1}", equivalents.phone_charges),
            Style::default().fg(theme.battery_color),
        )),
        Line::from(Span::styled(
            format!("60W bulb: {:.1} hrs", equivalents.lightbulb_hours),
            Style::default().fg(theme.warning_color),
        )),
        Line::from(""),
        Line::from(Span::styled(
            &equivalents.description,
            Style::default().fg(theme.fg),
        )),
    ];

    let block = Paragraph::new(lines).block(
        Block::default()
            .title(" CARBON ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.mem_color)),
    );

    f.render_widget(block, area);
}

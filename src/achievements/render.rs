use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use super::definitions::all_achievements;
use super::tracker::AchievementTracker;
use crate::ui::theme::Theme;

/// Render the achievements panel showing unlocked/locked achievements
pub fn render_achievements_panel(
    f: &mut Frame,
    area: Rect,
    tracker: &AchievementTracker,
    theme: &Theme,
) {
    let defs = all_achievements();
    let mut lines = Vec::new();

    // Header with progress
    lines.push(Line::from(Span::styled(
        format!(
            " {}/{} Unlocked",
            tracker.unlocked_count(),
            tracker.total_count()
        ),
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    for def in &defs {
        let unlocked = tracker.is_unlocked(def.id);
        let count = tracker.get_progress_count(def.id);

        let (status_icon, name_style) = if unlocked {
            ("[*]", Style::default().fg(theme.battery_color).add_modifier(Modifier::BOLD))
        } else {
            ("[ ]", Style::default().fg(Color::DarkGray))
        };

        let mut spans = vec![
            Span::styled(
                format!("{} ", status_icon),
                name_style,
            ),
            Span::styled(
                def.name.to_string(),
                name_style,
            ),
        ];

        if unlocked && count > 1 {
            spans.push(Span::styled(
                format!(" (x{})", count),
                Style::default().fg(theme.fg),
            ));
        }

        lines.push(Line::from(spans));

        // Show description for unlocked achievements
        if unlocked {
            lines.push(Line::from(Span::styled(
                format!("   {}", def.description),
                Style::default().fg(theme.fg),
            )));
        }
    }

    let title = format!(" ACHIEVEMENTS ({}/{}) ", tracker.unlocked_count(), tracker.total_count());
    let block = Paragraph::new(lines).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.accent)),
    );

    f.render_widget(block, area);
}

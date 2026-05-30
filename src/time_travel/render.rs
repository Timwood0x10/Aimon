use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use super::replay::TimeTravel;
use crate::ui::theme::Theme;

/// Render the time travel panel with timeline bar and snapshot data
pub fn render_time_travel(
    f: &mut Frame,
    area: Rect,
    time_travel: &TimeTravel,
    theme: &Theme,
) {
    if !time_travel.active {
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Timeline bar
            Constraint::Length(7), // Snapshot data
            Constraint::Length(2), // Navigation hints
        ])
        .split(area);

    render_timeline_bar(f, chunks[0], time_travel, theme);
    render_snapshot_data(f, chunks[1], time_travel, theme);
    render_navigation_hints(f, chunks[2], theme);
}

/// Render the timeline bar showing position in history
fn render_timeline_bar(
    f: &mut Frame,
    area: Rect,
    time_travel: &TimeTravel,
    theme: &Theme,
) {
    let count = time_travel.snapshot_count();
    if count == 0 {
        let empty = Paragraph::new("No snapshots available")
            .block(
                Block::default()
                    .title(" TIMELINE ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.accent)),
            );
        f.render_widget(empty, area);
        return;
    }

    let width = area.width.saturating_sub(2) as usize; // subtract borders
    let pos = time_travel.position;

    // Build timeline string
    let bar_width = width.saturating_sub(10); // leave room for position text
    let filled = if count > 1 {
        (pos * bar_width) / (count - 1)
    } else {
        bar_width
    };

    let bar: String = (0..bar_width)
        .map(|i| {
            // Mark event positions
            let snap_idx = if count > 1 {
                (i * (count - 1)) / bar_width
            } else {
                0
            };
            if i == filled {
                'O' // Current position
            } else if time_travel.snapshots.get(snap_idx)
                .and_then(|s| s.event_label.as_ref())
                .is_some()
            {
                '!' // Event marker
            } else {
                '-'
            }
        })
        .collect();

    let line = Line::from(vec![
        Span::styled(
            format!("[{}]", bar),
            Style::default().fg(theme.accent),
        ),
        Span::styled(
            format!(" {}/{}", pos + 1, count),
            Style::default().fg(theme.fg),
        ),
    ]);

    let block = Paragraph::new(line).block(
        Block::default()
            .title(" TIMELINE ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.accent)),
    );

    f.render_widget(block, area);
}

/// Render the current snapshot data
fn render_snapshot_data(
    f: &mut Frame,
    area: Rect,
    time_travel: &TimeTravel,
    theme: &Theme,
) {
    let Some(snapshot) = time_travel.current_snapshot() else {
        let empty = Paragraph::new("No snapshot selected")
            .block(
                Block::default()
                    .title(" SNAPSHOT ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.accent)),
            );
        f.render_widget(empty, area);
        return;
    };

    let mut lines = vec![
        Line::from(Span::styled(
            format!("Time: {}", snapshot.timestamp),
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled(
                format!("CPU: {:.1}%  ", snapshot.cpu_avg),
                Style::default().fg(theme.cpu_color),
            ),
            Span::styled(
                format!("MEM: {:.0}%", snapshot.memory_pct),
                Style::default().fg(theme.mem_color),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                format!("NET: {:.1} KB/s  ", snapshot.network_rx_rate / 1024.0),
                Style::default().fg(theme.net_rx_color),
            ),
            Span::styled(
                format!("TEMP: {:.0}C", snapshot.temperature_avg),
                Style::default().fg(theme.temp_color),
            ),
        ]),
        Line::from(Span::styled(
            format!(
                "Battery: {:.0}%  Thermal: {}%",
                snapshot.battery_pct, snapshot.thermal_pressure
            ),
            Style::default().fg(theme.fg),
        )),
    ];

    // Show event label if present
    if let Some(ref event) = snapshot.event_label {
        lines.push(Line::from(Span::styled(
            format!("EVENT: {}", event),
            Style::default()
                .fg(theme.warning_color)
                .add_modifier(Modifier::BOLD),
        )));
    }

    let block = Paragraph::new(lines).block(
        Block::default()
            .title(" SNAPSHOT ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.accent)),
    );

    f.render_widget(block, area);
}

/// Render navigation hints
fn render_navigation_hints(f: &mut Frame, area: Rect, theme: &Theme) {
    let line = Line::from(Span::styled(
        " [<-] Back  [->] Forward  [Home] Start  [End] Latest  [T] Toggle",
        Style::default().fg(theme.fg),
    ));

    let block = Paragraph::new(line).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.accent)),
    );

    f.render_widget(block, area);
}

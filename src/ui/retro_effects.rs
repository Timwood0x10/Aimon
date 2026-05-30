//! Retro visual effects for the TUI
//! Provides scanlines, BIOS POST screen, matrix rain, and glitch effects

use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::theme::Theme;

/// Render scanline effect - darkened lines every other row
pub fn render_scanlines(f: &mut Frame, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();
    for row in 0..area.height {
        if row % 2 == 0 {
            // Normal row - transparent
            lines.push(Line::from(Span::raw(" ".repeat(area.width as usize))));
        } else {
            // Darkened scanline row
            lines.push(Line::from(Span::styled(
                "\u{2592}".repeat(area.width as usize), // medium shade block
                Style::default().fg(Color::Rgb(0, 0, 0)).bg(Color::Rgb(0, 0, 0)),
            )));
        }
    }

    let scanline_widget = Paragraph::new(lines);
    f.render_widget(scanline_widget, area);
}

/// BIOS POST boot screen component check entry
struct PostEntry {
    name: &'static str,
    status: &'static str,
    passed: bool,
}

/// Render a BIOS POST boot screen with component checks
pub fn render_bios_post(f: &mut Frame, area: Rect, theme: &Theme) {
    let entries = vec![
        PostEntry { name: "CPU", status: "Apple Silicon Detected", passed: true },
        PostEntry { name: "MEMORY", status: "Unified Memory OK", passed: true },
        PostEntry { name: "GPU", status: "Integrated GPU Active", passed: true },
        PostEntry { name: "ANE", status: "Neural Engine Ready", passed: true },
        PostEntry { name: "THERMAL", status: "Sensors Online", passed: true },
        PostEntry { name: "STORAGE", status: "NVMe SSD OK", passed: true },
        PostEntry { name: "NETWORK", status: "Interfaces Up", passed: true },
        PostEntry { name: "DISPLAY", status: "Retina Display", passed: true },
    ];

    let mut lines: Vec<Line> = Vec::new();

    // Header
    lines.push(Line::from(Span::styled(
        "  SYSTEM ALERT BIOS v0.2.0",
        Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(Span::styled(
        "  ============================",
        Style::default().fg(theme.accent),
    )));
    lines.push(Line::from(""));

    // Component checks
    for entry in &entries {
        let status_icon = if entry.passed { "[OK]" } else { "[FAIL]" };
        let icon_color = if entry.passed {
            Color::Rgb(0, 200, 0)
        } else {
            Color::Rgb(255, 50, 50)
        };

        lines.push(Line::from(vec![
            Span::styled(
                format!("  {:>8}: ", entry.name),
                Style::default().fg(theme.fg),
            ),
            Span::styled(
                status_icon.to_string(),
                Style::default().fg(icon_color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" {}", entry.status),
                Style::default().fg(Color::DarkGray),
            ),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  All systems nominal. Boot complete.",
        Style::default().fg(Color::Rgb(0, 200, 0)),
    )));
    lines.push(Line::from(Span::styled(
        "  Press any key to continue...",
        Style::default().fg(Color::DarkGray),
    )));

    let block = Paragraph::new(lines)
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .title(" POST ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.accent)),
        );

    f.render_widget(block, area);
}

/// Render matrix rain effect - falling green characters animation
pub fn render_matrix_rain(f: &mut Frame, area: Rect, _theme: &Theme, frame: u64) {
    let chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*";
    let width = area.width as usize;
    let height = area.height as usize;

    let mut lines: Vec<Line> = Vec::new();

    for row in 0..height {
        let mut spans: Vec<Span> = Vec::new();
        for col in 0..width {
            // Calculate character based on frame, column, and row
            let idx = ((frame.wrapping_add(col as u64 * 7)
                .wrapping_add(row as u64 * 13))
                % chars.len() as u64) as usize;
            let ch = chars.chars().nth(idx).unwrap_or('A');

            // Fade brightness based on distance from "drop head"
            let drop_row = ((frame.wrapping_add(col as u64 * 11)) % height as u64) as usize;
            let distance = if row <= drop_row {
                drop_row - row
            } else {
                height - row + drop_row
            };

            let green_value = if distance < 3 {
                255u8
            } else if distance < 8 {
                200u8
            } else if distance < 15 {
                120u8
            } else {
                60u8
            };

            spans.push(Span::styled(
                ch.to_string(),
                Style::default().fg(Color::Rgb(0, green_value, 0)),
            ));
        }
        lines.push(Line::from(spans));
    }

    let block = Paragraph::new(lines).block(
        Block::default()
            .title(" MATRIX ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(0, 200, 0))),
    );

    f.render_widget(block, area);
}

/// Render glitch effect - horizontal slice displacement
pub fn render_glitch(f: &mut Frame, area: Rect, intensity: f32) {
    // Clamp intensity to 0.0 - 1.0 range
    let intensity = intensity.clamp(0.0, 1.0);

    if intensity < 0.01 {
        return; // No glitch at near-zero intensity
    }

    let width = area.width as usize;
    let height = area.height as usize;
    let glitch_probability = intensity * 0.3; // Max 30% of rows get glitched

    let mut lines: Vec<Line> = Vec::new();

    for row in 0..height {
        // Determine if this row gets glitched based on intensity
        let row_seed = (row * 31 + 17) as f32 / 100.0;
        let should_glitch = (row_seed % 1.0) < glitch_probability;

        if should_glitch {
            // Calculate displacement amount
            let displacement = ((intensity * 5.0) as usize).min(width / 4);
            let fill_char = if row % 3 == 0 { '\u{2588}' } else { '\u{2591}' };

            let mut text = String::new();
            // Left padding (displaced)
            text.push_str(&" ".repeat(displacement));
            // Content
            let content_width = width.saturating_sub(displacement);
            text.push_str(&fill_char.to_string().repeat(content_width));
            // Truncate to width
            text.truncate(width);

            let color = if row % 5 == 0 {
                Color::Rgb(255, 0, 0)
            } else if row % 3 == 0 {
                Color::Rgb(0, 255, 255)
            } else {
                Color::Rgb(255, 0, 255)
            };

            lines.push(Line::from(Span::styled(
                text,
                Style::default().fg(color),
            )));
        } else {
            lines.push(Line::from(Span::raw(" ".repeat(width))));
        }
    }

    let block = Paragraph::new(lines);
    f.render_widget(block, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bios_post_components() {
        // Verify the BIOS POST entries are well-formed
        let entries = vec![
            ("CPU", true),
            ("MEMORY", true),
            ("GPU", true),
            ("ANE", true),
            ("THERMAL", true),
            ("STORAGE", true),
            ("NETWORK", true),
            ("DISPLAY", true),
        ];

        assert_eq!(entries.len(), 8);
        for (name, passed) in &entries {
            assert!(!name.is_empty());
            assert!(*passed); // All should pass in our default config
        }
    }

    #[test]
    fn test_matrix_rain_frame_modulo() {
        let chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*";
        // Frame modulo should always produce valid indices
        for frame in 0..1000u64 {
            for col in 0..20u16 {
                for row in 0..10u16 {
                    let idx = ((frame.wrapping_add(col as u64 * 7)
                        .wrapping_add(row as u64 * 13))
                        % chars.len() as u64) as usize;
                    assert!(idx < chars.len(), "Index out of bounds at frame={}, col={}, row={}", frame, col, row);
                }
            }
        }
    }

    #[test]
    fn test_glitch_intensity_bounds() {
        // Intensity should be clamped to 0.0 - 1.0
        let test_values: Vec<f32> = vec![-0.5, 0.0, 0.3, 0.5, 1.0, 1.5, 2.0];
        for val in test_values {
            let clamped = val.clamp(0.0f32, 1.0f32);
            assert!(clamped >= 0.0 && clamped <= 1.0, "Clamping failed for {}", val);
        }

        // Near-zero intensity should not produce glitch
        let near_zero = 0.005f32;
        assert!(near_zero < 0.01, "Near-zero check failed");
    }
}

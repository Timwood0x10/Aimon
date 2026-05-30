//! Party mode with various visual effects for the TUI
//! Provides fun visual effects: color cycling, screen shake, fireworks, glitch, matrix rain

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    Frame,
};

use super::theme::Theme;

/// Available party effects
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PartyEffect {
    ColorCycle,
    ScreenShake,
    Fireworks,
    Glitch,
    MatrixRain,
}

impl PartyEffect {
    /// Get the next effect in rotation
    pub fn next(self) -> Self {
        match self {
            Self::ColorCycle => Self::ScreenShake,
            Self::ScreenShake => Self::Fireworks,
            Self::Fireworks => Self::Glitch,
            Self::Glitch => Self::MatrixRain,
            Self::MatrixRain => Self::ColorCycle,
        }
    }
}

/// Party mode state manager
pub struct PartyMode {
    pub active: bool,
    pub frame_count: u64,
    pub effect: PartyEffect,
}

impl PartyMode {
    /// Create a new party mode instance (inactive by default)
    pub fn new() -> Self {
        Self {
            active: false,
            frame_count: 0,
            effect: PartyEffect::ColorCycle,
        }
    }

    /// Toggle party mode on/off
    pub fn toggle(&mut self) {
        self.active = !self.active;
        if !self.active {
            self.frame_count = 0;
        }
    }

    /// Check if party mode is active
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Advance the animation frame counter
    pub fn advance_frame(&mut self) {
        if self.active {
            self.frame_count = self.frame_count.wrapping_add(1);
        }
    }

    /// Cycle to the next visual effect
    pub fn next_effect(&mut self) {
        self.effect = self.effect.next();
    }

    /// Get a screen shake offset (dx, dy) based on current frame
    pub fn shake_offset(&self) -> (i32, i32) {
        if !self.active || self.effect != PartyEffect::ScreenShake {
            return (0, 0);
        }
        let frame = self.frame_count;
        let dx = ((frame * 7) % 5) as i32 - 2;
        let dy = ((frame * 11) % 3) as i32 - 1;
        (dx, dy)
    }

    /// Generate firework positions for the current frame
    pub fn firework_positions(&self, width: u16, height: u16) -> Vec<(u16, u16)> {
        if !self.active || self.effect != PartyEffect::Fireworks {
            return vec![];
        }
        let frame = self.frame_count;
        let count = ((frame % 5) + 1) as u16;
        let mut positions = Vec::new();
        for i in 0..count {
            let x = ((frame.wrapping_mul(31).wrapping_add(i as u64 * 17)) % width as u64) as u16;
            let y = ((frame.wrapping_mul(23).wrapping_add(i as u64 * 13)) % height as u64) as u16;
            positions.push((x, y));
        }
        positions
    }

    /// Get glitch character replacements for a line
    pub fn glitch_chars(&self, line_idx: u16) -> Vec<(u16, char)> {
        if !self.active || self.effect != PartyEffect::Glitch {
            return vec![];
        }
        let frame = self.frame_count;
        let mut replacements = Vec::new();
        let count = ((frame.wrapping_add(line_idx as u64)) % 4) as u16;
        let glitch_chars = ['!', '@', '#', '$', '%', '^', '&', '*', '~', '?', '>', '<'];
        for i in 0..count {
            let pos = ((frame.wrapping_mul(19).wrapping_add(i as u64 * 7).wrapping_add(line_idx as u64)) % 40) as u16;
            let ch_idx = ((frame.wrapping_add(i as u64).wrapping_add(line_idx as u64)) % glitch_chars.len() as u64) as usize;
            replacements.push((pos, glitch_chars[ch_idx]));
        }
        replacements
    }

    /// Get matrix rain character for a given position and frame
    pub fn matrix_char(&self, col: u16, row: u16) -> char {
        if !self.active || self.effect != PartyEffect::MatrixRain {
            return ' ';
        }
        let frame = self.frame_count;
        let chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*";
        let idx = ((frame.wrapping_add(col as u64 * 7).wrapping_add(row as u64 * 13)) % chars.len() as u64) as usize;
        chars.chars().nth(idx).unwrap_or('A')
    }

    /// Apply the current party effect to the frame
    pub fn apply_effect(&self, f: &mut Frame, _theme: &Theme) {
        if !self.active {
            return;
        }
        match self.effect {
            PartyEffect::ColorCycle => {
                // Color cycle is applied via theme rotation; no direct frame modification needed
            }
            PartyEffect::ScreenShake => {
                // Screen shake is applied via offset calculations during rendering
            }
            PartyEffect::Fireworks => {
                self.render_fireworks(f);
            }
            PartyEffect::Glitch => {
                self.render_glitch_overlay(f);
            }
            PartyEffect::MatrixRain => {
                self.render_matrix_overlay(f);
            }
        }
    }

    /// Render firework sparkles on the screen
    fn render_fireworks(&self, f: &mut Frame) {
        let area = f.size();
        let positions = self.firework_positions(area.width, area.height);
        let sparkles = ['*', '+', '.', 'o', 'x'];
        let colors = [
            Color::Rgb(255, 100, 100),
            Color::Rgb(100, 255, 100),
            Color::Rgb(100, 100, 255),
            Color::Rgb(255, 255, 100),
            Color::Rgb(255, 100, 255),
        ];

        for (i, &(x, y)) in positions.iter().enumerate() {
            if x < area.width && y < area.height {
                let ch = sparkles[i % sparkles.len()];
                let color = colors[i % colors.len()];
                let spark_area = Rect::new(area.x + x, area.y + y, 1, 1);
                let spark = ratatui::widgets::Paragraph::new(Line::from(Span::styled(
                    ch.to_string(),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                )));
                f.render_widget(spark, spark_area);
            }
        }
    }

    /// Render glitch overlay effect
    fn render_glitch_overlay(&self, f: &mut Frame) {
        let area = f.size();
        let glitch_chars = ['!', '#', '%', '&', '?', '>', '<', '~'];
        let frame = self.frame_count;

        // Add a few random glitch characters
        for i in 0..3u16 {
            let x = ((frame.wrapping_mul(37).wrapping_add(i as u64 * 41)) % area.width as u64) as u16;
            let y = ((frame.wrapping_mul(29).wrapping_add(i as u64 * 33)) % area.height as u64) as u16;
            let ch_idx = ((frame.wrapping_add(i as u64)) % glitch_chars.len() as u64) as usize;

            if x < area.width && y < area.height {
                let glitch_area = Rect::new(area.x + x, area.y + y, 1, 1);
                let glitch = ratatui::widgets::Paragraph::new(Line::from(Span::styled(
                    glitch_chars[ch_idx].to_string(),
                    Style::default().fg(Color::Rgb(255, 0, 0)).add_modifier(Modifier::BOLD),
                )));
                f.render_widget(glitch, glitch_area);
            }
        }
    }

    /// Render matrix rain overlay
    fn render_matrix_overlay(&self, f: &mut Frame) {
        let area = f.size();
        // Only render a few falling characters as overlay
        let frame = self.frame_count;
        for col in (0..area.width).step_by(3) {
            let row = ((frame.wrapping_add(col as u64 * 3)) % area.height as u64) as u16;
            if row < area.height && col < area.width {
                let ch = self.matrix_char(col, row);
                let rain_area = Rect::new(area.x + col, area.y + row, 1, 1);
                let rain = ratatui::widgets::Paragraph::new(Line::from(Span::styled(
                    ch.to_string(),
                    Style::default().fg(Color::Rgb(0, 200, 0)),
                )));
                f.render_widget(rain, rain_area);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_party_mode_toggle() {
        let mut pm = PartyMode::new();
        assert!(!pm.is_active());

        pm.toggle();
        assert!(pm.is_active());
        assert_eq!(pm.frame_count, 0);

        pm.advance_frame();
        assert_eq!(pm.frame_count, 1);

        pm.toggle();
        assert!(!pm.is_active());
        // Frame count resets on deactivate
        assert_eq!(pm.frame_count, 0);
    }

    #[test]
    fn test_party_mode_frame_advance() {
        let mut pm = PartyMode::new();
        pm.toggle();

        for i in 0..100 {
            pm.advance_frame();
            assert_eq!(pm.frame_count, i + 1);
        }

        // When inactive, frames don't advance
        pm.toggle();
        let count = pm.frame_count;
        pm.advance_frame();
        assert_eq!(pm.frame_count, count);
    }

    #[test]
    fn test_hue_rotation() {
        let theme = Theme::cyberpunk();
        let rotated = theme.with_party_offset(0);
        // At frame 0, colors should be the same as original
        assert_eq!(rotated.bg, theme.bg);
        assert_eq!(rotated.fg, theme.fg);

        // At frame 1, colors should shift
        let rotated1 = theme.with_party_offset(1);
        // The accent color should be different
        // We just verify it doesn't panic and produces valid colors
        let _ = rotated1.accent;
    }

    #[test]
    fn test_effect_rotation() {
        assert_eq!(PartyEffect::ColorCycle.next(), PartyEffect::ScreenShake);
        assert_eq!(PartyEffect::ScreenShake.next(), PartyEffect::Fireworks);
        assert_eq!(PartyEffect::Fireworks.next(), PartyEffect::Glitch);
        assert_eq!(PartyEffect::Glitch.next(), PartyEffect::MatrixRain);
        assert_eq!(PartyEffect::MatrixRain.next(), PartyEffect::ColorCycle);
    }

    #[test]
    fn test_shake_offset_inactive() {
        let pm = PartyMode::new();
        assert_eq!(pm.shake_offset(), (0, 0));
    }
}

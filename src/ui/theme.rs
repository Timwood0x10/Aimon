//! Theme configuration for the UI
//! Provides color schemes and styling for different UI elements

use ratatui::style::Color;

/// Color scheme for the UI
#[derive(Debug, Clone)]
pub struct Theme {
    pub bg: Color,
    pub fg: Color,
    pub accent: Color,
    pub cpu_color: Color,
    pub mem_color: Color,
    pub temp_color: Color,
    pub net_rx_color: Color,
    pub net_tx_color: Color,
    pub battery_color: Color,
    pub warning_color: Color,
    pub critical_color: Color,
}

impl Theme {
    /// Cyberpunk neon - electric cyan, hot pink, acid green on pitch black
    pub fn cyberpunk() -> Self {
        Self {
            bg: Color::Rgb(8, 8, 16),
            fg: Color::Rgb(230, 230, 255),
            accent: Color::Rgb(0, 255, 255),
            cpu_color: Color::Rgb(0, 220, 255),
            mem_color: Color::Rgb(0, 255, 180),
            temp_color: Color::Rgb(255, 60, 100),
            net_rx_color: Color::Rgb(80, 180, 255),
            net_tx_color: Color::Rgb(255, 120, 80),
            battery_color: Color::Rgb(0, 255, 120),
            warning_color: Color::Rgb(255, 220, 0),
            critical_color: Color::Rgb(255, 40, 60),
        }
    }

    /// Nord aurora - brightened with vivid aurora borealis colors
    pub fn nord() -> Self {
        Self {
            bg: Color::Rgb(36, 42, 56),
            fg: Color::Rgb(228, 234, 246),
            accent: Color::Rgb(136, 192, 208),
            cpu_color: Color::Rgb(130, 200, 240),
            mem_color: Color::Rgb(160, 220, 140),
            temp_color: Color::Rgb(230, 100, 120),
            net_rx_color: Color::Rgb(100, 160, 230),
            net_tx_color: Color::Rgb(230, 150, 120),
            battery_color: Color::Rgb(160, 220, 140),
            warning_color: Color::Rgb(240, 210, 140),
            critical_color: Color::Rgb(230, 90, 110),
        }
    }

    /// Dracula vivid - amplified neon purple, cyan, green
    pub fn dracula() -> Self {
        Self {
            bg: Color::Rgb(32, 34, 48),
            fg: Color::Rgb(250, 250, 245),
            accent: Color::Rgb(200, 150, 255),
            cpu_color: Color::Rgb(130, 240, 255),
            mem_color: Color::Rgb(80, 255, 130),
            temp_color: Color::Rgb(255, 80, 90),
            net_rx_color: Color::Rgb(120, 130, 255),
            net_tx_color: Color::Rgb(255, 120, 200),
            battery_color: Color::Rgb(80, 255, 130),
            warning_color: Color::Rgb(250, 255, 140),
            critical_color: Color::Rgb(255, 70, 90),
        }
    }

    /// Tokyo Night - deep indigo with vivid blue and magenta accents
    pub fn tokyo_night() -> Self {
        Self {
            bg: Color::Rgb(18, 20, 32),
            fg: Color::Rgb(200, 210, 255),
            accent: Color::Rgb(120, 170, 255),
            cpu_color: Color::Rgb(100, 210, 255),
            mem_color: Color::Rgb(160, 220, 110),
            temp_color: Color::Rgb(255, 100, 140),
            net_rx_color: Color::Rgb(120, 160, 255),
            net_tx_color: Color::Rgb(255, 160, 100),
            battery_color: Color::Rgb(160, 220, 110),
            warning_color: Color::Rgb(255, 200, 100),
            critical_color: Color::Rgb(255, 100, 130),
        }
    }

    /// Monokai bright - vivid magenta, cyan, green on dark charcoal
    pub fn monokai() -> Self {
        Self {
            bg: Color::Rgb(30, 32, 28),
            fg: Color::Rgb(250, 250, 245),
            accent: Color::Rgb(255, 50, 140),
            cpu_color: Color::Rgb(100, 230, 255),
            mem_color: Color::Rgb(180, 255, 50),
            temp_color: Color::Rgb(255, 150, 30),
            net_rx_color: Color::Rgb(100, 230, 255),
            net_tx_color: Color::Rgb(255, 160, 40),
            battery_color: Color::Rgb(180, 255, 50),
            warning_color: Color::Rgb(255, 240, 120),
            critical_color: Color::Rgb(255, 50, 120),
        }
    }

    /// Solarized neon - classic solarized but with bumped-up saturation
    pub fn solarized_dark() -> Self {
        Self {
            bg: Color::Rgb(0, 36, 48),
            fg: Color::Rgb(180, 200, 210),
            accent: Color::Rgb(40, 170, 255),
            cpu_color: Color::Rgb(50, 200, 200),
            mem_color: Color::Rgb(160, 200, 0),
            temp_color: Color::Rgb(240, 90, 30),
            net_rx_color: Color::Rgb(40, 170, 255),
            net_tx_color: Color::Rgb(240, 70, 160),
            battery_color: Color::Rgb(160, 200, 0),
            warning_color: Color::Rgb(220, 170, 0),
            critical_color: Color::Rgb(250, 60, 50),
        }
    }

    /// Gruvbox neon - retro orange, green, red, maxed-out vibrancy
    pub fn gruvbox() -> Self {
        Self {
            bg: Color::Rgb(30, 30, 30),
            fg: Color::Rgb(240, 230, 190),
            accent: Color::Rgb(255, 120, 20),
            cpu_color: Color::Rgb(80, 180, 180),
            mem_color: Color::Rgb(180, 200, 40),
            temp_color: Color::Rgb(250, 50, 40),
            net_rx_color: Color::Rgb(80, 180, 180),
            net_tx_color: Color::Rgb(255, 120, 20),
            battery_color: Color::Rgb(180, 200, 40),
            warning_color: Color::Rgb(255, 190, 40),
            critical_color: Color::Rgb(250, 50, 40),
        }
    }

    /// Catppuccin Mocha - pastel but with enough contrast to pop
    pub fn catppuccin() -> Self {
        Self {
            bg: Color::Rgb(24, 24, 38),
            fg: Color::Rgb(210, 220, 250),
            accent: Color::Rgb(140, 190, 255),
            cpu_color: Color::Rgb(120, 210, 250),
            mem_color: Color::Rgb(170, 240, 170),
            temp_color: Color::Rgb(250, 140, 170),
            net_rx_color: Color::Rgb(120, 210, 250),
            net_tx_color: Color::Rgb(255, 180, 140),
            battery_color: Color::Rgb(170, 240, 170),
            warning_color: Color::Rgb(255, 230, 180),
            critical_color: Color::Rgb(250, 130, 160),
        }
    }

    /// One Dark vivid - brightened Atom theme with neon touches
    pub fn one_dark() -> Self {
        Self {
            bg: Color::Rgb(30, 34, 44),
            fg: Color::Rgb(200, 210, 230),
            accent: Color::Rgb(100, 190, 255),
            cpu_color: Color::Rgb(90, 200, 220),
            mem_color: Color::Rgb(160, 220, 130),
            temp_color: Color::Rgb(250, 110, 120),
            net_rx_color: Color::Rgb(100, 190, 255),
            net_tx_color: Color::Rgb(240, 170, 100),
            battery_color: Color::Rgb(160, 220, 130),
            warning_color: Color::Rgb(250, 210, 130),
            critical_color: Color::Rgb(250, 100, 110),
        }
    }

    /// Returns a list of all available theme names
    pub fn all_themes() -> Vec<&'static str> {
        vec![
            "cyberpunk",
            "nord",
            "dracula",
            "tokyo_night",
            "monokai",
            "solarized_dark",
            "gruvbox",
            "catppuccin",
            "one_dark",
        ]
    }

    /// Creates a theme from its name (case-insensitive)
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "cyberpunk" => Some(Self::cyberpunk()),
            "nord" => Some(Self::nord()),
            "dracula" => Some(Self::dracula()),
            "tokyo_night" | "tokyonight" => Some(Self::tokyo_night()),
            "monokai" => Some(Self::monokai()),
            "solarized_dark" | "solarized" => Some(Self::solarized_dark()),
            "gruvbox" => Some(Self::gruvbox()),
            "catppuccin" => Some(Self::catppuccin()),
            "one_dark" | "onedark" => Some(Self::one_dark()),
            _ => None,
        }
    }

    /// Create a party-mode variant with hue-rotated colors.
    pub fn with_party_offset(&self, frame: u64) -> Self {
        let shift = (frame % 360) as f32;
        Self {
            bg: self.bg,
            fg: rotate_color(self.fg, shift),
            accent: rotate_color(self.accent, shift),
            cpu_color: rotate_color(self.cpu_color, shift + 30.0),
            mem_color: rotate_color(self.mem_color, shift + 60.0),
            temp_color: rotate_color(self.temp_color, shift + 90.0),
            net_rx_color: rotate_color(self.net_rx_color, shift + 120.0),
            net_tx_color: rotate_color(self.net_tx_color, shift + 150.0),
            battery_color: rotate_color(self.battery_color, shift + 180.0),
            warning_color: rotate_color(self.warning_color, shift + 210.0),
            critical_color: rotate_color(self.critical_color, shift + 240.0),
        }
    }

    /// Returns the name of the current theme
    pub fn name(&self) -> &'static str {
        if self.bg == Color::Rgb(8, 8, 16) {
            "cyberpunk"
        } else if self.bg == Color::Rgb(36, 42, 56) {
            "nord"
        } else if self.bg == Color::Rgb(32, 34, 48) {
            "dracula"
        } else if self.bg == Color::Rgb(18, 20, 32) {
            "tokyo_night"
        } else if self.bg == Color::Rgb(30, 32, 28) {
            "monokai"
        } else if self.bg == Color::Rgb(0, 36, 48) {
            "solarized_dark"
        } else if self.bg == Color::Rgb(30, 30, 30) {
            "gruvbox"
        } else if self.bg == Color::Rgb(24, 24, 38) {
            "catppuccin"
        } else if self.bg == Color::Rgb(30, 34, 44) {
            "one_dark"
        } else {
            "unknown"
        }
    }
}

/// Rotate an RGB color's hue by the given number of degrees (0-360).
fn rotate_color(color: Color, degrees: f32) -> Color {
    let Color::Rgb(r, g, b) = color else {
        return color;
    };

    let r = r as f32 / 255.0;
    let g = g as f32 / 255.0;
    let b = b as f32 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    let s = if max == 0.0 { 0.0 } else { delta / max };
    let v = max;

    let h = if delta == 0.0 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / delta) % 6.0)
    } else if max == g {
        60.0 * (((b - r) / delta) + 2.0)
    } else {
        60.0 * (((r - g) / delta) + 4.0)
    };

    let h = ((h + degrees) % 360.0 + 360.0) % 360.0;

    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;

    let (r1, g1, b1) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    Color::Rgb(
        ((r1 + m) * 255.0) as u8,
        ((g1 + m) * 255.0) as u8,
        ((b1 + m) * 255.0) as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_themes_count() {
        let themes = Theme::all_themes();
        assert_eq!(themes.len(), 9);
    }

    #[test]
    fn test_from_name_valid_themes() {
        let test_cases = vec![
            ("cyberpunk", true),
            ("nord", true),
            ("dracula", true),
            ("tokyo_night", true),
            ("monokai", true),
            ("solarized_dark", true),
            ("gruvbox", true),
            ("catppuccin", true),
            ("one_dark", true),
            ("Cyberpunk", true),
            ("NORD", true),
            ("DrAcUlA", true),
            ("tokyonight", true),
            ("solarized", true),
            ("onedark", true),
            ("invalid", false),
            ("", false),
        ];

        for (name, should_exist) in test_cases {
            let result = Theme::from_name(name);
            assert_eq!(result.is_some(), should_exist, "Theme '{}' existence check failed", name);
        }
    }

    #[test]
    fn test_theme_name_roundtrip() {
        let themes = Theme::all_themes();
        for theme_name in themes {
            let theme = Theme::from_name(theme_name).unwrap();
            assert_eq!(theme.name(), theme_name, "Theme name roundtrip failed for '{}'", theme_name);
        }
    }

    #[test]
    fn test_default_theme_is_cyberpunk() {
        let theme = Theme::cyberpunk();
        assert_eq!(theme.name(), "cyberpunk");
        assert_eq!(theme.bg, Color::Rgb(8, 8, 16));
    }

    #[test]
    fn test_hue_rotation() {
        let theme = Theme::cyberpunk();

        let rotated = theme.with_party_offset(0);
        assert_eq!(rotated.bg, theme.bg);
        assert_eq!(rotated.fg, theme.fg);
        assert_eq!(rotated.accent, theme.accent);

        let red = Color::Rgb(255, 0, 0);
        let rotated_red = rotate_color(red, 120.0);
        if let Color::Rgb(r, g, b) = rotated_red {
            assert!(g > 200, "Expected high green component, got r={}, g={}, b={}", r, g, b);
            assert!(r < 10, "Expected low red component, got r={}, g={}, b={}", r, g, b);
        } else {
            panic!("Expected RGB color");
        }

        let rotated1 = theme.with_party_offset(1);
        assert_eq!(rotated1.bg, theme.bg);
        let _ = rotated1.accent;
    }

    #[test]
    fn test_cyberpunk_colors_are_vivid() {
        let theme = Theme::cyberpunk();
        // Accent should be pure cyan (max green and blue)
        if let Color::Rgb(r, g, b) = theme.accent {
            assert_eq!(g, 255, "Accent green should be max");
            assert_eq!(b, 255, "Accent blue should be max");
            assert_eq!(r, 0, "Accent red should be zero");
        }
        // CPU color should be bright
        if let Color::Rgb(_, g, b) = theme.cpu_color {
            assert!(g >= 200, "CPU color green should be bright");
            assert!(b >= 200, "CPU color blue should be bright");
        }
        // Critical should be clearly red
        if let Color::Rgb(r, g, b) = theme.critical_color {
            assert!(r >= 200, "Critical red should be dominant");
            assert!(g < 80, "Critical green should be low");
        }
    }
}

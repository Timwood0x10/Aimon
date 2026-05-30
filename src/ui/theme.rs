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
    /// Cyberpunk neon - electric cyan, hot pink, acid green on deep navy
    pub fn cyberpunk() -> Self {
        Self {
            bg: Color::Rgb(14, 14, 28),
            fg: Color::Rgb(235, 235, 255),
            accent: Color::Rgb(0, 255, 255),
            cpu_color: Color::Rgb(0, 220, 255),
            mem_color: Color::Rgb(0, 255, 180),
            temp_color: Color::Rgb(255, 70, 110),
            net_rx_color: Color::Rgb(90, 190, 255),
            net_tx_color: Color::Rgb(255, 130, 90),
            battery_color: Color::Rgb(0, 255, 120),
            warning_color: Color::Rgb(255, 225, 40),
            critical_color: Color::Rgb(255, 55, 70),
        }
    }

    /// Nord aurora - brightened with vivid aurora borealis colors
    pub fn nord() -> Self {
        Self {
            bg: Color::Rgb(40, 48, 64),
            fg: Color::Rgb(232, 238, 248),
            accent: Color::Rgb(140, 200, 220),
            cpu_color: Color::Rgb(136, 210, 250),
            mem_color: Color::Rgb(170, 230, 150),
            temp_color: Color::Rgb(240, 110, 130),
            net_rx_color: Color::Rgb(110, 170, 240),
            net_tx_color: Color::Rgb(240, 160, 130),
            battery_color: Color::Rgb(170, 230, 150),
            warning_color: Color::Rgb(245, 215, 145),
            critical_color: Color::Rgb(240, 100, 120),
        }
    }

    /// Dracula vivid - amplified neon purple, cyan, green
    pub fn dracula() -> Self {
        Self {
            bg: Color::Rgb(38, 40, 58),
            fg: Color::Rgb(250, 250, 250),
            accent: Color::Rgb(205, 160, 255),
            cpu_color: Color::Rgb(140, 245, 255),
            mem_color: Color::Rgb(90, 255, 145),
            temp_color: Color::Rgb(255, 95, 105),
            net_rx_color: Color::Rgb(135, 145, 255),
            net_tx_color: Color::Rgb(255, 135, 210),
            battery_color: Color::Rgb(90, 255, 145),
            warning_color: Color::Rgb(255, 255, 150),
            critical_color: Color::Rgb(255, 85, 100),
        }
    }

    /// Tokyo Night - deep indigo with vivid blue and magenta accents
    pub fn tokyo_night() -> Self {
        Self {
            bg: Color::Rgb(26, 28, 44),
            fg: Color::Rgb(215, 222, 255),
            accent: Color::Rgb(130, 180, 255),
            cpu_color: Color::Rgb(110, 218, 255),
            mem_color: Color::Rgb(170, 228, 120),
            temp_color: Color::Rgb(255, 115, 150),
            net_rx_color: Color::Rgb(130, 170, 255),
            net_tx_color: Color::Rgb(255, 170, 110),
            battery_color: Color::Rgb(170, 228, 120),
            warning_color: Color::Rgb(255, 210, 110),
            critical_color: Color::Rgb(255, 115, 140),
        }
    }

    /// Monokai bright - vivid magenta, cyan, green on warm charcoal
    pub fn monokai() -> Self {
        Self {
            bg: Color::Rgb(38, 40, 36),
            fg: Color::Rgb(250, 250, 248),
            accent: Color::Rgb(255, 70, 155),
            cpu_color: Color::Rgb(110, 235, 255),
            mem_color: Color::Rgb(190, 255, 60),
            temp_color: Color::Rgb(255, 165, 45),
            net_rx_color: Color::Rgb(110, 235, 255),
            net_tx_color: Color::Rgb(255, 175, 55),
            battery_color: Color::Rgb(190, 255, 60),
            warning_color: Color::Rgb(255, 245, 135),
            critical_color: Color::Rgb(255, 70, 135),
        }
    }

    /// Solarized neon - classic solarized but with bumped-up saturation and contrast
    pub fn solarized_dark() -> Self {
        Self {
            bg: Color::Rgb(7, 44, 58),
            fg: Color::Rgb(210, 225, 230),
            accent: Color::Rgb(55, 185, 255),
            cpu_color: Color::Rgb(65, 210, 210),
            mem_color: Color::Rgb(175, 210, 20),
            temp_color: Color::Rgb(245, 105, 45),
            net_rx_color: Color::Rgb(55, 185, 255),
            net_tx_color: Color::Rgb(245, 85, 175),
            battery_color: Color::Rgb(175, 210, 20),
            warning_color: Color::Rgb(230, 185, 15),
            critical_color: Color::Rgb(255, 75, 60),
        }
    }

    /// Gruvbox neon - retro orange, green, red, maxed-out vibrancy
    pub fn gruvbox() -> Self {
        Self {
            bg: Color::Rgb(38, 36, 32),
            fg: Color::Rgb(245, 235, 200),
            accent: Color::Rgb(255, 135, 30),
            cpu_color: Color::Rgb(95, 195, 195),
            mem_color: Color::Rgb(190, 215, 50),
            temp_color: Color::Rgb(255, 65, 50),
            net_rx_color: Color::Rgb(95, 195, 195),
            net_tx_color: Color::Rgb(255, 135, 30),
            battery_color: Color::Rgb(190, 215, 50),
            warning_color: Color::Rgb(255, 200, 50),
            critical_color: Color::Rgb(255, 65, 50),
        }
    }

    /// Catppuccin Mocha - pastel but with enough contrast to pop
    pub fn catppuccin() -> Self {
        Self {
            bg: Color::Rgb(30, 30, 48),
            fg: Color::Rgb(218, 228, 255),
            accent: Color::Rgb(150, 200, 255),
            cpu_color: Color::Rgb(130, 218, 255),
            mem_color: Color::Rgb(180, 245, 180),
            temp_color: Color::Rgb(255, 150, 180),
            net_rx_color: Color::Rgb(130, 218, 255),
            net_tx_color: Color::Rgb(255, 190, 150),
            battery_color: Color::Rgb(180, 245, 180),
            warning_color: Color::Rgb(255, 235, 190),
            critical_color: Color::Rgb(255, 145, 170),
        }
    }

    /// One Dark vivid - brightened Atom theme with neon touches
    pub fn one_dark() -> Self {
        Self {
            bg: Color::Rgb(36, 40, 54),
            fg: Color::Rgb(215, 222, 240),
            accent: Color::Rgb(115, 200, 255),
            cpu_color: Color::Rgb(105, 210, 230),
            mem_color: Color::Rgb(170, 228, 145),
            temp_color: Color::Rgb(255, 125, 135),
            net_rx_color: Color::Rgb(115, 200, 255),
            net_tx_color: Color::Rgb(245, 180, 115),
            battery_color: Color::Rgb(170, 228, 145),
            warning_color: Color::Rgb(255, 218, 145),
            critical_color: Color::Rgb(255, 115, 125),
        }
    }

    /// Matrix Neon - deep green-black with layered green/cyan/yellow for depth
    pub fn matrix_neon() -> Self {
        Self {
            bg: Color::Rgb(10, 18, 14),
            fg: Color::Rgb(195, 255, 195),
            accent: Color::Rgb(0, 255, 145),
            cpu_color: Color::Rgb(70, 255, 150),
            mem_color: Color::Rgb(20, 240, 180),
            temp_color: Color::Rgb(255, 120, 115),
            net_rx_color: Color::Rgb(90, 215, 205),
            net_tx_color: Color::Rgb(255, 208, 75),
            battery_color: Color::Rgb(20, 240, 180),
            warning_color: Color::Rgb(255, 225, 70),
            critical_color: Color::Rgb(255, 85, 100),
        }
    }

    /// Mactop Green - inspired by mactop's clean green theme with professional look
    pub fn mactop_green() -> Self {
        Self {
            bg: Color::Rgb(22, 28, 22),
            fg: Color::Rgb(195, 250, 192),
            accent: Color::Rgb(90, 195, 95),
            cpu_color: Color::Rgb(145, 215, 148),
            mem_color: Color::Rgb(120, 202, 125),
            temp_color: Color::Rgb(245, 100, 95),
            net_rx_color: Color::Rgb(95, 195, 185),
            net_tx_color: Color::Rgb(255, 198, 90),
            battery_color: Color::Rgb(120, 202, 125),
            warning_color: Color::Rgb(255, 240, 70),
            critical_color: Color::Rgb(240, 70, 65),
        }
    }

    /// Gold Pro - warm dark background with golden/amber accents for a premium look
    pub fn gold_pro() -> Self {
        Self {
            bg: Color::Rgb(26, 24, 20),
            fg: Color::Rgb(238, 230, 210),
            accent: Color::Rgb(225, 180, 45),
            cpu_color: Color::Rgb(255, 205, 25),
            mem_color: Color::Rgb(255, 172, 15),
            temp_color: Color::Rgb(255, 105, 60),
            net_rx_color: Color::Rgb(255, 222, 20),
            net_tx_color: Color::Rgb(255, 155, 10),
            battery_color: Color::Rgb(200, 155, 25),
            warning_color: Color::Rgb(255, 205, 25),
            critical_color: Color::Rgb(255, 82, 15),
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
            "matrix_neon",
            "mactop_green",
            "gold_pro",
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
            "matrix_neon" | "matrix" => Some(Self::matrix_neon()),
            "mactop_green" | "mactop" => Some(Self::mactop_green()),
            "gold_pro" | "gold" => Some(Self::gold_pro()),
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
        } else if self.bg == Color::Rgb(5, 10, 8) {
            "matrix_neon"
        } else if self.bg == Color::Rgb(16, 20, 16) {
            "mactop_green"
        } else if self.bg == Color::Rgb(18, 18, 18) {
            "gold_pro"
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
        assert_eq!(themes.len(), 12);
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
            assert_eq!(
                result.is_some(),
                should_exist,
                "Theme '{}' existence check failed",
                name
            );
        }
    }

    #[test]
    fn test_theme_name_roundtrip() {
        let themes = Theme::all_themes();
        for theme_name in themes {
            let theme = Theme::from_name(theme_name).unwrap();
            assert_eq!(
                theme.name(),
                theme_name,
                "Theme name roundtrip failed for '{}'",
                theme_name
            );
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
            assert!(
                g > 200,
                "Expected high green component, got r={}, g={}, b={}",
                r,
                g,
                b
            );
            assert!(
                r < 10,
                "Expected low red component, got r={}, g={}, b={}",
                r,
                g,
                b
            );
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
        if let Color::Rgb(r, g, _b) = theme.critical_color {
            assert!(r >= 200, "Critical red should be dominant");
            assert!(g < 80, "Critical green should be low");
        }
    }
}

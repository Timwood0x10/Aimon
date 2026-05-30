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
    /// Cyberpunk-inspired color scheme (default)
    pub fn cyberpunk() -> Self {
        Self {
            bg: Color::Rgb(15, 15, 25),
            fg: Color::Rgb(200, 200, 220),
            accent: Color::Rgb(0, 255, 255),
            cpu_color: Color::Rgb(0, 200, 255),
            mem_color: Color::Rgb(0, 255, 150),
            temp_color: Color::Rgb(255, 100, 50),
            net_rx_color: Color::Rgb(100, 200, 255),
            net_tx_color: Color::Rgb(255, 150, 100),
            battery_color: Color::Rgb(50, 255, 100),
            warning_color: Color::Rgb(255, 200, 0),
            critical_color: Color::Rgb(255, 50, 50),
        }
    }

    /// Nordic minimalist theme with cool tones
    pub fn nord() -> Self {
        Self {
            bg: Color::Rgb(46, 52, 64),
            fg: Color::Rgb(216, 222, 233),
            accent: Color::Rgb(136, 192, 208),
            cpu_color: Color::Rgb(129, 161, 193),
            mem_color: Color::Rgb(163, 190, 140),
            temp_color: Color::Rgb(191, 97, 106),
            net_rx_color: Color::Rgb(94, 129, 172),
            net_tx_color: Color::Rgb(208, 135, 112),
            battery_color: Color::Rgb(163, 190, 140),
            warning_color: Color::Rgb(235, 203, 139),
            critical_color: Color::Rgb(191, 97, 106),
        }
    }

    /// Classic dark theme with vibrant colors
    pub fn dracula() -> Self {
        Self {
            bg: Color::Rgb(40, 42, 54),
            fg: Color::Rgb(248, 248, 242),
            accent: Color::Rgb(189, 147, 249),
            cpu_color: Color::Rgb(139, 233, 253),
            mem_color: Color::Rgb(80, 250, 123),
            temp_color: Color::Rgb(255, 85, 85),
            net_rx_color: Color::Rgb(98, 114, 164),
            net_tx_color: Color::Rgb(255, 121, 198),
            battery_color: Color::Rgb(80, 250, 123),
            warning_color: Color::Rgb(241, 250, 140),
            critical_color: Color::Rgb(255, 85, 85),
        }
    }

    /// Tokyo night sky inspired theme
    pub fn tokyo_night() -> Self {
        Self {
            bg: Color::Rgb(26, 27, 38),
            fg: Color::Rgb(192, 202, 245),
            accent: Color::Rgb(122, 162, 247),
            cpu_color: Color::Rgb(125, 207, 255),
            mem_color: Color::Rgb(158, 206, 106),
            temp_color: Color::Rgb(247, 118, 142),
            net_rx_color: Color::Rgb(122, 162, 247),
            net_tx_color: Color::Rgb(255, 158, 100),
            battery_color: Color::Rgb(158, 206, 106),
            warning_color: Color::Rgb(224, 175, 104),
            critical_color: Color::Rgb(247, 118, 142),
        }
    }

    /// Classic Monokai editor theme
    pub fn monokai() -> Self {
        Self {
            bg: Color::Rgb(39, 40, 34),
            fg: Color::Rgb(248, 248, 242),
            accent: Color::Rgb(249, 38, 114),
            cpu_color: Color::Rgb(102, 217, 239),
            mem_color: Color::Rgb(166, 226, 46),
            temp_color: Color::Rgb(253, 151, 31),
            net_rx_color: Color::Rgb(102, 217, 239),
            net_tx_color: Color::Rgb(253, 151, 31),
            battery_color: Color::Rgb(166, 226, 46),
            warning_color: Color::Rgb(230, 219, 116),
            critical_color: Color::Rgb(249, 38, 114),
        }
    }

    /// Solarized dark theme with carefully chosen colors
    pub fn solarized_dark() -> Self {
        Self {
            bg: Color::Rgb(0, 43, 54),
            fg: Color::Rgb(131, 148, 150),
            accent: Color::Rgb(38, 139, 210),
            cpu_color: Color::Rgb(42, 161, 152),
            mem_color: Color::Rgb(133, 153, 0),
            temp_color: Color::Rgb(203, 75, 22),
            net_rx_color: Color::Rgb(38, 139, 210),
            net_tx_color: Color::Rgb(211, 54, 130),
            battery_color: Color::Rgb(133, 153, 0),
            warning_color: Color::Rgb(181, 137, 0),
            critical_color: Color::Rgb(220, 50, 47),
        }
    }

    /// Retro-inspired Gruvbox theme
    pub fn gruvbox() -> Self {
        Self {
            bg: Color::Rgb(40, 40, 40),
            fg: Color::Rgb(235, 219, 178),
            accent: Color::Rgb(214, 93, 14),
            cpu_color: Color::Rgb(69, 133, 136),
            mem_color: Color::Rgb(152, 151, 26),
            temp_color: Color::Rgb(204, 36, 29),
            net_rx_color: Color::Rgb(69, 133, 136),
            net_tx_color: Color::Rgb(214, 93, 14),
            battery_color: Color::Rgb(152, 151, 26),
            warning_color: Color::Rgb(215, 153, 33),
            critical_color: Color::Rgb(204, 36, 29),
        }
    }

    /// Soft pastel Catppuccin theme
    pub fn catppuccin() -> Self {
        Self {
            bg: Color::Rgb(30, 30, 46),
            fg: Color::Rgb(205, 214, 244),
            accent: Color::Rgb(137, 180, 250),
            cpu_color: Color::Rgb(116, 199, 236),
            mem_color: Color::Rgb(166, 227, 161),
            temp_color: Color::Rgb(243, 139, 168),
            net_rx_color: Color::Rgb(116, 199, 236),
            net_tx_color: Color::Rgb(250, 179, 135),
            battery_color: Color::Rgb(166, 227, 161),
            warning_color: Color::Rgb(249, 226, 175),
            critical_color: Color::Rgb(243, 139, 168),
        }
    }

    /// Atom editor's One Dark theme
    pub fn one_dark() -> Self {
        Self {
            bg: Color::Rgb(40, 44, 52),
            fg: Color::Rgb(171, 178, 191),
            accent: Color::Rgb(97, 175, 239),
            cpu_color: Color::Rgb(86, 182, 194),
            mem_color: Color::Rgb(152, 195, 121),
            temp_color: Color::Rgb(224, 108, 117),
            net_rx_color: Color::Rgb(97, 175, 239),
            net_tx_color: Color::Rgb(209, 154, 102),
            battery_color: Color::Rgb(152, 195, 121),
            warning_color: Color::Rgb(229, 192, 123),
            critical_color: Color::Rgb(224, 108, 117),
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
    /// Returns None if the theme name is not recognized
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

    /// Returns the name of the current theme
    pub fn name(&self) -> &'static str {
        // Compare colors to determine theme
        if self.bg == Color::Rgb(15, 15, 25) {
            "cyberpunk"
        } else if self.bg == Color::Rgb(46, 52, 64) {
            "nord"
        } else if self.bg == Color::Rgb(40, 42, 54) {
            "dracula"
        } else if self.bg == Color::Rgb(26, 27, 38) {
            "tokyo_night"
        } else if self.bg == Color::Rgb(39, 40, 34) {
            "monokai"
        } else if self.bg == Color::Rgb(0, 43, 54) {
            "solarized_dark"
        } else if self.bg == Color::Rgb(40, 40, 40) {
            "gruvbox"
        } else if self.bg == Color::Rgb(30, 30, 46) {
            "catppuccin"
        } else if self.bg == Color::Rgb(40, 44, 52) {
            "one_dark"
        } else {
            "unknown"
        }
    }
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
            // Case insensitive tests
            ("Cyberpunk", true),
            ("NORD", true),
            ("DrAcUlA", true),
            // Alternative names
            ("tokyonight", true),
            ("solarized", true),
            ("onedark", true),
            // Invalid names
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
        assert_eq!(theme.bg, Color::Rgb(15, 15, 25));
    }
}
//! UI module for the system monitor
//! Handles all terminal rendering and user interface components

pub mod chart;
pub mod chip_heatmap;
pub mod components;
pub mod layout;
pub mod layouts;
pub mod party_mode;
pub mod retro_effects;
pub mod sonification;
pub mod theme;

use crate::config::{Config, LayoutSettings};
use crate::history::HistoryData;
use crate::types::*;
use crate::ui::layouts::LayoutType;
use ratatui::{
    backend::TermionBackend,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame, Terminal,
};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io;
use std::process::Command;
use sysinfo::System;
use termion::raw::IntoRawMode;

use self::party_mode::PartyMode;
use self::theme::Theme;

/// Main UI structure
pub struct UI {
    terminal: Terminal<TermionBackend<termion::raw::RawTerminal<std::io::Stderr>>>,
    theme: Theme,
    party_state: PartyMode,
    /// Currently active layout
    current_layout: LayoutType,
    /// Whether help overlay is shown
    show_help: bool,
    /// Whether the session report overlay is shown
    show_session_report: bool,
    /// Process list scroll offset
    scroll_offset: usize,
}

impl UI {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Self::with_theme("cyberpunk")
    }

    pub fn with_theme(theme_name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let stderr = io::stderr().into_raw_mode()?;
        let backend = TermionBackend::new(stderr);
        let terminal = Terminal::new(backend)?;

        let theme = Theme::from_name(theme_name).unwrap_or_else(|| {
            log::warn!("Unknown theme '{}', using mactop_green", theme_name);
            Theme::mactop_green()
        });

        // Load persisted layout settings
        let settings = LayoutSettings::load_runtime_state();

        Ok(Self {
            terminal,
            theme,
            party_state: PartyMode::new(),
            current_layout: LayoutType::Startup,
            show_help: false,
            show_session_report: false,
            scroll_offset: settings.process_scroll_offset,
        })
    }

    /// Create UI with theme and initial layout
    pub fn with_theme_and_layout(
        theme_name: &str,
        layout: LayoutType,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut ui = Self::with_theme(theme_name)?;
        ui.current_layout = layout;
        Ok(ui)
    }

    /// Change theme without re-initializing the terminal.
    /// Re-creating UI would break raw mode and stdin, killing keyboard input.
    pub fn set_theme(&mut self, theme_name: &str) {
        if let Some(theme) = Theme::from_name(theme_name) {
            self.theme = theme;
        } else {
            log::warn!("Unknown theme '{}', keeping current", theme_name);
        }
    }

    /// Fill the entire terminal area with the theme background color.
    /// This prevents terminal transparency from showing through.
    fn fill_background(f: &mut Frame, area: Rect, bg: Color) {
        let bg_block = Block::default().style(Style::default().bg(bg));
        f.render_widget(Clear, area);
        f.render_widget(bg_block, area);
    }

    /// Show loading screen with animation
    pub fn show_loading_screen(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let theme = &self.theme;

        self.terminal.draw(|f| {
            Self::render_startup_page(f, theme);
        })?;

        Ok(())
    }

    fn render_startup_page(f: &mut Frame, theme: &Theme) {
        let size = f.size();
        Self::fill_background(f, size, theme.bg);

        let summary = StartupSummary::collect();
        let mut loading_lines = build_startup_banner_lines(theme);

        loading_lines.extend(build_startup_dashboard_lines(&summary, theme));
        loading_lines.push(Line::from(""));
        loading_lines.push(startup_centered_line(
            "Keys: Enter / Space / 1 dashboard · R report · t theme · ? help · q quit",
            Style::default().fg(Color::DarkGray),
        ));

        let startup_block = Paragraph::new(loading_lines)
            .block(
                Block::default()
                    .title(" AIMON STARTUP SUMMARY ")
                    .title_alignment(Alignment::Center)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border_color)),
            )
            .alignment(Alignment::Left);

        let startup_area = centered_startup_area(size);
        f.render_widget(startup_block, startup_area);
    }

    /// Main draw function - dispatches to the current layout
    pub fn draw(
        &mut self,
        data: &SystemData,
        history: &HistoryData,
        config: &Config,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let theme = &self.theme;
        let current_layout = self.current_layout;
        let show_help = self.show_help;
        let show_session_report = self.show_session_report;

        self.terminal.draw(|f| {
            // Fill background first to prevent terminal transparency
            Self::fill_background(f, f.size(), theme.bg);

            // Draw the current layout
            match current_layout {
                LayoutType::Startup => {
                    Self::render_startup_page(f, theme);
                }
                LayoutType::Full => {
                    layouts::full::draw(f, data, history, config, theme);
                }
                LayoutType::Advanced => {
                    layouts::advanced::draw(f, data, history, config, theme);
                }
                LayoutType::Minimal => {
                    layouts::minimal::draw(f, data, history, theme);
                }
                LayoutType::Compact => {
                    layouts::compact::draw(f, data, history, theme);
                }
                LayoutType::BatteryFocus => {
                    layouts::battery_focus::draw(f, data, history, theme);
                }
                LayoutType::GpuFocus => {
                    layouts::gpu_focus::draw(f, data, history, theme);
                }
                LayoutType::NetworkFocus => {
                    layouts::network_focus::draw(f, data, history, theme);
                }
                LayoutType::SystemHealth => {
                    layouts::system_health::draw(f, data, history, theme);
                }
                LayoutType::Thermals => {
                    layouts::thermals::draw(f, data, history, config, theme);
                }
            }

            // Draw help overlay on top if active
            if show_help {
                components::render_help_overlay(f, theme);
            }
            if show_session_report {
                components::render_session_report_overlay(f, data, theme);
            }
        })?;

        Ok(())
    }

    /// Switch to the next layout in sequence
    pub fn next_layout(&mut self) {
        let all = LayoutType::all();
        let current_idx = all
            .iter()
            .position(|&l| l == self.current_layout)
            .unwrap_or(0);
        let next_idx = (current_idx + 1) % all.len();
        self.current_layout = all[next_idx];
        log::info!("Layout changed to: {}", self.current_layout);
    }

    /// Switch to the previous layout in sequence
    pub fn previous_layout(&mut self) {
        let all = LayoutType::all();
        let current_idx = all
            .iter()
            .position(|&l| l == self.current_layout)
            .unwrap_or(0);
        let prev_idx = if current_idx == 0 {
            all.len() - 1
        } else {
            current_idx - 1
        };
        self.current_layout = all[prev_idx];
        log::info!("Layout changed to: {}", self.current_layout);
    }

    /// Jump to a specific layout by type
    pub fn set_layout(&mut self, layout: LayoutType) {
        self.current_layout = layout;
        log::info!("Layout set to: {}", self.current_layout);
    }

    /// Get the current layout type
    pub fn current_layout(&self) -> LayoutType {
        self.current_layout
    }

    /// Toggle help overlay visibility
    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    /// Toggle session report overlay visibility
    pub fn toggle_session_report(&mut self) {
        self.show_session_report = !self.show_session_report;
    }

    /// Check whether session report overlay is visible
    pub fn is_session_report_visible(&self) -> bool {
        self.show_session_report
    }

    /// Scroll process list down
    pub fn scroll_down(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_add(1);
    }

    /// Scroll process list up
    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    /// Jump to top of process list
    pub fn go_to_top(&mut self) {
        self.scroll_offset = 0;
    }

    /// Jump to bottom of process list
    pub fn go_to_bottom(&mut self) {
        // Will be clamped during render based on actual process count
        self.scroll_offset = usize::MAX;
    }

    /// Get current scroll offset
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Save current runtime state for persistence
    pub fn save_runtime_state(&self, config: &Config) {
        let settings = LayoutSettings {
            current_layout: self.current_layout,
            process_sort_by: config.process_sort_by.clone(),
            process_scroll_offset: self.scroll_offset,
            party_mode: false,
        };
        if let Err(e) = settings.save_runtime_state() {
            log::warn!("Failed to save runtime state: {}", e);
        }
    }

    /// Toggle party mode on/off
    pub fn toggle_party_mode(&mut self) {
        self.party_state.toggle();
    }

    /// Cleanup terminal
    pub fn cleanup(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.terminal.clear()?;
        self.terminal.show_cursor()?;
        Ok(())
    }

    pub fn next_tab(&mut self) {
        self.next_layout();
    }

    pub fn previous_tab(&mut self) {
        self.previous_layout();
    }
}

struct StartupSummary {
    system_name: String,
    host_name: String,
    os_version: String,
    kernel_version: String,
    model_identifier: String,
    cpu_arch: String,
    cpu_brand: String,
    cpu_cores: usize,
    cpu_frequency_mhz: u64,
    memory_gb: f64,
    used_memory_gb: f64,
    uptime_seconds: u64,
    serial_number: String,
    is_root: bool,
    has_battery_hint: bool,
}

impl StartupSummary {
    fn collect() -> Self {
        let system = System::new_all();
        let cpu_brand = system
            .cpus()
            .first()
            .map(|cpu| cpu.brand().to_string())
            .unwrap_or_else(|| "Unknown CPU".to_string());

        Self {
            system_name: System::name().unwrap_or_else(|| "Unknown".to_string()),
            host_name: System::host_name().unwrap_or_else(|| "Unknown".to_string()),
            os_version: System::os_version().unwrap_or_else(|| "Unknown".to_string()),
            kernel_version: System::kernel_version().unwrap_or_else(|| "Unknown".to_string()),
            model_identifier: read_sysctl_value("hw.model")
                .unwrap_or_else(|| "Unknown".to_string()),
            cpu_arch: System::cpu_arch().unwrap_or_else(|| "Unknown".to_string()),
            cpu_brand,
            cpu_cores: system.cpus().len(),
            cpu_frequency_mhz: system
                .cpus()
                .first()
                .map(|cpu| cpu.frequency())
                .unwrap_or(0),
            memory_gb: system.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0,
            used_memory_gb: system.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0,
            uptime_seconds: System::uptime(),
            serial_number: read_serial_number().unwrap_or_else(|| "Unknown".to_string()),
            is_root: unsafe { libc::geteuid() == 0 },
            has_battery_hint: Command::new("pmset")
                .arg("-g")
                .arg("batt")
                .output()
                .map(|output| String::from_utf8_lossy(&output.stdout).contains('%'))
                .unwrap_or(false),
        }
    }
}

const STARTUP_CONTENT_WIDTH: usize = 100;
const STARTUP_COLUMN_WIDTH: usize = 47;

fn build_startup_banner_lines(theme: &Theme) -> Vec<Line<'static>> {
    let logo_style = Style::default()
        .fg(theme.border_color)
        .add_modifier(Modifier::BOLD);
    vec![
        startup_centered_line(" █████╗ ██╗███╗   ███╗ ██████╗ ███╗   ██╗", logo_style),
        startup_centered_line("██╔══██╗██║████╗ ████║██╔═══██╗████╗  ██║", logo_style),
        startup_centered_line("███████║██║██╔████╔██║██║   ██║██╔██╗ ██║", logo_style),
        startup_centered_line("██╔══██║██║██║╚██╔╝██║██║   ██║██║╚██╗██║", logo_style),
        startup_centered_line("██║  ██║██║██║ ╚═╝ ██║╚██████╔╝██║ ╚████║", logo_style),
        startup_centered_line("╚═╝  ╚═╝╚═╝╚═╝     ╚═╝ ╚═════╝ ╚═╝  ╚═══╝", logo_style),
        startup_centered_line("by TimWood", Style::default().fg(Color::Gray)),
        startup_centered_line(
            "┌─ Apple Silicon Energy Dashboard ─┐",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Line::from(""),
        startup_centered_line(
            &format!("system-alert v{}", env!("CARGO_PKG_VERSION")),
            Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
        ),
        Line::from(""),
    ]
}

fn build_startup_dashboard_lines(summary: &StartupSummary, theme: &Theme) -> Vec<Line<'static>> {
    let battery = if summary.has_battery_hint {
        "detected"
    } else {
        "unknown"
    };
    let left = [
        startup_kv("Host", &summary.host_name),
        startup_kv("Name", &summary.system_name),
        startup_kv("macOS", &summary.os_version),
        startup_kv("Kernel", &summary.kernel_version),
        startup_kv("Model", &summary.model_identifier),
        startup_kv("Serial", &mask_serial_number(&summary.serial_number)),
    ];
    let right = [
        startup_kv("CPU", &summary.cpu_brand),
        startup_kv("Arch", &summary.cpu_arch),
        startup_kv("Cores", &format!("{} logical", summary.cpu_cores)),
        startup_kv("Clock", &format!("{} MHz", summary.cpu_frequency_mhz)),
        startup_kv(
            "Memory",
            &format!(
                "{:.1} / {:.1} GB",
                summary.used_memory_gb, summary.memory_gb
            ),
        ),
        startup_kv(
            "Power",
            &format!(
                "battery {} · uptime {}",
                battery,
                format_startup_uptime(summary.uptime_seconds)
            ),
        ),
    ];

    let checks = [
        ("terminal", true),
        ("snapshot", summary.cpu_cores > 0),
        ("powermetrics", summary.is_root),
        ("battery", summary.has_battery_hint),
    ];
    let check_line = checks
        .iter()
        .map(|(name, ok)| format!("[{} {}]", if *ok { "OK" } else { "WARN" }, name))
        .collect::<Vec<_>>()
        .join("  ");

    let mut lines = vec![startup_pair_line(
        "Hardware Summary",
        "System Profile",
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD),
    )];
    for (left_item, right_item) in left.iter().zip(right.iter()) {
        lines.push(startup_pair_line(
            left_item,
            right_item,
            Style::default().fg(theme.fg),
        ));
    }
    lines.push(Line::from(""));
    lines.push(startup_centered_line(
        &check_line,
        Style::default().fg(if summary.is_root {
            theme.fg
        } else {
            theme.warning_color
        }),
    ));
    lines
}

fn startup_kv(key: &str, value: &str) -> String {
    format!("{key:<8} {value}")
}

fn startup_pair_line(left: &str, right: &str, style: Style) -> Line<'static> {
    Line::from(Span::styled(
        format!(
            "  {:<left_width$}  {:<right_width$}",
            truncate_startup_field(left, STARTUP_COLUMN_WIDTH),
            truncate_startup_field(right, STARTUP_COLUMN_WIDTH),
            left_width = STARTUP_COLUMN_WIDTH,
            right_width = STARTUP_COLUMN_WIDTH
        ),
        style,
    ))
}

fn startup_centered_line(text: &str, style: Style) -> Line<'static> {
    let text = truncate_startup_field(text, STARTUP_CONTENT_WIDTH);
    let width = text.chars().count();
    let padding = STARTUP_CONTENT_WIDTH.saturating_sub(width) / 2;
    Line::from(Span::styled(
        format!("{}{}", " ".repeat(padding), text),
        style,
    ))
}

fn mask_serial_number(serial: &str) -> String {
    let serial = serial.trim();
    if serial.is_empty() || serial.eq_ignore_ascii_case("unknown") {
        return "Unknown".to_string();
    }

    let chars: Vec<char> = serial.chars().collect();
    if chars.len() <= 4 {
        return serial.to_string();
    }

    let prefix = chars.iter().take(2).collect::<String>();
    let suffix = chars
        .iter()
        .skip(chars.len().saturating_sub(2))
        .collect::<String>();
    let middle = chars[2..chars.len() - 2].iter().collect::<String>();
    let hash = short_serial_hash(&middle);

    format!("{}#{}#{}", prefix, hash, suffix)
}

fn short_serial_hash(serial: &str) -> String {
    let mut hasher = DefaultHasher::new();
    serial.hash(&mut hasher);
    format!("{:06X}", hasher.finish() & 0xFF_FFFF)
}

fn centered_startup_area(area: Rect) -> Rect {
    let width = area.width.saturating_sub(2).min(110).max(40);
    let height = area.height.saturating_sub(2).min(30).max(12);
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;

    Rect {
        x,
        y,
        width: width.min(area.width),
        height: height.min(area.height),
    }
}

fn truncate_startup_field(value: &str, max_len: usize) -> String {
    if value.chars().count() <= max_len {
        return value.to_string();
    }
    let mut truncated = value
        .chars()
        .take(max_len.saturating_sub(1))
        .collect::<String>();
    truncated.push('…');
    truncated
}

fn format_startup_uptime(seconds: u64) -> String {
    let days = seconds / 86_400;
    let hours = (seconds % 86_400) / 3_600;
    let minutes = (seconds % 3_600) / 60;
    if days > 0 {
        format!("{}d {}h {}m", days, hours, minutes)
    } else if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}

fn read_sysctl_value(key: &str) -> Option<String> {
    let output = Command::new("sysctl").args(["-n", key]).output().ok()?;
    String::from_utf8(output.stdout)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn read_serial_number() -> Option<String> {
    let output = Command::new("ioreg")
        .args(["-rd1", "-c", "IOPlatformExpertDevice"])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    text.lines()
        .find_map(|line| line.split_once("IOPlatformSerialNumber"))
        .and_then(|(_, value)| value.split_once('=').map(|(_, value)| value))
        .map(|value| value.trim().trim_matches('"').to_string())
        .filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_serial_number() {
        // Test empty/unknown
        assert_eq!(mask_serial_number(""), "Unknown");
        assert_eq!(mask_serial_number("unknown"), "Unknown");

        // Test short serial
        assert_eq!(mask_serial_number("A"), "A");
        assert_eq!(mask_serial_number("ABCD"), "ABCD");

        // Test normal serial (format: first 2 chars + # + 6-char hex hash + last 2 chars)
        let masked = mask_serial_number("C02X12345678");
        assert!(masked.starts_with("C0"));
        assert!(masked.ends_with("78"));
        assert_eq!(masked.matches('#').count(), 2);

        let parts: Vec<&str> = masked.split('#').collect();
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0], "C0");
        assert_eq!(parts[2], "78");
        assert_eq!(parts[1].len(), 6); // short_serial_hash returns 6 hex chars
    }
}

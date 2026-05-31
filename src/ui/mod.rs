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
        let loading_lines = vec![
            Line::from(Span::styled(
                "███╗   ███╗ █████╗  ██████╗",
                Style::default()
                    .fg(theme.border_color)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                "████╗ ████║██╔══██╗██╔════╝",
                Style::default()
                    .fg(theme.border_color)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                "██╔████╔██║███████║██║     ",
                Style::default()
                    .fg(theme.border_color)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                "██║╚██╔╝██║██╔══██║██║     ",
                Style::default()
                    .fg(theme.border_color)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                "██║ ╚═╝ ██║██║  ██║╚██████╗",
                Style::default()
                    .fg(theme.border_color)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                "╚═╝     ╚═╝╚═╝  ╚═╝ ╚═════╝",
                Style::default()
                    .fg(theme.border_color)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled("by TimWood", Style::default().fg(Color::Gray))),
            Line::from(""),
            Line::from(Span::styled(
                format!("system-alert v{}", env!("CARGO_PKG_VERSION")),
                Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                format!("Host:     {}", summary.host_name),
                Style::default().fg(theme.fg),
            )),
            Line::from(Span::styled(
                format!(
                    "macOS:    {} ({})",
                    summary.os_version, summary.kernel_version
                ),
                Style::default().fg(theme.fg),
            )),
            Line::from(Span::styled(
                format!("Model:    {}", summary.model_identifier),
                Style::default().fg(theme.fg),
            )),
            Line::from(Span::styled(
                format!("Arch:     {}", summary.cpu_arch),
                Style::default().fg(theme.fg),
            )),
            Line::from(Span::styled(
                format!("CPU:      {}", summary.cpu_brand),
                Style::default().fg(theme.fg),
            )),
            Line::from(Span::styled(
                format!("Cores:    {} logical", summary.cpu_cores),
                Style::default().fg(theme.fg),
            )),
            Line::from(Span::styled(
                format!("Memory:   {:.1} GB", summary.memory_gb),
                Style::default().fg(theme.fg),
            )),
            Line::from(Span::styled(
                format!("Serial:   {}", summary.serial_number),
                Style::default().fg(theme.fg),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Press Enter / Space / 1 to enter dashboard · 0 returns here · q quits",
                Style::default().fg(Color::DarkGray),
            )),
        ];

        let loading_block = Paragraph::new(loading_lines)
            .block(
                Block::default()
                    .title(" STARTUP SUMMARY ")
                    .title_alignment(Alignment::Center)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border_color)),
            )
            .alignment(Alignment::Center);

        let areas = layout::create_loading_layout(size);
        f.render_widget(loading_block, areas[0]);
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
    host_name: String,
    os_version: String,
    kernel_version: String,
    model_identifier: String,
    cpu_arch: String,
    cpu_brand: String,
    cpu_cores: usize,
    memory_gb: f64,
    serial_number: String,
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
            host_name: System::host_name().unwrap_or_else(|| "Unknown".to_string()),
            os_version: System::os_version().unwrap_or_else(|| "Unknown".to_string()),
            kernel_version: System::kernel_version().unwrap_or_else(|| "Unknown".to_string()),
            model_identifier: read_sysctl_value("hw.model")
                .unwrap_or_else(|| "Unknown".to_string()),
            cpu_arch: System::cpu_arch().unwrap_or_else(|| "Unknown".to_string()),
            cpu_brand,
            cpu_cores: system.cpus().len(),
            memory_gb: system.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0,
            serial_number: read_serial_number().unwrap_or_else(|| "Unknown".to_string()),
        }
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

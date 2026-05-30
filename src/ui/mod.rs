//! UI module for the system monitor
//! Handles all terminal rendering and user interface components

pub mod theme;
pub mod components;
pub mod layout;
pub mod chart;
pub mod layouts;
pub mod chip_heatmap;
pub mod party_mode;
pub mod retro_effects;
pub mod sonification;

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
use termion::raw::IntoRawMode;

use self::theme::Theme;
use self::party_mode::PartyMode;

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
            log::warn!("Unknown theme '{}', using cyberpunk", theme_name);
            Theme::cyberpunk()
        });

        // Load persisted layout settings
        let settings = LayoutSettings::load_runtime_state();

        Ok(Self {
            terminal,
            theme,
            party_state: PartyMode::new(),
            current_layout: settings.current_layout,
            show_help: false,
            scroll_offset: settings.process_scroll_offset,
        })
    }

    /// Create UI with theme and initial layout
    pub fn with_theme_and_layout(theme_name: &str, layout: LayoutType) -> Result<Self, Box<dyn std::error::Error>> {
        let mut ui = Self::with_theme(theme_name)?;
        ui.current_layout = layout;
        Ok(ui)
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
            let size = f.size();
            Self::fill_background(f, size, theme.bg);

            let loading_lines = vec![
                Line::from(Span::styled(
                    "SYSTEM ALERT",
                    Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓",
                    Style::default().fg(theme.cpu_color),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "Initializing quantum sensors...",
                    Style::default().fg(theme.fg),
                )),
                Line::from(Span::styled(
                    "Calibrating neural network...",
                    Style::default().fg(theme.fg),
                )),
                Line::from(Span::styled(
                    "Loading particle accelerators...",
                    Style::default().fg(theme.fg),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "Press 'q' to abort",
                    Style::default().fg(Color::DarkGray),
                )),
            ];

            let loading_block = Paragraph::new(loading_lines)
                .block(
                    Block::default()
                        .title(" INITIALIZING ")
                        .title_alignment(Alignment::Center)
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(theme.accent)),
                )
                .alignment(Alignment::Center);

            let areas = layout::create_loading_layout(size);
            f.render_widget(loading_block, areas[0]);
        })?;

        Ok(())
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
                LayoutType::Full => {
                    layouts::full::draw(f, data, history, config, theme);
                }
                LayoutType::Minimal => {
                    layouts::minimal::draw(f, data, theme);
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
        let current_idx = all.iter().position(|&l| l == self.current_layout).unwrap_or(0);
        let next_idx = (current_idx + 1) % all.len();
        self.current_layout = all[next_idx];
        log::info!("Layout changed to: {}", self.current_layout);
    }

    /// Switch to the previous layout in sequence
    pub fn previous_layout(&mut self) {
        let all = LayoutType::all();
        let current_idx = all.iter().position(|&l| l == self.current_layout).unwrap_or(0);
        let prev_idx = if current_idx == 0 { all.len() - 1 } else { current_idx - 1 };
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

//! UI module for the system monitor
//! Handles all terminal rendering and user interface components

pub mod theme;
pub mod components;
pub mod layout;
pub mod chart;

use crate::{config::Config, history::HistoryData, types::*};
use ratatui::{
    backend::TermionBackend,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use std::io;
use termion::raw::IntoRawMode;

use self::theme::Theme;

/// Main UI structure
pub struct UI {
    terminal: Terminal<TermionBackend<termion::raw::RawTerminal<std::io::Stderr>>>,
    theme: Theme,
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

        Ok(Self {
            terminal,
            theme,
        })
    }

    /// Show loading screen with animation
    pub fn show_loading_screen(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let theme = &self.theme;

        self.terminal.draw(|f| {
            let size = f.size();

            let loading_lines = vec![
                Line::from(Span::styled(
                    "⚡ SYSTEM ALERT ⚡",
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

    /// Main draw function - renders the complete UI
    pub fn draw(
        &mut self,
        data: &SystemData,
        history: &HistoryData,
        config: &Config,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let theme = &self.theme;

        self.terminal.draw(|f| {
            if config.minimal_mode {
                Self::draw_minimal(f, data, theme);
            } else {
                Self::draw_full(f, data, history, config, theme);
            }
        })?;

        Ok(())
    }

    /// Minimal mode - just CPU and memory gauges
    fn draw_minimal(f: &mut Frame, data: &SystemData, theme: &Theme) {
        let areas = layout::create_minimal_layout(f.size());

        components::render_cpu_gauge(f, areas[0], data.cpu_info.average_usage, theme);
        components::render_mem_gauge(f, areas[1], data.memory_info.usage_percentage, theme);
    }

    /// Full mode - comprehensive system overview
    fn draw_full(f: &mut Frame, data: &SystemData, history: &HistoryData, config: &Config, theme: &Theme) {
        let main = layout::create_full_layout(f.size());

        // Header with system info
        components::render_header(f, main[0], data, theme);

        // Top section: CPU, Memory, Battery, Power
        let top = layout::create_top_stats_layout(main[1]);

        Self::render_cpu_stats(f, top[0], data, theme);
        Self::render_mem_stats(f, top[1], data, theme);
        components::render_battery_stats(f, top[2], data, theme);
        components::render_power_stats(f, top[3], data, theme);

        // Middle section: Charts + CPU Cores
        let mid = layout::create_charts_layout(main[2]);
        
        // Left side: CPU history chart
        Self::render_cpu_chart(f, mid[0], history, theme);
        
        // Right side: CPU cores bar chart
        components::render_cpu_cores_bar_chart(f, mid[1], data, theme);

        // Bottom section: Processes, Network, Thermal, Network Sparkline
        let bottom = layout::create_bottom_stats_layout(main[3]);

        components::render_process_list(f, bottom[0], data, config, theme);
        components::render_network_stats(f, bottom[1], data, history, theme);
        components::render_thermal_stats(f, bottom[2], data, theme);
        components::render_network_sparkline(f, bottom[3], history, theme);
    }

    /// Render CPU statistics panel
    fn render_cpu_stats(f: &mut Frame, area: Rect, data: &SystemData, theme: &Theme) {
        components::render_cpu_gauge(f, area, data.cpu_info.average_usage, theme);
    }

    /// Render memory statistics panel
    fn render_mem_stats(f: &mut Frame, area: Rect, data: &SystemData, theme: &Theme) {
        components::render_mem_gauge(f, area, data.memory_info.usage_percentage, theme);
    }

    /// Render CPU history chart
    fn render_cpu_chart(f: &mut Frame, area: Rect, history: &HistoryData, theme: &Theme) {
        let config = chart::ChartConfig::new("CPU HISTORY", 0.0, 100.0, theme.cpu_color);
        chart::render_chart(f, area, &history.cpu_history, &config);
    }

    

    /// Cleanup terminal
    pub fn cleanup(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.terminal.clear()?;
        self.terminal.show_cursor()?;
        Ok(())
    }

    pub fn next_tab(&mut self) {}
    pub fn previous_tab(&mut self) {}
}
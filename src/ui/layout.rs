//! Layout management for the UI
//! Handles the arrangement of UI components

use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Create the main layout structure for full mode
pub fn create_full_layout(area: Rect) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Length(12), // Top stats
            Constraint::Length(14), // Charts
            Constraint::Min(1),     // Bottom stats
            Constraint::Length(1),  // Status bar
        ])
        .split(area)
        .to_vec()
}

/// Create layout for top stats section
pub fn create_top_stats_layout(area: Rect) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(area)
        .to_vec()
}

/// Create layout for charts section
pub fn create_charts_layout(area: Rect) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area)
        .to_vec()
}

/// Create layout for bottom stats section
pub fn create_bottom_stats_layout(area: Rect) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25), // Processes
            Constraint::Percentage(20), // Network
            Constraint::Percentage(20), // Thermal
            Constraint::Percentage(15), // Network Sparkline
            Constraint::Percentage(20), // Terminals (NEW!)
        ])
        .split(area)
        .to_vec()
}

/// Create minimal mode layout
pub fn create_minimal_layout(area: Rect) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area)
        .to_vec()
}

/// Create loading screen layout
pub fn create_loading_layout(area: Rect) -> Vec<Rect> {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(40),
            Constraint::Percentage(30),
        ])
        .split(area);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(60),
            Constraint::Percentage(20),
        ])
        .split(vertical[1]);

    horizontal.to_vec()
}

//! Chart rendering utilities
//! Provides configuration and rendering for charts

use ratatui::{
    layout::Rect,
    style::{Color, Style},
    symbols,
    text::Span,
    widgets::{
        Axis, Block, Borders, Chart, Dataset, GraphType, Sparkline,
    },
    Frame,
};

/// Configuration for chart rendering
#[derive(Debug, Clone)]
pub struct ChartConfig {
    pub min: f64,
    pub max: f64,
    pub color: Color,
    pub title: String,
}

/// Configuration for sparkline rendering
#[derive(Debug, Clone)]
pub struct SparklineConfig {
    pub title: String,
    pub color: Color,
    pub max: Option<u64>,
}

impl ChartConfig {
    pub fn new(title: &str, min: f64, max: f64, color: Color) -> Self {
        Self {
            min,
            max,
            color,
            title: title.to_string(),
        }
    }
}

impl SparklineConfig {
    pub fn new(title: &str, color: Color) -> Self {
        Self {
            title: title.to_string(),
            color,
            max: None,
        }
    }
    
    pub fn with_max(title: &str, color: Color, max: u64) -> Self {
        Self {
            title: title.to_string(),
            color,
            max: Some(max),
        }
    }
}

/// Render a chart with the given configuration
pub fn render_chart<T: Into<f64> + Copy>(
    f: &mut Frame,
    area: Rect,
    data: &std::collections::VecDeque<T>,
    config: &ChartConfig,
) {
    let points: Vec<(f64, f64)> = data
        .iter()
        .enumerate()
        .map(|(i, &v)| (i as f64, v.into()))
        .collect();

    let datasets = vec![Dataset::default()
        .name(&config.title)
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(config.color))
        .data(&points)];

    let chart = Chart::new(datasets)
        .block(
            Block::default()
                .title(format!(" {} ", config.title))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(config.color))
                .style(Style::default().bg(Color::Black)),
        )
        .x_axis(
            Axis::default()
                .bounds([0.0, data.len().max(1) as f64])
                .style(Style::default().fg(Color::DarkGray)),
        )
        .y_axis(
            Axis::default()
                .bounds([config.min, config.max])
                .labels(vec![
                    Span::styled(format!("{}", config.min), Style::default().fg(Color::DarkGray)),
                    Span::styled(format!("{}", config.max), Style::default().fg(Color::DarkGray)),
                ])
                .style(Style::default().fg(Color::DarkGray)),
        );

    f.render_widget(chart, area);
}

/// Render a sparkline with the given configuration
pub fn render_sparkline<T: Into<u64> + Copy>(
    f: &mut Frame,
    area: Rect,
    data: &std::collections::VecDeque<T>,
    config: &SparklineConfig,
) {
    let sparkline_data: Vec<u64> = data.iter().map(|&v| v.into()).collect();
    
    let sparkline = Sparkline::default()
        .block(
            Block::default()
                .title(format!(" {} ", config.title))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(config.color))
                .style(Style::default().bg(Color::Black)),
        )
        .data(&sparkline_data)
        .style(Style::default().fg(config.color).bg(Color::Black));
    
    f.render_widget(sparkline, area);
}
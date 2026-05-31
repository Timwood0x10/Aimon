//! Chart rendering utilities
//! Provides configuration and rendering for charts

use ratatui::{
    layout::Rect,
    prelude::Modifier,
    style::{Color, Style},
    symbols,
    text::Span,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType},
    Frame,
};

/// Configuration for chart rendering
#[derive(Debug, Clone)]
pub struct ChartConfig {
    pub min: f64,
    pub max: f64,
    pub color: Color,
    pub bg: Color,
    pub border_color: Color,
    pub title: String,
}

/// Configuration for sparkline rendering
#[derive(Debug, Clone)]
pub struct SparklineConfig {
    pub title: String,
    pub color: Color,
    pub bg: Color,
    pub border_color: Color,
    pub max: Option<u64>,
}

impl ChartConfig {
    pub fn new(title: &str, min: f64, max: f64, color: Color) -> Self {
        Self {
            min,
            max,
            color,
            bg: Color::Black,
            border_color: color,
            title: title.to_string(),
        }
    }

    pub fn with_bg(mut self, bg: Color) -> Self {
        self.bg = bg;
        self
    }

    pub fn with_border_color(mut self, border_color: Color) -> Self {
        self.border_color = border_color;
        self
    }
}

impl SparklineConfig {
    pub fn new(title: &str, color: Color) -> Self {
        Self {
            title: title.to_string(),
            color,
            bg: Color::Black,
            border_color: color,
            max: None,
        }
    }

    pub fn with_max(title: &str, color: Color, max: u64) -> Self {
        Self {
            title: title.to_string(),
            color,
            bg: Color::Black,
            border_color: color,
            max: Some(max),
        }
    }

    pub fn with_bg(mut self, bg: Color) -> Self {
        self.bg = bg;
        self
    }

    pub fn with_border_color(mut self, border_color: Color) -> Self {
        self.border_color = border_color;
        self
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
        .style(Style::default().fg(config.color).bg(config.bg))
        .data(&points)];

    let chart = Chart::new(datasets)
        .block(
            Block::default()
                .title(format!(" {} ", config.title))
                .borders(Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Plain)
                .border_style(Style::default().fg(config.border_color))
                .style(Style::default().bg(config.bg).fg(config.color)),
        )
        .style(Style::default().bg(config.bg).fg(config.color))
        .x_axis(
            Axis::default()
                .bounds([0.0, data.len().max(1) as f64])
                .style(
                    Style::default()
                        .fg(config.color)
                        .add_modifier(Modifier::DIM),
                ),
        )
        .y_axis(
            Axis::default()
                .bounds([config.min, config.max])
                .labels(vec![
                    Span::styled(
                        format!("{}", config.min),
                        Style::default()
                            .fg(config.color)
                            .add_modifier(Modifier::DIM),
                    ),
                    Span::styled(
                        format!("{}", config.max),
                        Style::default()
                            .fg(config.color)
                            .add_modifier(Modifier::DIM),
                    ),
                ])
                .style(
                    Style::default()
                        .fg(config.color)
                        .add_modifier(Modifier::DIM),
                ),
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
    let converted: std::collections::VecDeque<f64> =
        data.iter().map(|&value| value.into() as f64).collect();
    let max = config
        .max
        .map(|max| max as f64)
        .unwrap_or_else(|| converted.iter().copied().fold(0.0_f64, f64::max).max(1.0));
    let chart_config = ChartConfig::new(&config.title, 0.0, max, config.color)
        .with_bg(config.bg)
        .with_border_color(config.border_color);

    render_chart(f, area, &converted, &chart_config);
}

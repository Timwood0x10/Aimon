//! Chart rendering utilities
//! Provides configuration and rendering for charts with ratatui v0.30.1 features:
//! - Multiple marker styles (Braille, Dot, HalfBlock, Bar, etc.)
//! - Block shadows for visual depth
//! - Filled area charts (GraphType::Area)
//! - Multi-dataset overlay support

use ratatui::{
    layout::{Offset, Rect},
    prelude::Modifier,
    style::{Color, Style},
    symbols,
    text::Span,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Shadow},
    Frame,
};

/// Supported chart marker styles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ChartMarker {
    /// Braille dot-matrix (default, high resolution)
    #[default]
    Braille,
    /// Dot marker
    Dot,
    /// Half-block bar
    HalfBlock,
    /// Full block bar
    Bar,
}

impl From<ChartMarker> for symbols::Marker {
    fn from(marker: ChartMarker) -> Self {
        match marker {
            ChartMarker::Braille => symbols::Marker::Braille,
            ChartMarker::Dot => symbols::Marker::Dot,
            ChartMarker::HalfBlock => symbols::Marker::HalfBlock,
            ChartMarker::Bar => symbols::Marker::Bar,
        }
    }
}

/// Shadow presets for chart blocks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ChartShadow {
    #[default]
    NoShadow,
    Overlay,
    Block,
    LightShade,
    MediumShade,
    DarkShade,
}

impl ChartShadow {
    fn build(self) -> Option<Shadow> {
        match self {
            Self::NoShadow => None,
            Self::Overlay => Some(Shadow::overlay()),
            Self::Block => Some(Shadow::block()),
            Self::LightShade => Some(Shadow::light_shade()),
            Self::MediumShade => Some(Shadow::medium_shade()),
            Self::DarkShade => Some(Shadow::dark_shade()),
        }
    }
}

/// Graph type: line or filled area
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ChartGraphType {
    #[default]
    Line,
    Area,
}

/// Configuration for chart rendering
#[derive(Debug, Clone)]
pub struct ChartConfig {
    pub min: f64,
    pub max: f64,
    pub color: Color,
    pub bg: Color,
    pub border_color: Color,
    pub title: String,
    /// Marker style for rendering data points
    pub marker: ChartMarker,
    /// Shadow effect for the chart block
    pub shadow: ChartShadow,
    /// Line or area graph
    pub graph_type: ChartGraphType,
    /// Baseline Y value for area fill (only used when graph_type is Area)
    pub area_baseline: f64,
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
            marker: ChartMarker::default(),
            shadow: ChartShadow::default(),
            graph_type: ChartGraphType::default(),
            area_baseline: 0.0,
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

    /// Set the marker style for data point rendering
    pub fn with_marker(mut self, marker: ChartMarker) -> Self {
        self.marker = marker;
        self
    }

    /// Set shadow effect for the chart block (ratatui v0.30.1+)
    pub fn with_shadow(mut self, shadow: ChartShadow) -> Self {
        self.shadow = shadow;
        self
    }

    /// Render as filled area graph instead of line (ratatui v0.30.1+)
    pub fn as_area(mut self, baseline: f64) -> Self {
        self.graph_type = ChartGraphType::Area;
        self.area_baseline = baseline;
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

/// Build a dataset from data and config
fn build_dataset<'a>(
    points: &'a [(f64, f64)],
    config: &ChartConfig,
) -> Dataset<'a> {
    let graph_type = match config.graph_type {
        ChartGraphType::Line => GraphType::Line,
        ChartGraphType::Area => GraphType::Area,
    };

    let mut dataset = Dataset::default()
        .name(config.title.clone())
        .marker(config.marker.into())
        .graph_type(graph_type)
        .style(Style::default().fg(config.color).bg(config.bg))
        .data(points);

    if config.graph_type == ChartGraphType::Area {
        dataset = dataset.fill_to_y(config.area_baseline);
    }

    dataset
}

/// Build a block with optional shadow from config
fn build_block(config: &ChartConfig) -> Block<'static> {
    let mut block = Block::default()
        .title(format!(" {} ", config.title))
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Plain)
        .border_style(Style::default().fg(config.border_color))
        .style(Style::default().bg(config.bg).fg(config.color));

    if let Some(shadow) = config.shadow.build() {
        block = block.shadow(shadow.offset(Offset::new(1, 1)));
    }

    block
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
    let dataset = build_dataset(&points, config);
    let block = build_block(config);

    let chart = Chart::new(vec![dataset])
        .block(block)
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

/// Data entry for multi-dataset overlay charts
pub struct DatasetEntry {
    pub name: String,
    pub data: Vec<(f64, f64)>,
    pub color: Color,
    pub marker: Option<ChartMarker>,
    /// If true, render as filled area with this baseline
    pub area_baseline: Option<f64>,
}

/// Render a multi-dataset overlay chart on the same axes.
///
/// This allows rendering multiple lines (e.g., CPU + Memory) on one chart for comparison.
///
/// # Example
/// ```ignore
/// let datasets = vec![
///     DatasetEntry { name: "CPU".into(), data: cpu_points, color: Color::Red, ..Default::default() },
///     DatasetEntry { name: "MEM".into(), data: mem_points, color: Color::Cyan, ..Default::default() },
/// ];
/// render_multi_dataset_chart(f, area, &datasets, &shared_config);
/// ```
pub fn render_multi_dataset_chart(
    f: &mut Frame,
    area: Rect,
    datasets: &[DatasetEntry],
    config: &ChartConfig,
) {
    let ratatui_datasets: Vec<Dataset> = datasets
        .iter()
        .map(|entry| {
            let marker = entry.marker.unwrap_or(config.marker);
            let graph_type = if entry.area_baseline.is_some() {
                GraphType::Area
            } else {
                match config.graph_type {
                    ChartGraphType::Line => GraphType::Line,
                    ChartGraphType::Area => GraphType::Area,
                }
            };
            let mut ds = Dataset::default()
                .name(entry.name.clone())
                .marker(marker.into())
                .graph_type(graph_type)
                .style(Style::default().fg(entry.color).bg(config.bg))
                .data(&entry.data);
            if let Some(baseline) = entry.area_baseline {
                ds = ds.fill_to_y(baseline);
            }
            ds
        })
        .collect();

    let block = build_block(config);

    let chart = Chart::new(ratatui_datasets)
        .block(block)
        .style(Style::default().bg(config.bg).fg(config.color))
        .x_axis(
            Axis::default()
                .bounds([0.0, config.max_x(datasets)])
                .style(
                    Style::default()
                        .fg(config.border_color)
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
                            .fg(config.border_color)
                            .add_modifier(Modifier::DIM),
                    ),
                    Span::styled(
                        format!("{}", config.max),
                        Style::default()
                            .fg(config.border_color)
                            .add_modifier(Modifier::DIM),
                    ),
                ])
                .style(
                    Style::default()
                        .fg(config.border_color)
                        .add_modifier(Modifier::DIM),
                ),
        );

    f.render_widget(chart, area);
}

trait MaxX {
    fn max_x(&self, datasets: &[DatasetEntry]) -> f64;
}

impl MaxX for ChartConfig {
    fn max_x(&self, datasets: &[DatasetEntry]) -> f64 {
        datasets
            .iter()
            .map(|d| d.data.iter().map(|&(x, _)| x).fold(0.0_f64, f64::max))
            .fold(0.0_f64, f64::max)
            .max(1.0)
    }
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

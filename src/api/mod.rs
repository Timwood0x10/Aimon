//! API module for HTTP server and headless output
//! Provides REST API endpoints, Prometheus metrics, and data export formats

pub mod export;
pub mod formatters;
pub mod prometheus;
pub mod server;

pub use export::{HeadlessExporter, OutputFormat};
pub use formatters::{format_compact_json, format_csv, format_json, format_prometheus};
pub use prometheus::PrometheusExporter;
pub use server::ApiServer;

//! HTTP API server
//! Serves system metrics via REST endpoints

use actix_web::{web, App, HttpResponse, HttpServer};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::api::formatters::{format_compact_json, format_csv, format_json, format_prometheus};
use crate::collectors::DataCollector;

/// Shared application state
struct AppState {
    collector: Arc<Mutex<DataCollector>>,
}

/// HTTP API server for system metrics
pub struct ApiServer;

impl ApiServer {
    /// Start the HTTP API server on the given address and port.
    /// The server runs in a spawned tokio task and returns immediately.
    pub async fn start(
        addr: &str,
        port: u16,
        collector: Arc<Mutex<DataCollector>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let state = web::Data::new(AppState { collector });
        let bind_addr = addr.to_string();

        log::info!("Starting API server on {}:{}", bind_addr, port);

        let server = HttpServer::new(move || {
            App::new()
                .app_data(state.clone())
                .route("/health", web::get().to(health_handler))
                .route("/api/metrics", web::get().to(metrics_json_handler))
                .route("/api/metrics/csv", web::get().to(metrics_csv_handler))
                .route("/api/metrics/compact", web::get().to(metrics_compact_handler))
                .route("/metrics", web::get().to(metrics_prometheus_handler))
        })
        .bind((bind_addr.as_str(), port))?;

        // Run server in a background task so the caller is not blocked
        actix_rt::spawn(async move {
            if let Err(e) = server.run().await {
                log::error!("API server error: {}", e);
            }
        });

        log::info!("API server started on port {}", port);
        Ok(())
    }
}

/// Health check endpoint
async fn health_handler() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({"status": "ok"}))
}

/// Full metrics as JSON
async fn metrics_json_handler(state: web::Data<AppState>) -> HttpResponse {
    let mut collector = state.collector.lock().await;
    match collector.collect_all_data().await {
        Ok(data) => {
            let body = format_json(&data);
            HttpResponse::Ok()
                .content_type("application/json")
                .body(body)
        }
        Err(e) => HttpResponse::InternalServerError()
            .json(serde_json::json!({"error": e.to_string()})),
    }
}

/// Metrics as CSV
async fn metrics_csv_handler(state: web::Data<AppState>) -> HttpResponse {
    let mut collector = state.collector.lock().await;
    match collector.collect_all_data().await {
        Ok(data) => {
            let body = format_csv(&data);
            HttpResponse::Ok()
                .content_type("text/csv")
                .body(body)
        }
        Err(e) => HttpResponse::InternalServerError()
            .json(serde_json::json!({"error": e.to_string()})),
    }
}

/// Compact metrics as JSON
async fn metrics_compact_handler(state: web::Data<AppState>) -> HttpResponse {
    let mut collector = state.collector.lock().await;
    match collector.collect_all_data().await {
        Ok(data) => {
            let body = format_compact_json(&data);
            HttpResponse::Ok()
                .content_type("application/json")
                .body(body)
        }
        Err(e) => HttpResponse::InternalServerError()
            .json(serde_json::json!({"error": e.to_string()})),
    }
}

/// Prometheus metrics endpoint
async fn metrics_prometheus_handler(state: web::Data<AppState>) -> HttpResponse {
    let mut collector = state.collector.lock().await;
    match collector.collect_all_data().await {
        Ok(data) => {
            let body = format_prometheus(&data);
            HttpResponse::Ok()
                .content_type("text/plain; version=0.0.4; charset=utf-8")
                .body(body)
        }
        Err(e) => HttpResponse::InternalServerError()
            .json(serde_json::json!({"error": e.to_string()})),
    }
}

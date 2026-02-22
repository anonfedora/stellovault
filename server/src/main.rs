//! StelloVault Backend Server
//!
//! This is the main Rust backend server for StelloVault, providing APIs for
//! user management, trade analytics, risk scoring, and integration with
//! Soroban smart contracts.

use axum::{
    http::{header, HeaderValue, Method},
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};
use tower_http::cors::CorsLayer;

mod handlers;
mod models;
mod routes;
mod config;
mod services;

const INDEXER_SUPERVISOR_MAX_BACKOFF_SECONDS: u64 = 30;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load environment variables
    dotenvy::dotenv().ok();

    // Create the app router
    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health_check))
        .merge(routes::user_routes())
        .merge(routes::escrow_routes())
        .merge(routes::analytics_routes())
        .layer(build_cors_layer());

    // Start and supervise the background Soroban event indexer.
    tokio::spawn(async move {
        let mut restart_count: u32 = 0;
        loop {
            let event_indexer =
                services::event_monitoring_service::EventMonitoringService::from_env().await;
            let handle = tokio::spawn(async move { event_indexer.start().await });

            match handle.await {
                Ok(Ok(())) => {
                    info!("event indexer exited cleanly; stopping supervisor");
                    break;
                }
                Ok(Err(services::event_monitoring_service::StartError::NoContractsConfigured)) => {
                    info!("Indexer disabled: no contract IDs set in environment");
                    break;
                }
                Err(join_error) => {
                    if join_error.is_panic() {
                        error!("event indexer panicked; restarting");
                    } else {
                        error!(error = %join_error, "event indexer task failed; restarting");
                    }
                }
            }

            restart_count = restart_count.saturating_add(1);
            let backoff_seconds = (2u64.saturating_pow(restart_count.min(5)))
                .min(INDEXER_SUPERVISOR_MAX_BACKOFF_SECONDS);
            warn!(restart_count, backoff_seconds, "event indexer restart backoff");
            sleep(Duration::from_secs(backoff_seconds)).await;
        }
    });

    // Get port from environment or default to 3001
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3001".to_string())
        .parse()
        .expect("PORT must be a number");

    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    tracing::info!("Server starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "StelloVault API Server"
}

async fn health_check() -> &'static str {
    "OK"
}

fn build_cors_layer() -> CorsLayer {
    let allowed_origins = std::env::var("CORS_ALLOWED_ORIGINS")
        .unwrap_or_else(|_| "http://localhost:3000".to_string())
        .split(',')
        .filter_map(|origin| origin.trim().parse::<HeaderValue>().ok())
        .collect::<Vec<_>>();

    CorsLayer::new()
        .allow_origin(allowed_origins)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE, header::ACCEPT])
        .allow_credentials(false)
}

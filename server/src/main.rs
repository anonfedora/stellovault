//! StelloVault Backend Server
//!
//! This is the main Rust backend server for StelloVault, providing APIs for
//! user management, trade analytics, risk scoring, and integration with
//! Soroban smart contracts.

use axum::{
    routing::get,
    Router,
};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use axum::http::{HeaderValue, Method};

// Re-declare modules for binary
mod app_state;
mod collateral;
mod escrow;
mod escrow_service;
mod event_listener;
mod handlers;
mod models;
mod oracle_service;
mod routes;
mod services;
mod websocket;

use app_state::AppState;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load environment variables
    dotenvy::dotenv().ok();

    // Get configuration from environment
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://localhost/stellovault".to_string());
    let horizon_url = std::env::var("HORIZON_URL")
        .unwrap_or_else(|_| "https://horizon-testnet.stellar.org".to_string());
    let network_passphrase = std::env::var("NETWORK_PASSPHRASE")
        .unwrap_or_else(|_| "Test SDF Network ; September 2015".to_string());
    let contract_id = std::env::var("CONTRACT_ID")
        .unwrap_or_else(|_| "STELLOVAULT_CONTRACT_ID".to_string());
    let webhook_secret = std::env::var("WEBHOOK_SECRET").ok();

    // Initialize database connection pool
    tracing::info!("Connecting to database...");
    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");
    
    tracing::info!("Database connected successfully");

    // Initialize WebSocket state
    let ws_state = websocket::WsState::new();

    // Initialize escrow service
    let escrow_service = Arc::new(escrow_service::EscrowService::new(
        db_pool.clone(),
        horizon_url.clone(),
        network_passphrase.clone(),
    ));

    // Initialize collateral service
    let collateral_service = Arc::new(collateral::CollateralService::new(
        Arc::new(db_pool.clone()),
    ));

    // Initialize oracle service
    let oracle_service = Arc::new(oracle_service::OracleService::new(
        db_pool.clone(),
    ));

    // Create shared app state
    let app_state = AppState::new(
        escrow_service.clone(),
        collateral_service.clone(),
        oracle_service.clone(),
        ws_state.clone(),
        webhook_secret,
    );

    // Start event listener in background
    let event_listener = event_listener::EventListener::new(
        horizon_url,
        contract_id,
        escrow_service.clone(),
        ws_state.clone(),
        db_pool.clone(),
    );
    tokio::spawn(async move {
        tracing::info!("Event listener task started");
        event_listener.start().await;
        tracing::error!("Event listener task exited unexpectedly");
    });

    // Start timeout detector in background
    let escrow_service_timeout = escrow_service.clone();
    let ws_state_timeout = ws_state.clone();
    tokio::spawn(async move {
        tracing::info!("Timeout detector task started");
        event_listener::timeout_detector(escrow_service_timeout, ws_state_timeout).await;
        tracing::error!("Timeout detector task exited unexpectedly");
    });

    // Create the app router
    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health_check))
        .route("/ws", get(websocket::ws_handler))
        .merge(routes::user_routes())
        .merge(routes::escrow_routes())
        .merge(routes::collateral_routes())
        .merge(routes::oracle_routes())
        .merge(routes::analytics_routes())
        .with_state(app_state)
        .layer(configure_cors());

    // Get port from environment or default to 3001
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3001".to_string())
        .parse()
        .expect("PORT must be a number");

    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    tracing::info!("Server starting on {}", addr);
    tracing::info!("WebSocket available at ws://{}/ws", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "StelloVault API Server"
}

async fn health_check() -> &'static str {
    "OK"
}

fn configure_cors() -> CorsLayer {
    let allowed_origins_str = std::env::var("CORS_ALLOWED_ORIGINS").unwrap_or_default();
    
    if allowed_origins_str.is_empty() {
        tracing::warn!("CORS_ALLOWED_ORIGINS not set, allowing all origins (permissive)");
        return CorsLayer::permissive();
    }

    let origins: Vec<HeaderValue> = allowed_origins_str
        .split(',')
        .map(|s| s.trim().parse().expect("Invalid CORS origin"))
        .collect();

    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(Any)
}
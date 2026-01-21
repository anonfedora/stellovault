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
use tower_http::cors::CorsLayer;

mod app_state;
mod escrow;
mod escrow_service;
mod event_listener;
mod handlers;
mod models;
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

    // Initialize database connection pool
    tracing::info!("Connecting to database...");
    let db_pool = match PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
    {
        Ok(pool) => {
            tracing::info!("Database connected successfully");
            pool
        }
        Err(e) => {
            tracing::error!("Failed to connect to database: {}", e);
            tracing::warn!("Running without database - endpoints will fail");
            // Create a dummy pool that will fail on use
            PgPoolOptions::new()
                .max_connections(1)
                .connect("postgresql://localhost/nonexistent")
                .await
                .expect("Database connection required")
        }
    };

    // Initialize WebSocket state
    let ws_state = websocket::WsState::new();

    // Initialize escrow service
    let escrow_service = Arc::new(escrow_service::EscrowService::new(
        db_pool.clone(),
        horizon_url.clone(),
        network_passphrase.clone(),
    ));

    // Create shared app state
    let app_state = AppState::new(escrow_service.clone(), ws_state.clone());

    // Start event listener in background
    let event_listener = event_listener::EventListener::new(
        horizon_url,
        contract_id,
        escrow_service.clone(),
        ws_state.clone(),
        db_pool.clone(),
    );
    tokio::spawn(async move {
        event_listener.start().await;
    });

    // Start timeout detector in background
    tokio::spawn(event_listener::timeout_detector(
        escrow_service.clone(),
        ws_state.clone(),
    ));

    // Create the app router
    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health_check))
        .route("/ws", get(websocket::ws_handler))
        .merge(routes::user_routes())
        .merge(routes::escrow_routes())
        .merge(routes::analytics_routes())
        .with_state(app_state)
        .layer(CorsLayer::permissive()); // TODO: Configure CORS properly

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
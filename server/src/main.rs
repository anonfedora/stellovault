//! StelloVault Backend Server
//!
//! This is the main Rust backend server for StelloVault, providing APIs for
//! user management, trade analytics, risk scoring, and integration with
//! Soroban smart contracts.

use axum::http::{HeaderValue, Method};
use axum::{routing::get, Router};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use std::sync::Arc;

use tokio::signal;
use tower_http::cors::{Any, CorsLayer};

mod auth;
mod collateral;
mod config;
mod error;
mod escrow;
mod handlers;
mod loan;
mod loan_service;
mod middleware;
mod models;
mod routes;
mod services;
mod state;

// Domain modules
mod websocket;
mod indexer;

use config::Config;
use escrow::{timeout_detector, EscrowService, EventListener};
use middleware::RateLimiter;
use state::AppState;

#[tokio::main]
async fn main() {
    // Load configuration
    let config = match Config::from_env() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to load configuration: {}", e);
            std::process::exit(1);
        }
    };

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&config.log_level)),
        )
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();

    // Load environment variables
    dotenvy::dotenv().ok();

    // Get configuration from environment
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://localhost/stellovault".to_string());
    let horizon_url = std::env::var("HORIZON_URL")
        .unwrap_or_else(|_| "https://horizon-testnet.stellar.org".to_string());
    let network_passphrase = std::env::var("NETWORK_PASSPHRASE")
        .unwrap_or_else(|_| "Test SDF Network ; September 2015".to_string());
    let contract_id =
        std::env::var("CONTRACT_ID").unwrap_or_else(|_| "STELLOVAULT_CONTRACT_ID".to_string());
    
    // Contract IDs for Indexer
    let collateral_id = std::env::var("COLLATERAL_CONTRACT_ID").unwrap_or_else(|_| contract_id.clone());
    let escrow_id = std::env::var("ESCROW_CONTRACT_ID").unwrap_or_else(|_| contract_id.clone());
    let loan_id = std::env::var("LOAN_CONTRACT_ID").unwrap_or_else(|_| contract_id.clone());
    
    let soroban_rpc_url = std::env::var("SOROBAN_RPC_URL")
        .unwrap_or_else(|_| "https://soroban-testnet.stellar.org".to_string());

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
    let escrow_service = Arc::new(EscrowService::new(
        db_pool.clone(),
        config.horizon_url.clone(),
        config.network_passphrase.clone(),
    ));

    // Initialize collateral service
    let collateral_service = Arc::new(collateral::CollateralService::new(Arc::new(
        db_pool.clone(),
    )));

    // Initialize loan service
    let loan_service = Arc::new(loan_service::LoanService::new(db_pool.clone()));

    // Initialize auth service
    let auth_service = Arc::new(auth::AuthService::new(
        db_pool.clone(),
        config.jwt_secret.clone(),
        config.auth_nonce_ttl_seconds,
        config.jwt_access_token_ttl_seconds,
        config.jwt_refresh_token_ttl_days,
    ));

    // Create shared app state
    let app_state = AppState::new(
        escrow_service.clone(),
        collateral_service.clone(),
        loan_service,
        auth_service,
        ws_state.clone(),
        config.webhook_secret.clone(),
    );

    // Start event listener in background
    // Start Indexer Service
    let mut contracts_map = std::collections::HashMap::new();
    contracts_map.insert("collateral".to_string(), collateral_id);
    contracts_map.insert("escrow".to_string(), escrow_id);
    contracts_map.insert("loan".to_string(), loan_id);

    let indexer_service = Arc::new(indexer::IndexerService::new(
        soroban_rpc_url,
        db_pool.clone(),
        contracts_map,
        ws_state.clone(),
    ));

    tokio::spawn(async move {
        indexer_service.start().await;
    });

    // Start timeout detector in background
    let escrow_service_timeout = escrow_service.clone();
    let ws_state_timeout = ws_state.clone();
    tokio::spawn(async move {
        tracing::info!("Timeout detector task started");
        timeout_detector(escrow_service_timeout, ws_state_timeout).await;
        tracing::error!("Timeout detector task exited unexpectedly");
    });

    // Clone db_pool for health check
    let health_db_pool = db_pool.clone();

    // Initialize rate limiter (100 requests per second per client)
    let rate_limiter = RateLimiter::new(100);

    // Create the app router
    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(move || health_check(health_db_pool.clone())))
        .route("/ws", get(websocket::ws_handler))
        .merge(routes::auth_routes())
        .merge(routes::wallet_routes())
        .merge(routes::user_routes())
        .merge(routes::escrow_routes())
        .merge(routes::collateral_routes())
        .merge(routes::loan_routes())
        .merge(routes::analytics_routes())
        .with_state(app_state)
        .layer(axum::middleware::from_fn(middleware::security_headers))
        .layer(axum::middleware::from_fn(middleware::request_tracing))
        .layer(axum::middleware::from_fn(move |req, next| {
            let limiter = rate_limiter.clone();
            middleware::rate_limit_layer(limiter)(req, next)
        }))
        .layer(configure_cors());

    let addr = SocketAddr::from(([127, 0, 0, 1], config.port));

    tracing::info!("Server listening on {}", addr);
    tracing::info!("WebSocket available at ws://{}/ws", addr);
    tracing::info!("Health check at http://{}/health", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    // Serve with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    tracing::info!("Server shutdown complete");
}

async fn root() -> &'static str {
    "StelloVault API Server"
}

/// Health check response
#[derive(serde::Serialize)]
struct HealthResponse {
    status: String,
    database: String,
    version: String,
}

/// Health check endpoint
async fn health_check(pool: sqlx::PgPool) -> axum::Json<HealthResponse> {
    let db_status = match sqlx::query("SELECT 1").execute(&pool).await {
        Ok(_) => "connected".to_string(),
        Err(e) => format!("error: {}", e),
    };

    let status = if db_status == "connected" {
        "healthy"
    } else {
        "unhealthy"
    };

    axum::Json(HealthResponse {
        status: status.to_string(),
        database: db_status,
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

fn configure_cors() -> CorsLayer {
    let allowed_origins_str = std::env::var("CORS_ALLOWED_ORIGINS").unwrap_or_default();

    if allowed_origins_str.is_empty() {
        tracing::warn!("CORS_ALLOWED_ORIGINS not set, allowing all origins (permissive)");
        return CorsLayer::permissive();
    }

    let origins: Vec<HeaderValue> = allowed_origins_str
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();

    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(Any)
}

/// Graceful shutdown signal handler
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C, initiating graceful shutdown...");
        }
        _ = terminate => {
            tracing::info!("Received SIGTERM, initiating graceful shutdown...");
        }
    }
}

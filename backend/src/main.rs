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
use tower_http::cors::{CorsLayer, Any};

// Use modules from lib
use stellovault_server::{
    auth::AuthService,
    collateral::{CollateralIndexer, CollateralService},
    config::Config,
    escrow::{timeout_detector, EscrowService},
    indexer::IndexerService,
    loan_service::LoanService,
    middleware::{self, RateLimiter},
    oracle::OracleService,
    routes,
    services::RiskEngine,
    state::AppState,
    websocket::{self, WsState},
};

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
    let ws_state = WsState::new();

    // Initialize collateral service
    let collateral_service = CollateralService::new(
        db_pool.clone(),
        config.soroban_rpc_url.clone(),
        config.contract_id.clone(),
    );

    // Initialize escrow service
    let escrow_service = Arc::new(EscrowService::new(
        db_pool.clone(),
        config.horizon_url.clone(),
        config.network_passphrase.clone(),
        collateral_service.clone(),
    ));

    let collateral_service = Arc::new(collateral_service);

    // Initialize oracle service
    let oracle_service = Arc::new(OracleService::new(
        db_pool.clone(),
        horizon_url.clone(),
        network_passphrase.clone(),
        soroban_rpc_url.clone(),
    ));

    // Initialize loan service
    let loan_service = Arc::new(LoanService::new(
        db_pool.clone(),
    ));

    // Initialize auth service (with default TTL values)
    let auth_service = Arc::new(AuthService::new(
        db_pool.clone(),
        config.jwt_secret.clone(),
        300,      // nonce_ttl_seconds: 5 minutes
        3600,     // access_token_ttl_seconds: 1 hour
        30,       // refresh_token_ttl_days: 30 days
    ));

    // Initialize risk engine
    let risk_engine = Arc::new(RiskEngine::new(
        db_pool.clone(),
    ));

    // Create shared app state
    let app_state = AppState::new(
        escrow_service.clone(),
        collateral_service.clone(),
        loan_service.clone(),
        auth_service.clone(),
        risk_engine.clone(),
        oracle_service.clone(),
        ws_state.clone(),
        config.webhook_secret.clone(),
    );

    // Start event listener in background
    // Start Indexer Service
    let mut contracts_map = std::collections::HashMap::new();
    contracts_map.insert("collateral".to_string(), collateral_id);
    contracts_map.insert("escrow".to_string(), escrow_id);
    contracts_map.insert("loan".to_string(), loan_id);

    let indexer_service = Arc::new(IndexerService::new(
        soroban_rpc_url,
        db_pool.clone(),
        contracts_map,
        ws_state.clone(),
    ));

    tokio::spawn(async move {
        indexer_service.start().await;
    });

    // Start collateral indexer
    let collateral_indexer = CollateralIndexer::new(
        db_pool.clone(),
        config.soroban_rpc_url.clone(),
        config.contract_id.clone(),
    );
    tokio::spawn(async move {
        tracing::info!("Collateral indexer task started");
        collateral_indexer.start().await;
    });

    // Start timeout detector in background
    let escrow_service_timeout = escrow_service.clone();
    let ws_state_timeout = ws_state.clone();
    tokio::spawn(async move {
        tracing::info!("Timeout detector task started");
        timeout_detector(escrow_service_timeout, ws_state_timeout).await;
        tracing::error!("Timeout detector task exited unexpectedly");
    });

    // Start WebSocket heartbeat pruner (checks every 30 seconds)
    let ws_state_heartbeat = ws_state.clone();
    tokio::spawn(async move {
        tracing::info!("WebSocket heartbeat pruner started");
        websocket::heartbeat_pruner(ws_state_heartbeat, 30).await;
        tracing::error!("Heartbeat pruner task exited unexpectedly");
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
        .merge(routes::oracle_routes())
        .merge(routes::analytics_routes())
        .merge(routes::risk_routes())
        .merge(routes::loan_routes())
        .merge(routes::document_routes())
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
    websocket: WebSocketStats,
}

/// WebSocket statistics
#[derive(serde::Serialize)]
struct WebSocketStats {
    connected_clients: usize,
    active_rooms: usize,
    buffered_events: usize,
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

    // WebSocket stats are static placeholders since we don't have access to ws_state here
    // In production, this would be fetched from shared state
    let ws_stats = WebSocketStats {
        connected_clients: 0,
        active_rooms: 0,
        buffered_events: 0,
    };

    axum::Json(HealthResponse {
        status: status.to_string(),
        database: db_status,
        version: env!("CARGO_PKG_VERSION").to_string(),
        websocket: ws_stats,
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
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
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

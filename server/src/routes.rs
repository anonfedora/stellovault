//! Route definitions for StelloVault API

use axum::{routing::get, Router};

use crate::app_state::AppState;
use crate::handlers::*;

// User routes
pub fn user_routes() -> Router<AppState> {
    Router::new()
        .route("/api/users/:id", get(get_user))
        .route("/api/users", axum::routing::post(create_user))
}

// Escrow routes
pub fn escrow_routes() -> Router<AppState> {
    Router::new()
        .route("/api/escrows", axum::routing::post(create_escrow))
        .route("/api/escrows", get(list_escrows))
        .route("/api/escrows/:id", get(get_escrow))
        .route("/api/escrows/webhook", axum::routing::post(webhook_escrow_update))
}

// Collateral routes
pub fn collateral_routes() -> Router<AppState> {
    Router::new()
        .route("/api/collateral", axum::routing::post(create_collateral))
        .route("/api/collateral", get(list_collateral))
        .route("/api/collateral/:id", get(get_collateral))
        .route("/api/collateral/metadata/:hash", get(get_collateral_by_metadata))
}

// Oracle routes
pub fn oracle_routes() -> Router<AppState> {
    Router::new()
        .route("/api/oracles", axum::routing::post(register_oracle))
        .route("/api/oracles", get(list_oracles))
        .route("/api/oracles/:address", get(get_oracle))
        .route("/api/oracles/:address/deactivate", axum::routing::post(deactivate_oracle))
        .route("/api/confirmations", axum::routing::post(submit_confirmation))
        .route("/api/confirmations/:escrow_id", get(get_confirmations))
        .route("/api/oracles/metrics", get(get_oracle_metrics))
}

// Analytics routes
pub fn analytics_routes() -> Router<AppState> {
    Router::new()
        .route("/api/analytics", get(get_analytics))
}
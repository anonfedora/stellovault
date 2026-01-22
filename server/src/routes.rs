//! Route definitions for StelloVault API

use axum::{routing::get, Router};

use crate::handlers::*;

// User routes
pub fn user_routes() -> Router {
    Router::new()
        .route("/api/users/:id", get(get_user))
        .route("/api/users", axum::routing::post(create_user))
}

// Escrow routes
pub fn escrow_routes() -> Router {
    Router::new()
        .route("/api/escrows", get(get_escrows))
        // TODO: Add more escrow routes
}

// Analytics routes
pub fn analytics_routes() -> Router {
    Router::new()
        .route("/api/analytics", get(get_analytics))
        // TODO: Add more analytics routes
}

// Oracle routes
pub fn oracle_routes() -> Router {
    Router::new()
        .route("/oracle/confirm", axum::routing::post(oracle::confirm_handler))
}
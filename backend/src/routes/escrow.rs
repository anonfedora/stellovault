//! Escrow route definitions

use axum::{
    routing::{get, post, put},
    Router,
};

use crate::handlers::*;
use crate::state::AppState;

pub fn escrow_routes() -> Router<AppState> {
    Router::new()
        // Core CRUD
        .route("/api/v1/escrows", post(create_escrow))
        .route("/api/v1/escrows", get(list_escrows))
        .route("/api/v1/escrows/:id", get(get_escrow))
        // Status update (state machine)
        .route("/api/v1/escrows/:id/status", put(update_escrow_status))
        // Lifecycle actions
        .route("/api/v1/escrows/:id/fund", post(fund_escrow))
        .route("/api/v1/escrows/:id/release", post(release_escrow))
        .route("/api/v1/escrows/:id/dispute", post(dispute_escrow))
        // Audit trail
        .route("/api/v1/escrows/:id/history", get(get_escrow_history))
        // Webhook (keep for backward compat)
        .route("/api/v1/escrows/webhook", post(webhook_escrow_update))
        // Legacy routes (keep for backward compat)
        .route("/api/escrows", post(create_escrow))
        .route("/api/escrows", get(list_escrows))
        .route("/api/escrows/:id", get(get_escrow))
        .route("/api/escrows/webhook", post(webhook_escrow_update))
}

//! Escrow route definitions

use axum::{routing::get, Router};

use crate::handlers::*;
use crate::state::AppState;

pub fn escrow_routes() -> Router<AppState> {
    Router::new()
        .route("/api/escrows", axum::routing::post(create_escrow))
        .route("/api/escrows", get(list_escrows))
        .route("/api/escrows/:id", get(get_escrow))
        .route(
            "/api/escrows/webhook",
            axum::routing::post(webhook_escrow_update),
        )
}

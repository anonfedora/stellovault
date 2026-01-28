//! Oracle route definitions for StelloVault API
//!
//! I'm defining all oracle-related routes here, following the existing routing pattern.

use axum::{
    routing::{get, post},
    Router,
};

use crate::handlers::oracle;
use crate::state::AppState;

/// Create oracle routes
pub fn oracle_routes() -> Router<AppState> {
    Router::new()
        .route("/oracle/confirm", post(oracle::confirm_oracle_event))
        .route("/oracle/events", get(oracle::list_oracle_events))
        .route("/oracle/events/:id", get(oracle::get_oracle_event))
        .route("/oracle/dispute", post(oracle::flag_dispute))
}

//! Analytics route definitions

use axum::{routing::get, Router};

use crate::handlers::analytics::get_analytics;
use crate::state::AppState;

pub fn analytics_routes() -> Router<AppState> {
    Router::new().route("/api/analytics", get(get_analytics))
}

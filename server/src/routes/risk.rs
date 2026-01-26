//! Risk scoring route definitions

use axum::{routing::{get, post}, Router};

use crate::handlers::risk::{get_risk_history, get_risk_score, simulate_risk_score};
use crate::state::AppState;

pub fn risk_routes() -> Router<AppState> {
    Router::new()
        .route("/api/risk/{wallet}", get(get_risk_score))
        .route("/api/risk/{wallet}/history", get(get_risk_history))
        .route("/api/risk/{wallet}/simulate", post(simulate_risk_score))
}

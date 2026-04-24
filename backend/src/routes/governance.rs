//! Governance route definitions

use axum::{
    routing::{get, post},
    Router,
};

use crate::handlers::governance::{
    cast_vote, create_proposal, execute_proposal, get_governance_metrics, get_proposal,
    get_voting_power, list_proposals,
};
use crate::state::AppState;

pub fn governance_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/governance/proposals", post(create_proposal).get(list_proposals))
        .route("/api/v1/governance/proposals/:id", get(get_proposal))
        .route("/api/v1/governance/proposals/:id/vote", post(cast_vote))
        .route("/api/v1/governance/voting-power/:address", get(get_voting_power))
        .route("/api/v1/governance/execute/:id", post(execute_proposal))
        .route("/api/v1/governance/metrics", get(get_governance_metrics))
}

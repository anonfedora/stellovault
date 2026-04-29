//! Loan route definitions

use axum::Router;

use crate::handlers::loan::*;
use crate::state::AppState;

pub fn loan_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/loans", axum::routing::get(list_loans))
        .route("/api/v1/loans", axum::routing::post(create_loan))
        .route("/api/v1/loans/:id", axum::routing::get(get_loan))
        .route("/api/v1/loans/:id/repay", axum::routing::post(make_repayment))
        .route("/api/v1/loans/:id/schedule", axum::routing::get(get_repayment_schedule))
        .route("/api/v1/loans/:id/extend", axum::routing::post(request_extension))
        .route("/api/v1/loans/:id/history", axum::routing::get(get_loan_history))
}

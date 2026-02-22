//! Loan route definitions

use axum::Router;

use crate::handlers::*;
use crate::state::AppState;

pub fn loan_routes() -> Router<AppState> {
    Router::new()
        .route("/api/loans", axum::routing::get(list_loans))
        .route("/api/loans/:id", axum::routing::get(get_loan))
        .route("/api/loans", axum::routing::post(create_loan))
        .route(
            "/api/loans/repayment",
            axum::routing::post(record_repayment),
        )
}

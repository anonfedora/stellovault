//! Document verification route definitions

use axum::{routing::post, Router};

use crate::handlers::document::verify_document;
use crate::state::AppState;

pub fn document_routes() -> Router<AppState> {
    Router::new()
        .route("/api/documents/verify", post(verify_document))
}

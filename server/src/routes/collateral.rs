//! Collateral route definitions

use axum::{routing::get, Router};

use crate::handlers::collateral::{create_collateral, get_collateral, get_collateral_by_metadata, list_collateral};
use crate::state::AppState;

pub fn collateral_routes() -> Router<AppState> {
    Router::new()
        .route("/api/collateral", axum::routing::post(create_collateral))
        .route("/api/collateral", get(list_collateral))
        .route("/api/collateral/:id", get(get_collateral))
        .route(
            "/api/collateral/metadata/:hash",
            get(get_collateral_by_metadata),
        )
}

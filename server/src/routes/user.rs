//! User route definitions

use axum::{routing::get, Router};

use crate::handlers::user::{create_user, get_user};
use crate::state::AppState;

pub fn user_routes() -> Router<AppState> {
    Router::new()
        .route("/api/users/:id", get(get_user))
        .route("/api/users", axum::routing::post(create_user))
}

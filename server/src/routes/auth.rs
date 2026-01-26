//! Authentication routes

use axum::{
    routing::{get, post},
    Router,
};

use crate::handlers::auth;
use crate::state::AppState;

/// Create authentication routes
pub fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/auth/challenge", post(auth::request_challenge))
        .route("/auth/verify", post(auth::verify_signature))
        .route("/auth/refresh", post(auth::refresh_token))
        .route("/auth/logout", post(auth::logout))
        .route("/auth/logout-all", post(auth::logout_all))
        .route("/auth/me", get(auth::get_current_user))
}

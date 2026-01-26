//! Wallet management routes

use axum::{
    routing::{delete, get, patch, post, put},
    Router,
};

use crate::handlers::wallet;
use crate::state::AppState;

/// Create wallet management routes
pub fn wallet_routes() -> Router<AppState> {
    Router::new()
        .route("/wallets", get(wallet::list_wallets))
        .route("/wallets/challenge", post(wallet::wallet_challenge))
        .route("/wallets", post(wallet::link_wallet))
        .route("/wallets/:id", delete(wallet::unlink_wallet))
        .route("/wallets/:id", patch(wallet::update_wallet))
        .route("/wallets/:id/primary", put(wallet::set_primary_wallet))
}

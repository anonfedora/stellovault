//! Wallet management HTTP handlers
//!
//! Endpoints for managing linked wallets.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::error::ApiError;
use crate::handlers::AuthenticatedUser;
use crate::models::{ChallengeResponse, LinkWalletRequest, UpdateWalletRequest, WalletResponse};
use crate::state::AppState;

/// GET /wallets - List user's linked wallets
pub async fn list_wallets(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<Vec<WalletResponse>>, ApiError> {
    let wallets = state
        .auth_service
        .get_user_wallets(user.user_id)
        .await
        .map_err(|e| ApiError::InternalError(e.to_string()))?;

    let response: Vec<WalletResponse> = wallets.into_iter().map(|w| w.into()).collect();
    Ok(Json(response))
}

/// POST /wallets/challenge - Request challenge for linking a new wallet
pub async fn wallet_challenge(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Json(req): Json<WalletChallengeRequest>,
) -> Result<Json<ChallengeResponse>, ApiError> {
    let challenge = state
        .auth_service
        .generate_challenge(&req.wallet_address)
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    Ok(Json(challenge))
}

#[derive(Debug, serde::Deserialize)]
pub struct WalletChallengeRequest {
    pub wallet_address: String,
}

/// POST /wallets - Link a new wallet to the authenticated user
pub async fn link_wallet(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(req): Json<LinkWalletRequest>,
) -> Result<(StatusCode, Json<WalletResponse>), ApiError> {
    let wallet = state
        .auth_service
        .link_wallet(
            user.user_id,
            &req.wallet_address,
            &req.nonce,
            &req.signature,
            req.label,
        )
        .await
        .map_err(|e| match e.to_string().as_str() {
            s if s.contains("already linked") => ApiError::Conflict(e.to_string()),
            s if s.contains("Invalid signature") => ApiError::Unauthorized(e.to_string()),
            s if s.contains("Nonce") => ApiError::BadRequest(e.to_string()),
            _ => ApiError::InternalError(e.to_string()),
        })?;

    Ok((StatusCode::CREATED, Json(wallet.into())))
}

/// DELETE /wallets/:id - Unlink a wallet from the authenticated user
pub async fn unlink_wallet(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(wallet_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state
        .auth_service
        .unlink_wallet(user.user_id, wallet_id)
        .await
        .map_err(|e| match e.to_string().as_str() {
            s if s.contains("primary") => ApiError::BadRequest(e.to_string()),
            s if s.contains("at least one") => ApiError::BadRequest(e.to_string()),
            _ => ApiError::NotFound(e.to_string()),
        })?;

    Ok(StatusCode::NO_CONTENT)
}

/// PUT /wallets/:id/primary - Set a wallet as primary
pub async fn set_primary_wallet(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(wallet_id): Path<Uuid>,
) -> Result<Json<WalletResponse>, ApiError> {
    let wallet = state
        .auth_service
        .set_primary_wallet(user.user_id, wallet_id)
        .await
        .map_err(|e| ApiError::NotFound(e.to_string()))?;

    Ok(Json(wallet.into()))
}

/// PATCH /wallets/:id - Update wallet label
pub async fn update_wallet(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(wallet_id): Path<Uuid>,
    Json(req): Json<UpdateWalletRequest>,
) -> Result<Json<WalletResponse>, ApiError> {
    // Get the wallet first to verify ownership
    let wallets = state
        .auth_service
        .get_user_wallets(user.user_id)
        .await
        .map_err(|e| ApiError::InternalError(e.to_string()))?;

    let wallet = wallets
        .into_iter()
        .find(|w| w.id == wallet_id)
        .ok_or_else(|| ApiError::NotFound("Wallet not found".to_string()))?;

    // Update label in database
    sqlx::query(
        r#"
        UPDATE wallets SET label = $1, updated_at = NOW() WHERE id = $2
        "#,
    )
    .bind(&req.label)
    .bind(wallet_id)
    .execute(state.auth_service.db_pool())
    .await
    .map_err(|e| ApiError::InternalError(e.to_string()))?;

    Ok(Json(WalletResponse {
        id: wallet.id,
        wallet_address: wallet.wallet_address,
        is_primary: wallet.is_primary,
        label: req.label,
        verified_at: wallet.verified_at,
    }))
}

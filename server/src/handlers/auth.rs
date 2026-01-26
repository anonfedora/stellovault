//! Authentication HTTP handlers
//!
//! Endpoints for wallet-based authentication.

use axum::{
    extract::{ConnectInfo, State},
    http::StatusCode,
    Json,
};
use std::net::SocketAddr;

use super::AuthenticatedUser;
use crate::error::ApiError;
use crate::models::{
    AuthTokensResponse, ChallengeRequest, ChallengeResponse, RefreshTokenRequest, UserResponse,
};
use crate::state::AppState;

/// Request body for signature verification
#[derive(Debug, serde::Deserialize)]
pub struct VerifyRequest {
    pub wallet_address: String,
    pub nonce: String,
    pub signature: String,
}

/// POST /auth/challenge - Request a nonce for wallet authentication
pub async fn request_challenge(
    State(state): State<AppState>,
    Json(req): Json<ChallengeRequest>,
) -> Result<Json<ChallengeResponse>, ApiError> {
    let challenge = state
        .auth_service
        .generate_challenge(&req.wallet_address)
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    Ok(Json(challenge))
}

/// POST /auth/verify - Verify signed nonce and issue tokens
pub async fn verify_signature(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(req): Json<VerifyRequest>,
) -> Result<Json<AuthTokensResponse>, ApiError> {
    let tokens = state
        .auth_service
        .verify_signature(
            &req.wallet_address,
            &req.nonce,
            &req.signature,
            None, // device_info
            Some(addr.ip().to_string()),
            None, // user_agent (we could extract this from headers)
        )
        .await
        .map_err(|e| match e.to_string().as_str() {
            s if s.contains("Invalid signature") => ApiError::Unauthorized(e.to_string()),
            s if s.contains("Nonce") => ApiError::BadRequest(e.to_string()),
            _ => ApiError::InternalError(e.to_string()),
        })?;

    Ok(Json(tokens))
}

/// POST /auth/refresh - Refresh access token using refresh token
pub async fn refresh_token(
    State(state): State<AppState>,
    Json(req): Json<RefreshTokenRequest>,
) -> Result<Json<AuthTokensResponse>, ApiError> {
    let tokens = state
        .auth_service
        .refresh_tokens(&req.refresh_token)
        .await
        .map_err(|e| match e.to_string().as_str() {
            s if s.contains("Invalid") || s.contains("Session") => {
                ApiError::Unauthorized(e.to_string())
            }
            _ => ApiError::InternalError(e.to_string()),
        })?;

    Ok(Json(tokens))
}

/// POST /auth/logout - Revoke current session
pub async fn logout(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<StatusCode, ApiError> {
    state
        .auth_service
        .revoke_session(&user.jti)
        .await
        .map_err(|e| ApiError::InternalError(e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

/// GET /auth/me - Get current authenticated user
pub async fn get_current_user(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<UserResponse>, ApiError> {
    let user = state
        .auth_service
        .get_user_by_id(user.user_id)
        .await
        .map_err(|e| ApiError::NotFound(e.to_string()))?;

    Ok(Json(user.into()))
}

/// POST /auth/logout-all - Revoke all sessions for current user
pub async fn logout_all(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<LogoutAllResponse>, ApiError> {
    let revoked_count = state
        .auth_service
        .revoke_all_sessions(user.user_id)
        .await
        .map_err(|e| ApiError::InternalError(e.to_string()))?;

    Ok(Json(LogoutAllResponse {
        revoked_sessions: revoked_count,
    }))
}

#[derive(Debug, serde::Serialize)]
pub struct LogoutAllResponse {
    pub revoked_sessions: u64,
}

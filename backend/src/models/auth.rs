//! Authentication models for StelloVault

use serde::{Deserialize, Serialize};
use sqlx::types::chrono::{DateTime, Utc};
use uuid::Uuid;

use super::UserRole;

/// Wallet linked to a user
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct Wallet {
    pub id: Uuid,
    pub user_id: Uuid,
    pub wallet_address: String,
    pub is_primary: bool,
    pub label: Option<String>,
    pub verified_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Authentication nonce for challenge-response
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct AuthNonce {
    pub id: Uuid,
    pub nonce: String,
    pub wallet_address: String,
    pub message: String,
    pub expires_at: DateTime<Utc>,
    pub used: bool,
    pub used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Authentication session for JWT tracking
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct AuthSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub jti: String,
    pub refresh_token_hash: String,
    pub device_info: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub revoked: bool,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// Request/Response DTOs
// ============================================================================

/// Request for authentication challenge
#[derive(Debug, Deserialize)]
pub struct ChallengeRequest {
    pub wallet_address: String,
}

/// Response containing the authentication challenge
#[derive(Debug, Serialize)]
pub struct ChallengeResponse {
    pub nonce: String,
    pub message: String,
    pub expires_at: DateTime<Utc>,
}

/// Request to verify a signed message
#[derive(Debug, Deserialize)]
pub struct VerifyRequest {
    pub wallet_address: String,
    pub nonce: String,
    pub signature: String, // Base64-encoded signature
}

/// Auth tokens response
#[derive(Debug, Serialize)]
pub struct AuthTokensResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub user: UserResponse,
}

/// User response (sanitized for API)
#[derive(Debug, Serialize, Clone)]
pub struct UserResponse {
    pub id: Uuid,
    pub primary_wallet_address: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub role: UserRole,
    pub created_at: DateTime<Utc>,
}

/// Refresh token request
#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

/// Wallet list response
#[derive(Debug, Serialize)]
pub struct WalletResponse {
    pub id: Uuid,
    pub wallet_address: String,
    pub is_primary: bool,
    pub label: Option<String>,
    pub verified_at: DateTime<Utc>,
}

/// Request to link a new wallet
#[derive(Debug, Deserialize)]
pub struct LinkWalletRequest {
    pub wallet_address: String,
    pub signature: String,
    pub nonce: String,
    pub label: Option<String>,
}

/// Request to update wallet
#[derive(Debug, Deserialize)]
pub struct UpdateWalletRequest {
    pub label: Option<String>,
}

impl From<Wallet> for WalletResponse {
    fn from(wallet: Wallet) -> Self {
        Self {
            id: wallet.id,
            wallet_address: wallet.wallet_address,
            is_primary: wallet.is_primary,
            label: wallet.label,
            verified_at: wallet.verified_at,
        }
    }
}

use serde::{Deserialize, Serialize};
use sqlx::types::chrono::{DateTime, Utc};
use uuid::Uuid;

/// Collateral token model
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct CollateralToken {
    pub id: Uuid,
    pub token_id: String, // Soroban contract token ID
    pub owner_id: Uuid,
    pub asset_type: AssetType,
    pub asset_value: i64,
    pub metadata_hash: String,
    pub fractional_shares: i32,
    pub status: TokenStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Asset types
#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, Copy, PartialEq, Eq)]
#[sqlx(type_name = "asset_type", rename_all = "UPPERCASE")]
pub enum AssetType {
    Invoice,
    Commodity,
    Receivable,
}

/// Token status
#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, Copy, PartialEq, Eq)]
#[sqlx(type_name = "token_status", rename_all = "lowercase")]
pub enum TokenStatus {
    Active,
    Locked,  // Locked in escrow
    Burned,
}

/// Request DTO for creating collateral
#[derive(Debug, Deserialize)]
pub struct CreateCollateralRequest {
    pub token_id: String,
    pub owner_id: Uuid,
    pub asset_type: AssetType,
    pub asset_value: i64,
    pub metadata_hash: String,
    pub fractional_shares: i32,
}

/// Response DTO for creating collateral
#[derive(Debug, Serialize)]
pub struct CreateCollateralResponse {
    pub id: Uuid,
    pub token_id: String,
    pub status: TokenStatus,
    pub tx_hash: String,
}

/// Query parameters for listing collateral
#[derive(Debug, Deserialize)]
pub struct ListCollateralQuery {
    pub owner_id: Option<Uuid>,
    pub asset_type: Option<AssetType>,
    pub status: Option<TokenStatus>,
    pub page: Option<i32>,
    pub limit: Option<i32>,
}

/// Collateral event types for real-time updates
#[derive(Debug, Serialize, Clone)]
#[serde(tag = "type")]
#[allow(dead_code)]
pub enum CollateralEvent {
    Registered { token_id: String, owner_id: Uuid, asset_value: i64 },
    Locked { token_id: String },
    Unlocked { token_id: String },
    Burned { token_id: String },
}

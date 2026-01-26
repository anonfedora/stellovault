use serde::{Deserialize, Serialize};
use sqlx::types::chrono::{DateTime, Utc};
use uuid::Uuid;
use validator::Validate;

/// Collateral token model
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct Collateral {
    pub id: Uuid,
    pub token_id: String, // Soroban contract token ID
    pub owner_id: Uuid,
    pub asset_type: AssetType,
    pub asset_value: i64,
    pub metadata_hash: String,
    pub fractional_shares: i32,
    pub status: CollateralStatus,
    pub tx_hash: Option<String>,
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

/// Collateral status
#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, Copy, PartialEq, Eq)]
#[sqlx(type_name = "token_status", rename_all = "lowercase")]
pub enum CollateralStatus {
    Active,
    Locked,  // Locked in escrow
    Burned,
}

/// Request DTO for creating collateral
#[derive(Debug, Deserialize, Validate)]
pub struct CreateCollateralRequest {
    pub owner_id: Uuid,
    pub asset_type: AssetType,
    #[validate(range(min = 1))]
    pub asset_value: i64,
    #[validate(length(min = 1))]
    pub metadata_hash: String,
    #[validate(range(min = 1))]
    pub fractional_shares: i32,
}

/// Response DTO for collateral creation
#[derive(Debug, Serialize)]
pub struct CreateCollateralResponse {
    pub id: Uuid,
    pub token_id: String,
    pub status: CollateralStatus,
    pub tx_hash: Option<String>,
}

/// Query parameters for listing collateral
#[derive(Debug, Deserialize)]
pub struct ListCollateralQuery {
    pub owner_id: Option<Uuid>,
    pub asset_type: Option<AssetType>,
    pub status: Option<CollateralStatus>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

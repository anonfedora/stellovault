//! Data models for StelloVault backend

use serde::{Deserialize, Serialize};
use sqlx::types::chrono::{DateTime, Utc};
use uuid::Uuid;
use validator::Validate;

pub mod auth;
pub use auth::*;

/// User model
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct User {
    pub id: Uuid,
    pub primary_wallet_address: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub role: UserRole,
    pub risk_score: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            primary_wallet_address: user.primary_wallet_address,
            email: user.email,
            name: user.name,
            role: user.role.clone(),
            created_at: user.created_at,
        }
    }
}

/// User roles
#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum UserRole {
    Buyer,
    Seller,
    Oracle,
    Admin,
}

/// Trade escrow model
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct TradeEscrow {
    pub id: Uuid,
    pub escrow_id: String, // Soroban contract escrow ID
    pub buyer_id: Uuid,
    pub seller_id: Uuid,
    pub collateral_token_id: String,
    pub amount: i64,
    pub status: EscrowStatus,
    pub oracle_address: String,
    pub release_conditions: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Escrow status
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "escrow_status", rename_all = "lowercase")]
pub enum EscrowStatus {
    Pending,
    Active,
    Released,
    Cancelled,
}

/// Collateral model
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
    Locked,
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

/// Transaction model
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Transaction {
    pub id: Uuid,
    pub tx_hash: String,
    pub transaction_type: TransactionType,
    pub from_address: String,
    pub to_address: String,
    pub amount: i64,
    pub status: TransactionStatus,
    pub created_at: DateTime<Utc>,
}

/// Transaction types
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "transaction_type", rename_all = "snake_case")]
pub enum TransactionType {
    Tokenize,
    EscrowCreate,
    EscrowRelease,
    Transfer,
}

/// Transaction status
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "transaction_status", rename_all = "lowercase")]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Failed,
}

/// API response wrapper
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

/// Pagination parameters
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub page: Option<i32>,
    pub limit: Option<i32>,
}

/// Paginated response
#[allow(dead_code)]
#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: i64,
    pub page: i32,
    pub limit: i32,
}

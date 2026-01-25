//! Data models for StelloVault backend

use serde::{Deserialize, Serialize};
use sqlx::types::chrono::{DateTime, Utc};
use uuid::Uuid;

/// User model
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub stellar_address: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub role: UserRole,
    pub risk_score: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// User roles
#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum UserRole {
    Buyer,
    Seller,
    Oracle,
    Admin,
}

/// Trade escrow model
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
#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "escrow_status", rename_all = "lowercase")]
pub enum EscrowStatus {
    Pending,
    Active,
    Released,
    Cancelled,
}

/// Collateral token model
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
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
#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
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
    Locked, // Locked in escrow
    Burned,
}

/// Collateral registry model (mirror of Soroban contract)
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Collateral {
    pub id: Uuid,
    pub collateral_id: String, // Soroban contract collateral ID
    pub owner_id: Uuid,
    pub face_value: i64,
    pub expiry_ts: i64,
    pub metadata_hash: String,
    pub registered_at: DateTime<Utc>,
    pub locked: bool,
    pub status: CollateralStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Collateral status
#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, Copy, PartialEq, Eq)]
#[sqlx(type_name = "collateral_status", rename_all = "lowercase")]
pub enum CollateralStatus {
    Active,
    Locked,
    Expired,
    Burned,
}


/// Transaction model
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
#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "transaction_type", rename_all = "snake_case")]
pub enum TransactionType {
    Tokenize,
    EscrowCreate,
    EscrowRelease,
    Transfer,
}

/// Transaction status
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
#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub page: Option<i32>,
    pub limit: Option<i32>,
}

/// Paginated response
#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: i64,
    pub page: i32,
    pub limit: i32,
}

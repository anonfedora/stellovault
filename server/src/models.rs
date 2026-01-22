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
    Locked,  // Locked in escrow
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

/// Oracle provider model
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Oracle {
    pub id: Uuid,
    pub address: String,
    pub name: Option<String>,
    pub endpoint_url: Option<String>,
    pub public_key: Option<String>,
    pub is_active: bool,
    pub reputation_score: Option<f64>,
    pub total_confirmations: i32,
    pub successful_confirmations: i32,
    pub added_at: DateTime<Utc>,
    pub added_by: Option<Uuid>,
    pub updated_at: DateTime<Utc>,
}

/// Oracle confirmation model
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct OracleConfirmation {
    pub id: Uuid,
    pub escrow_id: String,
    pub oracle_address: String,
    pub event_type: i32,
    pub result: serde_json::Value,
    pub signature: String,
    pub transaction_hash: Option<String>,
    pub block_number: Option<i64>,
    pub gas_used: Option<i64>,
    pub confirmed_at: DateTime<Utc>,
    pub verification_status: VerificationStatus,
    pub error_message: Option<String>,
}

/// Oracle event types
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum OracleEventType {
    Shipment = 1,
    Delivery = 2,
    Quality = 3,
    Custom = 4,
}

/// Verification status for oracle confirmations
#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, Copy, PartialEq, Eq)]
#[sqlx(type_name = "verification_status", rename_all = "lowercase")]
pub enum VerificationStatus {
    Pending,
    Verified,
    Failed,
}

/// Oracle confirmation request payload
#[derive(Debug, Deserialize)]
pub struct OracleConfirmationRequest {
    pub escrow_id: String,
    pub event_type: i32,
    pub result: serde_json::Value,
    pub signature: String,
}

/// Oracle registration request payload
#[derive(Debug, Deserialize)]
pub struct OracleRegistrationRequest {
    pub address: String,
    pub name: Option<String>,
    pub endpoint_url: Option<String>,
    pub public_key: Option<String>,
}

/// Oracle metrics for dashboard
#[derive(Debug, Serialize)]
pub struct OracleMetrics {
    pub total_oracles: i64,
    pub active_oracles: i64,
    pub total_confirmations: i64,
    pub successful_confirmations: i64,
    pub average_reputation_score: f64,
}
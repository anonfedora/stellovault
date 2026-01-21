//! Escrow models and data structures for StelloVault backend

use serde::{Deserialize, Serialize};
use sqlx::types::chrono::{DateTime, Utc};
use uuid::Uuid;

/// Escrow model
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct Escrow {
    pub id: Uuid,
    pub escrow_id: i64,           // On-chain escrow ID from Soroban (i64 for PostgreSQL BIGINT)
    pub buyer_id: Uuid,
    pub seller_id: Uuid,
    pub collateral_id: Uuid,
    pub amount: i64,              // Amount in stroops
    pub status: EscrowStatus,
    pub oracle_address: String,
    pub release_conditions: String, // JSON string of conditions
    pub timeout_at: Option<DateTime<Utc>>,
    pub disputed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Enhanced escrow status with timeout and dispute states
#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, Copy, PartialEq, Eq)]
#[sqlx(type_name = "escrow_status", rename_all = "lowercase")]
pub enum EscrowStatus {
    Pending,    // Created but not funded
    Active,     // Funded and awaiting conditions
    Released,   // Funds released to seller
    Cancelled,  // Cancelled by parties
    TimedOut,   // Expired without completion
    Disputed,   // Under dispute resolution
}

/// Request DTO for creating an escrow
#[derive(Debug, Deserialize)]
pub struct CreateEscrowRequest {
    pub buyer_id: Uuid,
    pub seller_id: Uuid,
    pub collateral_id: Uuid,
    pub amount: i64,
    pub oracle_address: String,
    pub release_conditions: String,
    pub timeout_hours: Option<i64>, // Timeout in hours from creation
}

impl CreateEscrowRequest {
    /// Validate request
    pub fn validate(&self) -> Result<(), String> {
        if self.amount <= 0 {
            return Err("Amount must be greater than 0".to_string());
        }
        if self.buyer_id == self.seller_id {
            return Err("Buyer and seller must be different".to_string());
        }
        Ok(())
    }
}


/// Response DTO for escrow creation
#[derive(Debug, Serialize)]
pub struct CreateEscrowResponse {
    pub id: Uuid,
    pub escrow_id: i64,
    pub status: EscrowStatus,
    pub tx_hash: String,
}

/// Query parameters for listing escrows
#[derive(Debug, Deserialize)]
pub struct ListEscrowsQuery {
    pub status: Option<EscrowStatus>,
    pub buyer_id: Option<Uuid>,
    pub seller_id: Option<Uuid>,
    pub page: Option<i32>,
    pub limit: Option<i32>,
}

/// Escrow with related collateral information
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct EscrowWithCollateral {
    // Escrow fields
    pub id: Uuid,
    pub escrow_id: i64,
    pub buyer_id: Uuid,
    pub seller_id: Uuid,
    pub collateral_id: Uuid,
    pub amount: i64,
    pub status: EscrowStatus,
    pub oracle_address: String,
    pub release_conditions: String,
    pub timeout_at: Option<DateTime<Utc>>,
    pub disputed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    
    // Collateral fields
    pub token_id: String,
    pub asset_type: String,
    pub asset_value: i64,
}

/// Escrow event types for real-time updates
#[derive(Debug, Serialize, Clone)]
#[serde(tag = "type")]
pub enum EscrowEvent {
    Created { escrow_id: i64, buyer_id: Uuid, seller_id: Uuid },
    Activated { escrow_id: i64 },
    Released { escrow_id: i64 },
    Cancelled { escrow_id: i64 },
    TimedOut { escrow_id: i64 },
    Disputed { escrow_id: i64, reason: String },
    StatusUpdated { escrow_id: i64, status: EscrowStatus },
}

/// Webhook payload structure for escrow updates
#[derive(Debug, Deserialize)]
pub struct WebhookPayload {
    pub _event_type: String,
    pub escrow_id: i64,
    pub status: Option<EscrowStatus>,
    pub _tx_hash: Option<String>,
    pub _timestamp: DateTime<Utc>,
}

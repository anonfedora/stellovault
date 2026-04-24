//! Escrow models and data structures for StelloVault backend

use serde::{Deserialize, Serialize};
use sqlx::types::chrono::{DateTime, Utc};
use uuid::Uuid;

/// Escrow model
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct Escrow {
    pub id: Uuid,
    pub escrow_id: i64, // On-chain escrow ID from Soroban (i64 for PostgreSQL BIGINT)
    pub buyer_id: Uuid,
    pub seller_id: Uuid,
    pub lender_id: Uuid,
    pub collateral_id: String,    // Collateral registry ID from Soroban contract
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
    Pending,   // Created but not funded
    Active,    // Funded and awaiting conditions
    Released,  // Funds released to seller
    Refunded,  // Funds returned to lender
    Cancelled, // Cancelled by parties
    TimedOut,  // Expired without completion
    Disputed,  // Under dispute resolution
}

/// Request DTO for creating an escrow
#[derive(Debug, Deserialize)]
pub struct CreateEscrowRequest {
    pub buyer_id: Uuid,
    pub seller_id: Uuid,
    pub lender_id: Uuid,
    pub collateral_id: String,    // Collateral registry ID from Soroban contract
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
    pub lender_id: Uuid,
    pub collateral_id: String,
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
    Created {
        escrow_id: i64,
        buyer_id: Uuid,
        seller_id: Uuid,
    },
    Activated {
        escrow_id: i64,
    },
    Released {
        escrow_id: i64,
    },
    Refunded {
        escrow_id: i64,
    },
    Cancelled {
        escrow_id: i64,
    },
    TimedOut {
        escrow_id: i64,
    },
    Disputed {
        escrow_id: i64,
        reason: String,
    },
    StatusUpdated {
        escrow_id: i64,
        status: EscrowStatus,
    },
}

/// Webhook payload structure for escrow updates
#[derive(Debug, Deserialize)]
pub struct WebhookPayload {
    #[serde(rename = "event_type")]
    pub _event_type: String,
    pub escrow_id: i64,
    pub status: Option<EscrowStatus>,
    #[serde(rename = "tx_hash")]
    pub _tx_hash: Option<String>,
    #[serde(rename = "timestamp")]
    pub _timestamp: DateTime<Utc>,
}

/// Request DTO for updating escrow status
#[derive(Debug, Deserialize)]
pub struct UpdateEscrowStatusRequest {
    pub status: EscrowStatus,
    pub reason: Option<String>,
}

/// Request DTO for releasing escrow funds
#[derive(Debug, Deserialize)]
pub struct ReleaseEscrowRequest {
    pub release_code: String,
    pub actor_id: Option<Uuid>,
}

/// Request DTO for disputing an escrow
#[derive(Debug, Deserialize)]
pub struct DisputeEscrowRequest {
    pub reason: String,
    pub actor_id: Option<Uuid>,
    pub evidence: Option<serde_json::Value>,
}

/// Request DTO for funding an escrow
#[derive(Debug, Deserialize)]
pub struct FundEscrowRequest {
    pub amount: i64,
    pub tx_hash: Option<String>,
}

/// A single entry in the escrow history/audit trail
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct EscrowHistoryEntry {
    pub id: Uuid,
    pub escrow_id: Uuid,
    pub action: String,
    pub old_status: Option<EscrowStatus>,
    pub new_status: Option<EscrowStatus>,
    pub actor_id: Option<Uuid>,
    pub metadata: Option<serde_json::Value>,
    pub tx_hash: Option<String>,
    pub created_at: DateTime<Utc>,
}

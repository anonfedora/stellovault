//! Oracle models and data structures for StelloVault backend
//!
//! I'm defining the core oracle event shape here, mapping exactly to what we'll store in Postgres.

use serde::{Deserialize, Serialize};
use sqlx::types::chrono::{DateTime, Utc};
use uuid::Uuid;

/// Oracle data type - the source of the off-chain confirmation
#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, Copy, PartialEq, Eq)]
#[sqlx(type_name = "oracle_data_type", rename_all = "lowercase")]
pub enum OracleDataType {
    Shipping,
    Iot,
    Manual,
}

/// Oracle event status - tracks the lifecycle of a confirmation
#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, Copy, PartialEq, Eq)]
#[sqlx(type_name = "oracle_event_status", rename_all = "lowercase")]
pub enum OracleEventStatus {
    Pending,    // Received but not yet aggregated
    Confirmed,  // Single oracle confirmed
    Aggregated, // Threshold met, Soroban tx submitted
    Disputed,   // Flagged for dispute resolution
    Rejected,   // Rejected due to validation failure or duplicate
}

/// Oracle event model - represents a single confirmation from an oracle
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct OracleEvent {
    pub id: Uuid,
    pub escrow_id: i64,
    pub oracle_address: String,
    pub data_type: OracleDataType,
    pub payload_hash: String,
    pub payload: serde_json::Value, // JSONB in Postgres
    pub signature: String,
    pub status: OracleEventStatus,
    pub tx_hash: Option<String>, // Soroban tx hash when submitted
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Oracle audit log model - tracks all oracle actions for compliance
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct OracleAuditLog {
    pub id: Uuid,
    pub oracle_event_id: Option<Uuid>,
    pub action: String,
    pub actor_address: String,
    pub details: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// Request/Response DTOs
// ============================================================================

/// Request DTO for POST /oracle/confirm
#[derive(Debug, Deserialize)]
pub struct OracleConfirmRequest {
    pub escrow_id: i64,
    pub oracle_address: String,
    pub data_type: OracleDataType,
    pub payload: OraclePayload,
    pub signature: String, // Base64-encoded Ed25519 signature
}

impl OracleConfirmRequest {
    /// I'm validating the request before processing to catch obvious errors early.
    pub fn validate(&self) -> Result<(), String> {
        if self.escrow_id <= 0 {
            return Err("escrow_id must be positive".to_string());
        }
        if self.oracle_address.is_empty() {
            return Err("oracle_address is required".to_string());
        }
        if self.signature.is_empty() {
            return Err("signature is required".to_string());
        }
        self.payload.validate()
    }
}

/// Oracle payload - the actual data being confirmed
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OraclePayload {
    /// Unique identifier for this confirmation (prevents replay)
    pub confirmation_id: String,
    /// Timestamp when the event was observed
    pub observed_at: DateTime<Utc>,
    /// Human-readable description
    pub description: Option<String>,
    /// Type-specific data
    #[serde(flatten)]
    pub data: OraclePayloadData,
}

impl OraclePayload {
    pub fn validate(&self) -> Result<(), String> {
        if self.confirmation_id.is_empty() {
            return Err("confirmation_id is required".to_string());
        }
        Ok(())
    }
}

/// Type-specific oracle payload data
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "payload_type")]
pub enum OraclePayloadData {
    #[serde(rename = "shipping")]
    Shipping {
        tracking_number: String,
        carrier: String,
        status: String,
        location: Option<String>,
    },
    #[serde(rename = "iot")]
    Iot {
        device_id: String,
        sensor_type: String,
        value: f64,
        unit: String,
    },
    #[serde(rename = "manual")]
    Manual {
        verifier_name: String,
        verification_method: String,
        notes: Option<String>,
    },
}

/// Response DTO for POST /oracle/confirm
#[derive(Debug, Serialize)]
pub struct OracleConfirmResponse {
    pub event_id: Uuid,
    pub status: OracleEventStatus,
    pub aggregation_count: i32,
    pub threshold_met: bool,
    pub tx_hash: Option<String>,
}

/// Request DTO for POST /oracle/dispute
#[derive(Debug, Deserialize)]
pub struct OracleDisputeRequest {
    pub escrow_id: i64,
    pub reason: String,
    pub disputer_address: String,
    pub signature: String,
}

/// Query parameters for GET /oracle/events
#[derive(Debug, Deserialize)]
pub struct ListOracleEventsQuery {
    pub escrow_id: Option<i64>,
    pub oracle_address: Option<String>,
    pub status: Option<OracleEventStatus>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

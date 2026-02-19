//! Data models for StelloVault backend

use serde::{Deserialize, Serialize};
use sqlx::types::chrono::{DateTime, Utc};
use uuid::Uuid;

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
    pub tx_hash: Option<String>,
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

/// Governance proposal model
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct GovernanceProposal {
    pub id: Uuid,
    pub proposal_id: String, // Soroban contract proposal ID
    pub title: String,
    pub description: String,
    pub proposer: String, // Stellar address
    pub proposal_type: ProposalType,
    pub status: ProposalStatus,
    pub voting_start: DateTime<Utc>,
    pub voting_end: DateTime<Utc>,
    pub execution_time: Option<DateTime<Utc>>,
    pub for_votes: i64,
    pub against_votes: i64,
    pub abstain_votes: i64,
    pub quorum_required: i64,
    pub approval_threshold: f64, // Percentage 0.0-1.0
    pub executed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Proposal types
#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone)]
#[sqlx(type_name = "proposal_type", rename_all = "snake_case")]
pub enum ProposalType {
    ParameterChange,
    ContractUpgrade,
    TreasuryAction,
    EmergencyAction,
    Custom,
}

/// Proposal status
#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, Copy, PartialEq)]
#[sqlx(type_name = "proposal_status", rename_all = "lowercase")]
pub enum ProposalStatus {
    Pending,
    Active,
    Succeeded,
    Failed,
    Executed,
    Cancelled,
}

/// Governance vote model
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct GovernanceVote {
    pub id: Uuid,
    pub proposal_id: String,
    pub voter: String, // Stellar address
    pub vote_option: VoteOption,
    pub voting_power: i64,
    pub transaction_hash: Option<String>,
    pub voted_at: DateTime<Utc>,
}

/// Vote options
#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone)]
#[sqlx(type_name = "vote_option", rename_all = "lowercase")]
pub enum VoteOption {
    For,
    Against,
    Abstain,
}

/// Governance parameter model
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct GovernanceParameter {
    pub id: Uuid,
    pub parameter_key: String,
    pub parameter_value: serde_json::Value,
    pub parameter_type: ParameterType,
    pub description: String,
    pub proposed_by: Option<String>,
    pub proposal_id: Option<String>,
    pub effective_from: DateTime<Utc>,
    pub effective_until: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Parameter types
#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone)]
#[sqlx(type_name = "parameter_type", rename_all = "snake_case")]
pub enum ParameterType {
    Integer,
    Float,
    Boolean,
    String,
    Json,
}

/// Governance audit log model
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct GovernanceAuditLog {
    pub id: Uuid,
    pub action_type: AuditActionType,
    pub entity_type: AuditEntityType,
    pub entity_id: String,
    pub user_address: String,
    pub old_value: Option<serde_json::Value>,
    pub new_value: Option<serde_json::Value>,
    pub transaction_hash: Option<String>,
    pub block_number: Option<i64>,
    pub created_at: DateTime<Utc>,
}

/// Audit action types
#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone)]
#[sqlx(type_name = "audit_action_type", rename_all = "snake_case")]
pub enum AuditActionType {
    ProposalCreated,
    VoteCast,
    ProposalExecuted,
    ParameterChanged,
    EmergencyAction,
}

/// Audit entity types
#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone)]
#[sqlx(type_name = "audit_entity_type", rename_all = "snake_case")]
pub enum AuditEntityType {
    Proposal,
    Vote,
    Parameter,
    Contract,
}

/// Governance metrics for dashboard
#[derive(Debug, Serialize)]
pub struct GovernanceMetrics {
    pub total_proposals: i64,
    pub active_proposals: i64,
    pub total_votes: i64,
    pub participation_rate: f64,
    pub average_voting_time: f64, // in hours
    pub successful_proposals: i64,
    pub failed_proposals: i64,
}

/// Governance configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct GovernanceConfig {
    pub voting_period_hours: i32,
    pub execution_delay_hours: i32,
    pub quorum_percentage: f64,
    pub approval_threshold_percentage: f64,
    pub min_voting_power: i64,
    pub emergency_quorum_percentage: f64,
    pub emergency_approval_threshold_percentage: f64,
}

/// Vote submission request
#[derive(Debug, Deserialize)]
pub struct VoteSubmissionRequest {
    pub proposal_id: String,
    pub vote_option: VoteOption,
    pub voter_address: String,
}

/// Proposal creation request
#[derive(Debug, Deserialize)]
pub struct ProposalCreationRequest {
    pub title: String,
    pub description: String,
    pub proposal_type: ProposalType,
    pub parameters: Option<serde_json::Value>, // For parameter change proposals
    pub execution_time: Option<DateTime<Utc>>,
}

/// Governance parameter cache
#[derive(Debug, Clone)]
pub struct GovernanceParameterCache {
    pub voting_period_hours: i32,
    pub execution_delay_hours: i32,
    pub quorum_percentage: f64,
    pub approval_threshold_percentage: f64,
    pub min_voting_power: i64,
    pub emergency_quorum_percentage: f64,
    pub emergency_approval_threshold_percentage: f64,
    pub last_updated: DateTime<Utc>,
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

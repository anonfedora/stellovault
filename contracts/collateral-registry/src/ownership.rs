//! Ownership tracking and transfer management

use soroban_sdk::{contracttype, Address};

/// Ownership transfer record
#[contracttype]
#[derive(Clone)]
pub struct OwnershipTransfer {
    pub collateral_id: u64,
    pub from: Address,
    pub to: Address,
    pub timestamp: u64,
}

/// Ownership history
#[contracttype]
#[derive(Clone)]
pub struct OwnershipHistory {
    pub collateral_id: u64,
    pub transfers: soroban_sdk::Vec<OwnershipTransfer>,
}

/// Ownership proof
#[contracttype]
#[derive(Clone)]
pub struct OwnershipProof {
    pub collateral_id: u64,
    pub owner: Address,
    pub proof_type: ProofType,
    pub proof_data: soroban_sdk::BytesN<32>,
    pub verified: bool,
    pub created_at: u64,
}

/// Proof type
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProofType {
    DocumentHash = 0,
    SignatureVerification = 1,
    ThirdPartyAttestation = 2,
    ChainOfTitle = 3,
}

/// Ownership rights
#[contracttype]
#[derive(Clone)]
pub struct OwnershipRights {
    pub collateral_id: u64,
    pub owner: Address,
    pub can_transfer: bool,
    pub can_lock: bool,
    pub can_modify: bool,
    pub can_liquidate: bool,
    pub created_at: u64,
}

/// Ownership dispute
#[contracttype]
#[derive(Clone)]
pub struct OwnershipDispute {
    pub id: u64,
    pub collateral_id: u64,
    pub claimant: Address,
    pub dispute_reason: soroban_sdk::String,
    pub status: DisputeStatus,
    pub created_at: u64,
    pub resolved_at: u64,
}

/// Dispute status
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DisputeStatus {
    Open = 0,
    UnderReview = 1,
    Resolved = 2,
    Rejected = 3,
}

/// Ownership transfer request
#[contracttype]
#[derive(Clone)]
pub struct TransferRequest {
    pub id: u64,
    pub collateral_id: u64,
    pub from: Address,
    pub to: Address,
    pub status: TransferStatus,
    pub created_at: u64,
    pub expires_at: u64,
}

/// Transfer status
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransferStatus {
    Pending = 0,
    Approved = 1,
    Rejected = 2,
    Completed = 3,
    Expired = 4,
}

/// Ownership delegation
#[contracttype]
#[derive(Clone)]
pub struct OwnershipDelegation {
    pub id: u64,
    pub collateral_id: u64,
    pub owner: Address,
    pub delegate: Address,
    pub delegated_rights: soroban_sdk::Vec<DelegatedRight>,
    pub expires_at: u64,
    pub created_at: u64,
}

/// Delegated right
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DelegatedRight {
    View = 0,
    Modify = 1,
    Transfer = 2,
    Lock = 3,
}

/// Ownership verification
#[contracttype]
#[derive(Clone)]
pub struct OwnershipVerification {
    pub collateral_id: u64,
    pub owner: Address,
    pub verified_by: Address,
    pub verification_method: VerificationMethod,
    pub verified_at: u64,
    pub valid_until: u64,
}

/// Verification method
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VerificationMethod {
    DocumentReview = 0,
    ThirdPartyAttestation = 1,
    BlockchainVerification = 2,
    LegalReview = 3,
}

//! Collateral data structures and types

use soroban_sdk::{contracttype, Address, BytesN, String};

/// Collateral status enumeration
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CollateralStatus {
    Active = 0,
    Inactive = 1,
    Seized = 2,
    Liquidated = 3,
    Disputed = 4,
}

/// Asset type enumeration
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AssetType {
    RealEstate = 0,
    Equipment = 1,
    Inventory = 2,
    Receivables = 3,
    Securities = 4,
    Commodities = 5,
    Vehicles = 6,
    Intellectual = 7,
    Other = 8,
}

/// Verification status enumeration
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VerificationStatus {
    Pending = 0,
    Verified = 1,
    Failed = 2,
    Expired = 3,
}

/// Main collateral structure
#[contracttype]
#[derive(Clone)]
pub struct Collateral {
    pub id: u64,
    pub owner: Address,
    pub asset_hash: BytesN<32>,
    pub metadata_uri: String,
    pub asset_type: AssetType,
    pub current_valuation: i128,
    pub previous_valuation: i128,
    pub valuation_timestamp: u64,
    pub status: CollateralStatus,
    pub locked: bool,
    pub locked_by_escrow: u64,
    pub verification_status: VerificationStatus,
    pub verified_by: Option<Address>,
    pub verified_at: u64,
    pub created_at: u64,
    pub updated_at: u64,
    pub expiry_date: u64,
    pub fractionalized: bool,
    pub fraction_count: u32,
}

/// Collateral metadata
#[contracttype]
#[derive(Clone)]
pub struct CollateralMetadata {
    pub collateral_id: u64,
    pub description: String,
    pub location: String,
    pub condition: AssetCondition,
    pub insurance_status: InsuranceStatus,
    pub maintenance_records: u32,
    pub last_inspection: u64,
    pub next_inspection: u64,
}

/// Asset condition enumeration
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AssetCondition {
    Excellent = 0,
    Good = 1,
    Fair = 2,
    Poor = 3,
    Critical = 4,
}

/// Insurance status enumeration
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InsuranceStatus {
    Uninsured = 0,
    Insured = 1,
    PartiallyInsured = 2,
    Expired = 3,
}

/// Collateral fraction
#[contracttype]
#[derive(Clone)]
pub struct CollateralFraction {
    pub id: u64,
    pub collateral_id: u64,
    pub owner: Address,
    pub fraction_percentage: u32,
    pub value: i128,
    pub created_at: u64,
}

/// Collateral batch operation
#[contracttype]
#[derive(Clone)]
pub struct BatchOperation {
    pub id: u64,
    pub operation_type: BatchOperationType,
    pub collateral_ids: soroban_sdk::Vec<u64>,
    pub status: BatchStatus,
    pub created_at: u64,
    pub completed_at: u64,
}

/// Batch operation type
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BatchOperationType {
    Registration = 0,
    Valuation = 1,
    Transfer = 2,
    Locking = 3,
    Verification = 4,
}

/// Batch status
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BatchStatus {
    Pending = 0,
    InProgress = 1,
    Completed = 2,
    Failed = 3,
}

/// Collateral document
#[contracttype]
#[derive(Clone)]
pub struct CollateralDocument {
    pub id: u64,
    pub collateral_id: u64,
    pub document_type: DocumentType,
    pub document_hash: BytesN<32>,
    pub ipfs_uri: String,
    pub uploaded_by: Address,
    pub uploaded_at: u64,
    pub verified: bool,
}

/// Document type
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentType {
    Title = 0,
    Appraisal = 1,
    Insurance = 2,
    Inspection = 3,
    Certificate = 4,
    Other = 5,
}

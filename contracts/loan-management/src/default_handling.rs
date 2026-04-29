//! Default handling and recovery module

use soroban_sdk::{contracttype, String};

/// Default event
#[contracttype]
#[derive(Clone)]
pub struct DefaultEvent {
    pub loan_id: u64,
    pub default_date: u64,
    pub outstanding_balance: i128,
    pub accrued_interest: i128,
    pub default_reason: String,
}

/// Default recovery plan
#[contracttype]
#[derive(Clone)]
pub struct RecoveryPlan {
    pub id: u64,
    pub loan_id: u64,
    pub created_date: u64,
    pub recovery_strategy: RecoveryStrategy,
    pub status: RecoveryStatus,
}

/// Recovery strategy
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoveryStrategy {
    Restructuring = 0,
    Forbearance = 1,
    CollateralSeizure = 2,
    LegalAction = 3,
    Settlement = 4,
}

/// Recovery status
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoveryStatus {
    Pending = 0,
    InProgress = 1,
    Successful = 2,
    Failed = 3,
    Abandoned = 4,
}

/// Forbearance agreement
#[contracttype]
#[derive(Clone)]
pub struct ForbearanceAgreement {
    pub id: u64,
    pub loan_id: u64,
    pub start_date: u64,
    pub end_date: u64,
    pub payment_reduction: u32, // basis points
    pub status: ForbearanceStatus,
}

/// Forbearance status
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ForbearanceStatus {
    Active = 0,
    Completed = 1,
    Terminated = 2,
    Failed = 3,
}

/// Collateral seizure
#[contracttype]
#[derive(Clone)]
pub struct CollateralSeizure {
    pub id: u64,
    pub loan_id: u64,
    pub collateral_id: u64,
    pub seizure_date: u64,
    pub sale_date: u64,
    pub sale_amount: i128,
    pub status: SeizureStatus,
}

/// Seizure status
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SeizureStatus {
    Initiated = 0,
    InProgress = 1,
    Sold = 2,
    Completed = 3,
    Disputed = 4,
}

/// Default fee
#[contracttype]
#[derive(Clone)]
pub struct DefaultFee {
    pub loan_id: u64,
    pub fee_amount: i128,
    pub fee_rate: u32,
    pub applied_date: u64,
}

/// Loss mitigation
#[contracttype]
#[derive(Clone)]
pub struct LossMitigation {
    pub loan_id: u64,
    pub mitigation_type: MitigationType,
    pub amount: i128,
    pub status: MitigationStatus,
}

/// Mitigation type
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MitigationType {
    Refinancing = 0,
    ShortSale = 1,
    DeedInLieu = 2,
    Modification = 3,
    Forbearance = 4,
}

/// Mitigation status
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MitigationStatus {
    Proposed = 0,
    Approved = 1,
    InProgress = 2,
    Completed = 3,
    Failed = 4,
}

//! Collateral locking mechanism for loan security

use soroban_sdk::{contracttype, Address};

/// Collateral lock record
#[contracttype]
#[derive(Clone)]
pub struct CollateralLock {
    pub collateral_id: u64,
    pub escrow_id: u64,
    pub locked_at: u64,
    pub locked_by: Address,
}

/// Lock status enumeration
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LockStatus {
    Active = 0,
    Released = 1,
    Disputed = 2,
    Liquidated = 3,
}

/// Lock record with extended details
#[contracttype]
#[derive(Clone)]
pub struct LockRecord {
    pub id: u64,
    pub collateral_id: u64,
    pub escrow_id: u64,
    pub locked_by: Address,
    pub locked_at: u64,
    pub unlock_requested_at: u64,
    pub status: LockStatus,
    pub reason: soroban_sdk::String,
}

/// Lock history
#[contracttype]
#[derive(Clone)]
pub struct LockHistory {
    pub collateral_id: u64,
    pub locks: soroban_sdk::Vec<LockRecord>,
}

/// Lock release request
#[contracttype]
#[derive(Clone)]
pub struct LockReleaseRequest {
    pub id: u64,
    pub collateral_id: u64,
    pub escrow_id: u64,
    pub requested_by: Address,
    pub status: ReleaseStatus,
    pub created_at: u64,
    pub approved_at: u64,
}

/// Release status
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReleaseStatus {
    Pending = 0,
    Approved = 1,
    Rejected = 2,
    Completed = 3,
}

/// Lock conditions
#[contracttype]
#[derive(Clone)]
pub struct LockConditions {
    pub collateral_id: u64,
    pub escrow_id: u64,
    pub minimum_lock_period: u64,
    pub maximum_lock_period: u64,
    pub early_release_penalty: u32,
    pub created_at: u64,
}

/// Lock dispute
#[contracttype]
#[derive(Clone)]
pub struct LockDispute {
    pub id: u64,
    pub collateral_id: u64,
    pub escrow_id: u64,
    pub disputed_by: Address,
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

/// Lock enforcement
#[contracttype]
#[derive(Clone)]
pub struct LockEnforcement {
    pub id: u64,
    pub collateral_id: u64,
    pub escrow_id: u64,
    pub enforcement_type: EnforcementType,
    pub enforced_by: Address,
    pub enforced_at: u64,
}

/// Enforcement type
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EnforcementType {
    Liquidation = 0,
    Seizure = 1,
    Auction = 2,
    Restructuring = 3,
}

/// Lock monitoring
#[contracttype]
#[derive(Clone)]
pub struct LockMonitoring {
    pub collateral_id: u64,
    pub escrow_id: u64,
    pub monitoring_status: MonitoringStatus,
    pub last_checked: u64,
    pub next_check: u64,
}

/// Monitoring status
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MonitoringStatus {
    Active = 0,
    Paused = 1,
    Completed = 2,
}

/// Lock release schedule
#[contracttype]
#[derive(Clone)]
pub struct LockReleaseSchedule {
    pub id: u64,
    pub collateral_id: u64,
    pub escrow_id: u64,
    pub scheduled_release_date: u64,
    pub release_percentage: u32,
    pub created_at: u64,
}

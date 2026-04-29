//! Valuation management and oracle verification

use soroban_sdk::{contracttype, Address, BytesN};

/// Valuation record
#[contracttype]
#[derive(Clone)]
pub struct ValuationRecord {
    pub collateral_id: u64,
    pub old_valuation: i128,
    pub new_valuation: i128,
    pub oracle: Address,
    pub timestamp: u64,
}

/// Valuation method enumeration
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValuationMethod {
    MarketComparable = 0,
    CostApproach = 1,
    IncomeApproach = 2,
    Automated = 3,
    Expert = 4,
}

/// Valuation report
#[contracttype]
#[derive(Clone)]
pub struct ValuationReport {
    pub id: u64,
    pub collateral_id: u64,
    pub valuation_method: ValuationMethod,
    pub appraiser: Address,
    pub valuation_amount: i128,
    pub confidence_level: u32,
    pub report_hash: BytesN<32>,
    pub created_at: u64,
    pub valid_until: u64,
}

/// Valuation adjustment
#[contracttype]
#[derive(Clone)]
pub struct ValuationAdjustment {
    pub id: u64,
    pub collateral_id: u64,
    pub adjustment_type: AdjustmentType,
    pub adjustment_amount: i128,
    pub reason: soroban_sdk::String,
    pub applied_by: Address,
    pub applied_at: u64,
}

/// Adjustment type
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdjustmentType {
    Depreciation = 0,
    Appreciation = 1,
    Damage = 2,
    Maintenance = 3,
    MarketShift = 4,
}

/// Valuation history entry
#[contracttype]
#[derive(Clone)]
pub struct ValuationHistory {
    pub collateral_id: u64,
    pub valuations: soroban_sdk::Vec<ValuationSnapshot>,
}

/// Valuation snapshot
#[contracttype]
#[derive(Clone)]
pub struct ValuationSnapshot {
    pub amount: i128,
    pub timestamp: u64,
    pub method: ValuationMethod,
    pub appraiser: Address,
}

/// Valuation trend
#[contracttype]
#[derive(Clone)]
pub struct ValuationTrend {
    pub collateral_id: u64,
    pub trend_direction: TrendDirection,
    pub trend_percentage: i32,
    pub period_days: u64,
    pub calculated_at: u64,
}

/// Trend direction
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrendDirection {
    Increasing = 0,
    Decreasing = 1,
    Stable = 2,
}

/// Valuation dispute
#[contracttype]
#[derive(Clone)]
pub struct ValuationDispute {
    pub id: u64,
    pub collateral_id: u64,
    pub disputed_by: Address,
    pub dispute_reason: soroban_sdk::String,
    pub disputed_valuation: i128,
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

/// Valuation insurance
#[contracttype]
#[derive(Clone)]
pub struct ValuationInsurance {
    pub id: u64,
    pub collateral_id: u64,
    pub insurer: Address,
    pub coverage_amount: i128,
    pub premium_rate: u32,
    pub status: InsuranceStatus,
    pub created_at: u64,
    pub expires_at: u64,
}

/// Insurance status
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InsuranceStatus {
    Active = 0,
    Claimed = 1,
    Expired = 2,
    Cancelled = 3,
}

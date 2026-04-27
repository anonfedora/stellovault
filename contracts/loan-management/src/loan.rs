//! Loan data structures and types

use soroban_sdk::{contracttype, Address, String};

/// Loan status enumeration
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoanStatus {
    Active = 0,
    Paid = 1,
    Defaulted = 2,
    Restructured = 3,
    Closed = 4,
    Suspended = 5,
}

/// Interest calculation type
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InterestType {
    Simple = 0,
    Compound = 1,
    Fixed = 2,
}

/// Main loan structure
#[contracttype]
#[derive(Clone)]
pub struct Loan {
    pub id: u64,
    pub borrower: Address,
    pub principal: i128,
    pub outstanding_balance: i128,
    pub interest_rate: u32,           // basis points
    pub interest_type: InterestType,
    pub collateral_id: u64,
    pub collateral_value: i128,
    pub ltv: u32,                     // Loan-to-Value ratio (basis points)
    pub term: u64,                    // in seconds
    pub start_date: u64,
    pub maturity_date: u64,
    pub status: LoanStatus,
    pub total_interest_paid: i128,
    pub total_repaid: i128,
    pub last_payment_date: u64,
    pub next_payment_date: u64,
    pub grace_period: u64,            // in seconds
    pub default_date: u64,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Loan creation parameters
#[contracttype]
#[derive(Clone)]
pub struct LoanParams {
    pub borrower: Address,
    pub collateral_id: u64,
    pub principal: i128,
    pub interest_rate: u32,
    pub term: u64,
    pub interest_type: InterestType,
}

/// Loan restructuring parameters
#[contracttype]
#[derive(Clone)]
pub struct LoanRestructuring {
    pub new_interest_rate: Option<u32>,
    pub new_term: Option<u64>,
    pub new_grace_period: Option<u64>,
    pub reason: String,
}

/// Loan performance metrics
#[contracttype]
#[derive(Clone)]
pub struct LoanPerformance {
    pub loan_id: u64,
    pub on_time_payments: u32,
    pub late_payments: u32,
    pub missed_payments: u32,
    pub payment_history_score: u32,  // 0-10000
    pub days_past_due: u64,
    pub last_updated: u64,
}

/// Loan portfolio
#[contracttype]
#[derive(Clone)]
pub struct LoanPortfolio {
    pub id: u64,
    pub owner: Address,
    pub loan_ids: soroban_sdk::Vec<u64>,
    pub total_principal: i128,
    pub total_outstanding: i128,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Loan syndication
#[contracttype]
#[derive(Clone)]
pub struct LoanSyndication {
    pub id: u64,
    pub loan_id: u64,
    pub lead_lender: Address,
    pub participants: soroban_sdk::Vec<Address>,
    pub total_amount: i128,
    pub status: SyndicationStatus,
    pub created_at: u64,
}

/// Syndication status
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SyndicationStatus {
    Pending = 0,
    Active = 1,
    Completed = 2,
    Defaulted = 3,
}

/// Loan securitization
#[contracttype]
#[derive(Clone)]
pub struct LoanSecuritization {
    pub id: u64,
    pub portfolio_id: u64,
    pub issuer: Address,
    pub total_value: i128,
    pub tranche_count: u32,
    pub status: SecuritizationStatus,
    pub created_at: u64,
}

/// Securitization status
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SecuritizationStatus {
    Pending = 0,
    Active = 1,
    Completed = 2,
    Defaulted = 3,
}

/// Cross-currency loan
#[contracttype]
#[derive(Clone)]
pub struct CrossCurrencyLoan {
    pub id: u64,
    pub loan_id: u64,
    pub base_currency: String,
    pub loan_currency: String,
    pub exchange_rate: i128,
    pub hedge_status: HedgeStatus,
    pub created_at: u64,
}

/// Hedge status
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HedgeStatus {
    Unhedged = 0,
    Hedged = 1,
    PartiallyHedged = 2,
}

/// Loan covenant
#[contracttype]
#[derive(Clone)]
pub struct LoanCovenant {
    pub id: u64,
    pub loan_id: u64,
    pub covenant_type: CovenantType,
    pub threshold: i128,
    pub status: CovenantStatus,
    pub created_at: u64,
}

/// Covenant type
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CovenantType {
    DebtToEquity = 0,
    InterestCoverage = 1,
    MinimumCash = 2,
    MaximumLeverage = 3,
    AssetCoverage = 4,
}

/// Covenant status
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CovenantStatus {
    Compliant = 0,
    Breached = 1,
    Waived = 2,
}

/// Loan fee structure
#[contracttype]
#[derive(Clone)]
pub struct LoanFees {
    pub origination_fee: u32,         // basis points
    pub servicing_fee: u32,           // basis points
    pub prepayment_penalty: u32,      // basis points
    pub late_payment_fee: u32,        // basis points
    pub default_fee: u32,             // basis points
}

/// Loan pricing
#[contracttype]
#[derive(Clone)]
pub struct LoanPricing {
    pub base_rate: u32,
    pub risk_premium: u32,
    pub collateral_premium: u32,
    pub term_premium: u32,
    pub total_rate: u32,
}

/// Loan insurance
#[contracttype]
#[derive(Clone)]
pub struct LoanInsurance {
    pub id: u64,
    pub loan_id: u64,
    pub insurer: Address,
    pub coverage_amount: i128,
    pub premium_rate: u32,
    pub status: InsuranceStatus,
    pub created_at: u64,
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

/// Loan modification
#[contracttype]
#[derive(Clone)]
pub struct LoanModification {
    pub id: u64,
    pub loan_id: u64,
    pub modification_type: ModificationType,
    pub old_value: i128,
    pub new_value: i128,
    pub approved_by: Address,
    pub created_at: u64,
}

/// Modification type
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModificationType {
    RateChange = 0,
    TermExtension = 1,
    PrincipalReduction = 2,
    PaymentReschedule = 3,
    CovenantWaiver = 4,
}

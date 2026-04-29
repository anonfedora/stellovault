//! Loan restructuring module

use soroban_sdk::{contracttype, Address, String};

/// Restructuring proposal
#[contracttype]
#[derive(Clone)]
pub struct RestructuringProposal {
    pub id: u64,
    pub loan_id: u64,
    pub proposed_by: Address,
    pub proposal_date: u64,
    pub new_terms: RestructuringTerms,
    pub status: ProposalStatus,
}

/// Restructuring terms
#[contracttype]
#[derive(Clone)]
pub struct RestructuringTerms {
    pub new_principal: Option<i128>,
    pub new_interest_rate: Option<u32>,
    pub new_term: Option<u64>,
    pub new_payment_schedule: Option<String>,
    pub forgiveness_amount: Option<i128>,
}

/// Proposal status
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProposalStatus {
    Proposed = 0,
    UnderReview = 1,
    Approved = 2,
    Rejected = 3,
    Implemented = 4,
}

/// Restructuring history
#[contracttype]
#[derive(Clone)]
pub struct RestructuringHistory {
    pub loan_id: u64,
    pub restructuring_count: u32,
    pub total_forgiveness: i128,
    pub last_restructure_date: u64,
}

/// Debt modification
#[contracttype]
#[derive(Clone)]
pub struct DebtModification {
    pub id: u64,
    pub loan_id: u64,
    pub modification_type: ModificationType,
    pub old_value: i128,
    pub new_value: i128,
    pub effective_date: u64,
}

/// Modification type
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModificationType {
    PrincipalReduction = 0,
    InterestRateReduction = 1,
    TermExtension = 2,
    PaymentDeferral = 3,
    InterestForgiveness = 4,
}

/// Loan extension
#[contracttype]
#[derive(Clone)]
pub struct LoanExtension {
    pub id: u64,
    pub loan_id: u64,
    pub original_maturity: u64,
    pub new_maturity: u64,
    pub extension_fee: i128,
    pub approved_date: u64,
}

/// Payment deferral
#[contracttype]
#[derive(Clone)]
pub struct PaymentDeferral {
    pub id: u64,
    pub loan_id: u64,
    pub deferral_start: u64,
    pub deferral_end: u64,
    pub deferred_amount: i128,
    pub status: DeferralStatus,
}

/// Deferral status
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeferralStatus {
    Active = 0,
    Completed = 1,
    Terminated = 2,
    Failed = 3,
}

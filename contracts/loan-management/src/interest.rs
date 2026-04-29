//! Interest calculation module

use soroban_sdk::{contracttype, String};

/// Interest calculation result
#[contracttype]
#[derive(Clone)]
pub struct InterestCalculation {
    pub principal: i128,
    pub rate: u32,
    pub period: u64,
    pub interest_amount: i128,
    pub calculation_method: String,
}

/// Interest accrual schedule
#[contracttype]
#[derive(Clone)]
pub struct InterestAccrual {
    pub loan_id: u64,
    pub accrual_date: u64,
    pub accrued_amount: i128,
    pub paid_amount: i128,
    pub unpaid_amount: i128,
}

/// Interest rate adjustment
#[contracttype]
#[derive(Clone)]
pub struct InterestRateAdjustment {
    pub loan_id: u64,
    pub old_rate: u32,
    pub new_rate: u32,
    pub effective_date: u64,
    pub reason: String,
}

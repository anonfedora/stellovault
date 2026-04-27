//! Repayment processing module

use soroban_sdk::{contracttype, Address, String};

/// Repayment record
#[contracttype]
#[derive(Clone)]
pub struct RepaymentRecord {
    pub loan_id: u64,
    pub payer: Address,
    pub amount: i128,
    pub principal_payment: i128,
    pub interest_payment: i128,
    pub timestamp: u64,
}

/// Repayment schedule
#[contracttype]
#[derive(Clone)]
pub struct RepaymentSchedule {
    pub loan_id: u64,
    pub installments: soroban_sdk::Vec<Installment>,
    pub total_amount: i128,
    pub paid_amount: i128,
    pub status: ScheduleStatus,
}

/// Individual installment
#[contracttype]
#[derive(Clone)]
pub struct Installment {
    pub amount: i128,
    pub due_date: u64,
    pub paid: bool,
    pub paid_date: u64,
    pub principal: i128,
    pub interest: i128,
}

/// Schedule status
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScheduleStatus {
    Active = 0,
    Completed = 1,
    Defaulted = 2,
    Suspended = 3,
}

/// Early repayment calculation
#[contracttype]
#[derive(Clone)]
pub struct EarlyRepayment {
    pub loan_id: u64,
    pub outstanding_balance: i128,
    pub remaining_interest: i128,
    pub prepayment_penalty: i128,
    pub total_amount: i128,
}

/// Partial payment
#[contracttype]
#[derive(Clone)]
pub struct PartialPayment {
    pub loan_id: u64,
    pub amount: i128,
    pub principal_portion: i128,
    pub interest_portion: i128,
    pub timestamp: u64,
}

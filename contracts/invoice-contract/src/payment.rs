//! Payment terms and processing

use soroban_sdk::{contracttype, Address, String};

/// Payment terms structure
#[contracttype]
#[derive(Clone)]
pub struct PaymentTerms {
    pub discount_rate: u32,          // basis points
    pub early_payment_discount: u32, // basis points
    pub late_payment_fee: u32,       // basis points
    pub payment_deadline: u64,       // timestamp
}

/// Payment record
#[contracttype]
#[derive(Clone)]
pub struct PaymentRecord {
    pub invoice_id: u64,
    pub payer: Address,
    pub amount: i128,
    pub timestamp: u64,
    pub currency: String,
}

/// Payment schedule
#[contracttype]
#[derive(Clone)]
pub struct PaymentSchedule {
    pub invoice_id: u64,
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

/// Payment method
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PaymentMethod {
    DirectTransfer = 0,
    EscrowRelease = 1,
    AutomatedACH = 2,
    BlockchainTransfer = 3,
    StablecoinTransfer = 4,
}

/// Payment gateway configuration
#[contracttype]
#[derive(Clone)]
pub struct PaymentGateway {
    pub id: u64,
    pub provider: String,
    pub enabled: bool,
    pub fee_rate: u32, // basis points
    pub supported_currencies: soroban_sdk::Vec<String>,
}

/// Recurring payment setup
#[contracttype]
#[derive(Clone)]
pub struct RecurringPayment {
    pub id: u64,
    pub invoice_id: u64,
    pub payer: Address,
    pub amount: i128,
    pub frequency: PaymentFrequency,
    pub next_payment_date: u64,
    pub status: RecurringStatus,
}

/// Payment frequency
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PaymentFrequency {
    Daily = 0,
    Weekly = 1,
    BiWeekly = 2,
    Monthly = 3,
    Quarterly = 4,
    Annually = 5,
}

/// Recurring payment status
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecurringStatus {
    Active = 0,
    Paused = 1,
    Completed = 2,
    Cancelled = 3,
    Failed = 4,
}

/// Payment dispute
#[contracttype]
#[derive(Clone)]
pub struct PaymentDispute {
    pub id: u64,
    pub invoice_id: u64,
    pub payer: Address,
    pub amount: i128,
    pub reason: String,
    pub status: DisputeStatus,
    pub created_at: u64,
}

/// Dispute status
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DisputeStatus {
    Pending = 0,
    UnderReview = 1,
    Resolved = 2,
    Rejected = 3,
    Appealed = 4,
}

/// Payment reconciliation
#[contracttype]
#[derive(Clone)]
pub struct PaymentReconciliation {
    pub id: u64,
    pub invoice_id: u64,
    pub expected_amount: i128,
    pub received_amount: i128,
    pub variance: i128,
    pub reconciled: bool,
    pub reconciled_at: u64,
}

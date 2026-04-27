//! Invoice data structures and types

use soroban_sdk::{contracttype, Address, String};

/// Invoice status enumeration
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InvoiceStatus {
    Pending = 0,
    Verified = 1,
    PartiallyPaid = 2,
    Paid = 3,
    Disputed = 4,
    Cancelled = 5,
    Matured = 6,
}

/// Invoice data structure
#[contracttype]
#[derive(Clone)]
pub struct Invoice {
    pub id: u64,
    pub issuer: Address,
    pub buyer: Address,
    pub amount: i128,
    pub currency: String,
    pub invoice_number: String,
    pub issue_date: u64,
    pub due_date: u64,
    pub description: String,
    pub verification_hash: soroban_sdk::BytesN<32>,
    pub verified: bool,
    pub verification_timestamp: u64,
    pub paid_amount: i128,
    pub status: InvoiceStatus,
    pub owner: Address,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Invoice input data
#[contracttype]
#[derive(Clone)]
pub struct InvoiceData {
    pub buyer: Address,
    pub amount: i128,
    pub currency: String,
    pub invoice_number: String,
    pub due_date: u64,
    pub description: String,
}

/// Invoice portfolio for aggregated operations
#[contracttype]
#[derive(Clone)]
pub struct InvoicePortfolio {
    pub id: u64,
    pub owner: Address,
    pub invoice_ids: soroban_sdk::Vec<u64>,
    pub total_value: i128,
    pub total_paid: i128,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Invoice securitization structure
#[contracttype]
#[derive(Clone)]
pub struct InvoiceSecuritization {
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

/// Invoice insurance structure
#[contracttype]
#[derive(Clone)]
pub struct InvoiceInsurance {
    pub id: u64,
    pub invoice_id: u64,
    pub insurer: Address,
    pub coverage_amount: i128,
    pub premium_rate: u32, // basis points
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

/// Supply chain finance structure
#[contracttype]
#[derive(Clone)]
pub struct SupplyChainFinance {
    pub id: u64,
    pub invoice_id: u64,
    pub supplier: Address,
    pub buyer: Address,
    pub financing_amount: i128,
    pub financing_rate: u32, // basis points
    pub status: FinancingStatus,
    pub created_at: u64,
}

/// Financing status
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FinancingStatus {
    Pending = 0,
    Active = 1,
    Repaid = 2,
    Defaulted = 3,
}

/// Invoice factoring structure
#[contracttype]
#[derive(Clone)]
pub struct InvoiceFactoring {
    pub id: u64,
    pub invoice_id: u64,
    pub factor: Address,
    pub advance_amount: i128,
    pub factoring_fee: i128,
    pub status: FactoringStatus,
    pub created_at: u64,
}

/// Factoring status
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactoringStatus {
    Pending = 0,
    Active = 1,
    Completed = 2,
    Disputed = 3,
}

/// Invoice format types
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InvoiceFormat {
    Standard = 0,
    UBL = 1,
    PEPPOL = 2,
    EDI = 3,
    Custom = 4,
}

/// Invoice metadata
#[contracttype]
#[derive(Clone)]
pub struct InvoiceMetadata {
    pub invoice_id: u64,
    pub format: InvoiceFormat,
    pub document_uri: String,
    pub document_hash: soroban_sdk::BytesN<32>,
    pub metadata: String,
}

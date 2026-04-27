//! Cross-border payment and currency conversion

use soroban_sdk::{contracttype, Address, String};

/// Currency exchange rate
#[contracttype]
#[derive(Clone)]
pub struct ExchangeRate {
    pub id: u64,
    pub from_currency: String,
    pub to_currency: String,
    pub rate: i128,
    pub timestamp: u64,
    pub source: String,
}

/// Cross-border payment
#[contracttype]
#[derive(Clone)]
pub struct CrossBorderPayment {
    pub id: u64,
    pub invoice_id: u64,
    pub payer: Address,
    pub payee: Address,
    pub amount: i128,
    pub from_currency: String,
    pub to_currency: String,
    pub exchange_rate: i128,
    pub converted_amount: i128,
    pub status: PaymentStatus,
    pub created_at: u64,
}

/// Payment status
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PaymentStatus {
    Initiated = 0,
    Processing = 1,
    Completed = 2,
    Failed = 3,
    Reversed = 4,
}

/// Compliance check
#[contracttype]
#[derive(Clone)]
pub struct ComplianceCheck {
    pub id: u64,
    pub payment_id: u64,
    pub check_type: ComplianceType,
    pub status: ComplianceStatus,
    pub details: String,
    pub checked_at: u64,
}

/// Compliance type
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ComplianceType {
    AML = 0,
    KYC = 1,
    Sanctions = 2,
    OFAC = 3,
    RegulatoryClearance = 4,
}

/// Compliance status
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ComplianceStatus {
    Pending = 0,
    Approved = 1,
    Rejected = 2,
    RequiresReview = 3,
}

/// Jurisdiction information
#[contracttype]
#[derive(Clone)]
pub struct Jurisdiction {
    pub id: u64,
    pub country_code: String,
    pub country_name: String,
    pub risk_level: JurisdictionRisk,
    pub regulatory_requirements: String,
    pub supported: bool,
}

/// Jurisdiction risk level
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JurisdictionRisk {
    Low = 0,
    Medium = 1,
    High = 2,
    Restricted = 3,
}

/// Currency pair
#[contracttype]
#[derive(Clone)]
pub struct CurrencyPair {
    pub id: u64,
    pub base_currency: String,
    pub quote_currency: String,
    pub supported: bool,
    pub min_amount: i128,
    pub max_amount: i128,
}

/// Settlement instruction
#[contracttype]
#[derive(Clone)]
pub struct SettlementInstruction {
    pub id: u64,
    pub payment_id: u64,
    pub settlement_method: SettlementMethod,
    pub bank_details: String,
    pub status: SettlementStatus,
    pub created_at: u64,
}

/// Settlement method
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SettlementMethod {
    SWIFT = 0,
    SEPA = 1,
    ACH = 2,
    BlockchainTransfer = 3,
    StablecoinTransfer = 4,
    LocalBankTransfer = 5,
}

/// Settlement status
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SettlementStatus {
    Pending = 0,
    Initiated = 1,
    Confirmed = 2,
    Completed = 3,
    Failed = 4,
}

/// Correspondent bank
#[contracttype]
#[derive(Clone)]
pub struct CorrespondentBank {
    pub id: u64,
    pub bank_name: String,
    pub swift_code: String,
    pub country: String,
    pub supported_currencies: soroban_sdk::Vec<String>,
    pub active: bool,
}

/// Regulatory requirement
#[contracttype]
#[derive(Clone)]
pub struct RegulatoryRequirement {
    pub id: u64,
    pub jurisdiction: String,
    pub requirement_type: RequirementType,
    pub description: String,
    pub mandatory: bool,
}

/// Requirement type
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RequirementType {
    Documentation = 0,
    Verification = 1,
    Reporting = 2,
    Approval = 3,
    Disclosure = 4,
}

/// Tax information
#[contracttype]
#[derive(Clone)]
pub struct TaxInformation {
    pub id: u64,
    pub payment_id: u64,
    pub tax_type: TaxType,
    pub tax_rate: u32, // basis points
    pub tax_amount: i128,
    pub tax_jurisdiction: String,
}

/// Tax type
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TaxType {
    VAT = 0,
    GST = 1,
    WithholdingTax = 2,
    CorporateIncomeTax = 3,
    TransactionTax = 4,
}

/// Currency conversion record
#[contracttype]
#[derive(Clone)]
pub struct CurrencyConversion {
    pub id: u64,
    pub payment_id: u64,
    pub from_currency: String,
    pub to_currency: String,
    pub from_amount: i128,
    pub to_amount: i128,
    pub exchange_rate: i128,
    pub conversion_fee: i128,
    pub timestamp: u64,
}

/// Hedging strategy
#[contracttype]
#[derive(Clone)]
pub struct HedgingStrategy {
    pub id: u64,
    pub payment_id: u64,
    pub strategy_type: HedgingType,
    pub hedge_amount: i128,
    pub status: HedgeStatus,
    pub created_at: u64,
}

/// Hedging type
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HedgingType {
    Forward = 0,
    Option = 1,
    Swap = 2,
    Money = 3,
}

/// Hedge status
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HedgeStatus {
    Active = 0,
    Matured = 1,
    Exercised = 2,
    Expired = 3,
}

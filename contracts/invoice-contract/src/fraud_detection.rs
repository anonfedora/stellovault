//! Fraud detection and prevention

use crate::{ContractError, Invoice, VerificationData};
use soroban_sdk::{contracttype, Address, Env, String};

/// Fraud detection result
#[contracttype]
#[derive(Clone)]
pub struct FraudDetectionResult {
    pub invoice_id: u64,
    pub fraud_score: u32, // 0-10000 (0-100%)
    pub risk_level: RiskLevel,
    pub detected_anomalies: soroban_sdk::Vec<FraudIndicator>,
    pub timestamp: u64,
}

/// Risk level
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RiskLevel {
    Low = 0,
    Medium = 1,
    High = 2,
    Critical = 3,
}

/// Fraud indicator
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FraudIndicator {
    DuplicateInvoice = 0,
    UnusualAmount = 1,
    SuspiciousPattern = 2,
    UnknownBuyer = 3,
    UnknownIssuer = 4,
    InvalidDocumentHash = 5,
    TimestampAnomaly = 6,
    CurrencyMismatch = 7,
    HighRiskJurisdiction = 8,
    BlacklistedParty = 9,
    DocumentTampering = 10,
    VerificationFailure = 11,
}

/// Fraud rule
#[contracttype]
#[derive(Clone)]
pub struct FraudRule {
    pub id: u64,
    pub rule_type: FraudRuleType,
    pub threshold: u32,
    pub enabled: bool,
    pub weight: u32, // basis points
}

/// Fraud rule type
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FraudRuleType {
    AmountThreshold = 0,
    FrequencyThreshold = 1,
    GeographicRisk = 2,
    PartyBlacklist = 3,
    DocumentValidation = 4,
    PatternAnalysis = 5,
    BehavioralAnalysis = 6,
    NetworkAnalysis = 7,
}

/// Blacklist entry
#[contracttype]
#[derive(Clone)]
pub struct BlacklistEntry {
    pub id: u64,
    pub address: Address,
    pub reason: String,
    pub added_at: u64,
    pub expires_at: u64,
    pub active: bool,
}

/// Whitelist entry
#[contracttype]
#[derive(Clone)]
pub struct WhitelistEntry {
    pub id: u64,
    pub address: Address,
    pub trust_level: u32, // 0-10000
    pub added_at: u64,
    pub active: bool,
}

/// Fraud alert
#[contracttype]
#[derive(Clone)]
pub struct FraudAlert {
    pub id: u64,
    pub invoice_id: u64,
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub message: String,
    pub created_at: u64,
    pub resolved: bool,
}

/// Alert type
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AlertType {
    SuspiciousActivity = 0,
    DocumentAnomaly = 1,
    PartyRisk = 2,
    AmountAnomaly = 3,
    VerificationFailure = 4,
    BlacklistMatch = 5,
    PatternMatch = 6,
}

/// Alert severity
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AlertSeverity {
    Info = 0,
    Warning = 1,
    Critical = 2,
}

/// Fraud case
#[contracttype]
#[derive(Clone)]
pub struct FraudCase {
    pub id: u64,
    pub invoice_id: u64,
    pub reported_by: Address,
    pub description: String,
    pub status: CaseStatus,
    pub created_at: u64,
    pub resolved_at: u64,
}

/// Case status
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CaseStatus {
    Reported = 0,
    UnderInvestigation = 1,
    Confirmed = 2,
    Dismissed = 3,
    Resolved = 4,
}

/// Fraud statistics
#[contracttype]
#[derive(Clone)]
pub struct FraudStatistics {
    pub total_invoices_checked: u64,
    pub fraudulent_invoices_detected: u64,
    pub false_positives: u64,
    pub true_positives: u64,
    pub detection_accuracy: u32, // basis points
    pub last_updated: u64,
}

/// Implement fraud detection logic
pub fn detect_fraud(
    env: &Env,
    invoice: &Invoice,
    verification_data: &VerificationData,
) -> Result<u32, ContractError> {
    let mut fraud_score: u32 = 0;

    // Check for unusual amounts (simplified)
    if invoice.amount > 100_000_000_000 {
        fraud_score += 200; // Add 2% to fraud score
    }

    // Check verification confidence
    if verification_data.confidence_score < 7000 {
        fraud_score += 300; // Add 3% to fraud score
    }

    // Check for timestamp anomalies
    if verification_data.verification_timestamp > env.ledger().timestamp() {
        fraud_score += 500; // Add 5% to fraud score
    }

    Ok(fraud_score)
}

/// Risk assessment
pub fn assess_risk(fraud_score: u32) -> RiskLevel {
    match fraud_score {
        0..=2500 => RiskLevel::Low,
        2501..=5000 => RiskLevel::Medium,
        5001..=7500 => RiskLevel::High,
        _ => RiskLevel::Critical,
    }
}

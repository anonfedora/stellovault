//! Collateral verification and authenticity checks

use soroban_sdk::{contracttype, Address, BytesN, String};

/// Verification data
#[contracttype]
#[derive(Clone)]
pub struct VerificationData {
    pub document_hash: BytesN<32>,
    pub verification_method: VerificationMethod,
    pub additional_data: String,
}

/// Verification method enumeration
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VerificationMethod {
    DocumentReview = 0,
    ThirdPartyAttestation = 1,
    BlockchainVerification = 2,
    LegalReview = 3,
    PhysicalInspection = 4,
}

/// Verification record
#[contracttype]
#[derive(Clone)]
pub struct VerificationRecord {
    pub collateral_id: u64,
    pub verified_by: Address,
    pub verification_method: VerificationMethod,
    pub verified_at: u64,
}

/// Verification status enumeration
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VerificationStatus {
    Pending = 0,
    Verified = 1,
    Failed = 2,
    Expired = 3,
}

/// Verification report
#[contracttype]
#[derive(Clone)]
pub struct VerificationReport {
    pub id: u64,
    pub collateral_id: u64,
    pub verifier: Address,
    pub verification_method: VerificationMethod,
    pub status: VerificationStatus,
    pub findings: String,
    pub report_hash: BytesN<32>,
    pub created_at: u64,
    pub valid_until: u64,
}

/// Fraud detection result
#[contracttype]
#[derive(Clone)]
pub struct FraudDetectionResult {
    pub collateral_id: u64,
    pub risk_level: RiskLevel,
    pub risk_score: u32,
    pub detected_issues: soroban_sdk::Vec<FraudIndicator>,
    pub checked_at: u64,
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
    DocumentMismatch = 0,
    DuplicateAsset = 1,
    InvalidSignature = 2,
    SuspiciousValuation = 3,
    UnauthorizedTransfer = 4,
    InconsistentMetadata = 5,
}

/// Verification challenge
#[contracttype]
#[derive(Clone)]
pub struct VerificationChallenge {
    pub id: u64,
    pub collateral_id: u64,
    pub challenger: Address,
    pub challenge_reason: String,
    pub status: ChallengeStatus,
    pub created_at: u64,
    pub resolved_at: u64,
}

/// Challenge status
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChallengeStatus {
    Open = 0,
    UnderReview = 1,
    Resolved = 2,
    Rejected = 3,
}

/// Verification audit trail
#[contracttype]
#[derive(Clone)]
pub struct VerificationAuditTrail {
    pub collateral_id: u64,
    pub verifications: soroban_sdk::Vec<VerificationRecord>,
}

/// Document verification
#[contracttype]
#[derive(Clone)]
pub struct DocumentVerification {
    pub id: u64,
    pub collateral_id: u64,
    pub document_hash: BytesN<32>,
    pub document_type: DocumentType,
    pub verified_by: Address,
    pub status: VerificationStatus,
    pub verified_at: u64,
}

/// Document type
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentType {
    Title = 0,
    Appraisal = 1,
    Insurance = 2,
    Inspection = 3,
    Certificate = 4,
    Other = 5,
}

/// Merkle tree verification
#[contracttype]
#[derive(Clone)]
pub struct MerkleTreeVerification {
    pub id: u64,
    pub collateral_id: u64,
    pub root_hash: BytesN<32>,
    pub leaf_count: u32,
    pub verified_at: u64,
}

/// Verification certificate
#[contracttype]
#[derive(Clone)]
pub struct VerificationCertificate {
    pub id: u64,
    pub collateral_id: u64,
    pub issuer: Address,
    pub certificate_hash: BytesN<32>,
    pub issued_at: u64,
    pub expires_at: u64,
    pub status: CertificateStatus,
}

/// Certificate status
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CertificateStatus {
    Valid = 0,
    Expired = 1,
    Revoked = 2,
    Suspended = 3,
}

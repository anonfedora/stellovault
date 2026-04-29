//! Invoice verification and authentication

use soroban_sdk::{contracttype, Address, BytesN, String};

/// Verification data structure
#[contracttype]
#[derive(Clone)]
pub struct VerificationData {
    pub document_hash: BytesN<32>,
    pub verification_timestamp: u64,
    pub verifier: Address,
    pub verification_method: VerificationMethod,
    pub confidence_score: u32, // 0-10000 (0-100%)
    pub metadata: String,
}

/// Verification method
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VerificationMethod {
    DocumentHash = 0,
    DigitalSignature = 1,
    ThirdPartyVerification = 2,
    BlockchainProof = 3,
    MultiSignature = 4,
    OracleVerification = 5,
}

/// Verification record
#[contracttype]
#[derive(Clone)]
pub struct VerificationRecord {
    pub id: u64,
    pub invoice_id: u64,
    pub verifier: Address,
    pub verification_method: VerificationMethod,
    pub document_hash: BytesN<32>,
    pub confidence_score: u32,
    pub verified_at: u64,
    pub expires_at: u64,
}

/// Multi-signature verification
#[contracttype]
#[derive(Clone)]
pub struct MultiSigVerification {
    pub id: u64,
    pub invoice_id: u64,
    pub required_signatures: u32,
    pub collected_signatures: u32,
    pub signers: soroban_sdk::Vec<Address>,
    pub status: VerificationStatus,
}

/// Verification status
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VerificationStatus {
    Pending = 0,
    PartiallyVerified = 1,
    FullyVerified = 2,
    Failed = 3,
    Expired = 4,
}

/// Document verification
#[contracttype]
#[derive(Clone)]
pub struct DocumentVerification {
    pub id: u64,
    pub invoice_id: u64,
    pub document_type: DocumentType,
    pub document_hash: BytesN<32>,
    pub document_uri: String,
    pub verified: bool,
    pub verified_at: u64,
}

/// Document type
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentType {
    Invoice = 0,
    PurchaseOrder = 1,
    ShippingDocument = 2,
    DeliveryProof = 3,
    QualityReport = 4,
    InsuranceCertificate = 5,
    BankStatement = 6,
    CustomsDocument = 7,
}

/// Oracle verification
#[contracttype]
#[derive(Clone)]
pub struct OracleVerification {
    pub id: u64,
    pub invoice_id: u64,
    pub oracle_address: Address,
    pub verification_data: String,
    pub confidence_score: u32,
    pub verified_at: u64,
}

/// Verification audit trail
#[contracttype]
#[derive(Clone)]
pub struct VerificationAuditTrail {
    pub id: u64,
    pub invoice_id: u64,
    pub action: VerificationAction,
    pub actor: Address,
    pub timestamp: u64,
    pub details: String,
}

/// Verification action
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VerificationAction {
    Initiated = 0,
    DocumentSubmitted = 1,
    SignatureCollected = 2,
    OracleQueried = 3,
    VerificationCompleted = 4,
    VerificationFailed = 5,
    VerificationExpired = 6,
}

/// Verification certificate
#[contracttype]
#[derive(Clone)]
pub struct VerificationCertificate {
    pub id: u64,
    pub invoice_id: u64,
    pub issuer: Address,
    pub certificate_hash: BytesN<32>,
    pub issued_at: u64,
    pub expires_at: u64,
    pub revoked: bool,
}

/// Batch verification
#[contracttype]
#[derive(Clone)]
pub struct BatchVerification {
    pub id: u64,
    pub invoice_ids: soroban_sdk::Vec<u64>,
    pub total_invoices: u32,
    pub verified_count: u32,
    pub failed_count: u32,
    pub status: BatchVerificationStatus,
}

/// Batch verification status
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BatchVerificationStatus {
    Pending = 0,
    InProgress = 1,
    Completed = 2,
    PartiallyCompleted = 3,
    Failed = 4,
}

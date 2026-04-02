//! Data models for document verification

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Document verification status
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "verification_status", rename_all = "snake_case")]
pub enum VerificationStatus {
    Pending,
    Verified,
    Rejected,
    ManualReview,
    Expired,
}

/// Document type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentType {
    Invoice,
    Contract,
    Receipt,
    TaxDocument,
}

impl DocumentType {
    pub fn as_str(&self) -> &str {
        match self {
            DocumentType::Invoice => "invoice",
            DocumentType::Contract => "contract",
            DocumentType::Receipt => "receipt",
            DocumentType::TaxDocument => "tax_document",
        }
    }
}

/// Document verification record from database
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DocumentVerification {
    pub id: Uuid,
    pub user_id: Uuid,
    pub risk_score_id: Option<Uuid>,
    pub document_type: String,
    pub document_url: String,
    pub document_hash: Option<String>,
    pub file_size_bytes: Option<i64>,
    pub mime_type: Option<String>,
    pub extracted_text: Option<String>,
    pub extracted_fields: Option<serde_json::Value>,
    pub verification_status: VerificationStatus,
    pub verification_score: Option<i32>,
    pub confidence: Option<f64>,
    pub fraud_indicators: Option<serde_json::Value>,
    pub ocr_provider: Option<String>,
    pub processing_time_ms: Option<i32>,
    pub error_message: Option<String>,
    pub verified_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request to upload and verify a document
#[derive(Debug, Deserialize)]
pub struct DocumentUploadRequest {
    pub user_id: Uuid,
    pub document_type: DocumentType,
    pub file_name: String,
    pub file_data: Vec<u8>,
}

/// Response after document verification
#[derive(Debug, Serialize)]
pub struct DocumentVerificationResponse {
    pub verification_id: Uuid,
    pub status: VerificationStatus,
    pub verification_score: Option<i32>,
    pub confidence: Option<f64>,
    pub extracted_fields: Option<serde_json::Value>,
    pub fraud_indicators: Vec<FraudIndicator>,
    pub requires_manual_review: bool,
    pub processing_time_ms: i32,
}

/// Fraud indicator detected in document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FraudIndicator {
    pub indicator_type: String,
    pub severity: FraudSeverity,
    pub description: String,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FraudSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Extracted fields from document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedFields {
    pub invoice_number: Option<String>,
    pub date: Option<String>,
    pub amount: Option<f64>,
    pub currency: Option<String>,
    pub vendor_name: Option<String>,
    pub vendor_address: Option<String>,
    pub line_items: Vec<LineItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineItem {
    pub description: String,
    pub quantity: Option<f64>,
    pub unit_price: Option<f64>,
    pub total: Option<f64>,
}

/// OCR result from provider
#[derive(Debug, Clone)]
pub struct OcrResult {
    pub extracted_text: String,
    pub extracted_fields: ExtractedFields,
    pub confidence: f64,
    pub processing_time_ms: i32,
}

/// Error types for document verification
#[derive(Debug, thiserror::Error)]
pub enum DocumentVerificationError {
    #[error("Invalid document format: {0}")]
    InvalidFormat(String),
    
    #[error("Document too large: {0} bytes (max: {1} bytes)")]
    FileTooLarge(usize, usize),
    
    #[error("OCR processing failed: {0}")]
    OcrFailed(String),
    
    #[error("Storage error: {0}")]
    StorageError(String),
    
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
}

pub type DocumentVerificationResult<T> = Result<T, DocumentVerificationError>;

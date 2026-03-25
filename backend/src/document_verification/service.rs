//! Document Verification Service - Main business logic

use chrono::Utc;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use std::path::PathBuf;
use uuid::Uuid;

use super::model::*;
use super::ocr_provider::{create_ocr_provider, OcrProvider, OcrProviderConfig};
use super::validator::{DocumentValidator, FraudDetectionLevel};

/// Document verification service
pub struct DocumentVerificationService {
    db_pool: PgPool,
    ocr_provider: Box<dyn OcrProvider>,
    validator: DocumentValidator,
    storage_path: PathBuf,
    max_file_size_mb: usize,
}

impl DocumentVerificationService {
    pub fn new(
        db_pool: PgPool,
        ocr_provider_type: String,
        ocr_config: OcrProviderConfig,
        storage_path: PathBuf,
        max_file_size_mb: usize,
        min_confidence: f64,
        fraud_detection_level: String,
    ) -> Self {
        let ocr_provider = create_ocr_provider(&ocr_provider_type, &ocr_config);
        
        let fraud_level = match fraud_detection_level.as_str() {
            "low" => FraudDetectionLevel::Low,
            "high" => FraudDetectionLevel::High,
            _ => FraudDetectionLevel::Medium,
        };

        let validator = DocumentValidator::new(min_confidence, fraud_level);

        Self {
            db_pool,
            ocr_provider,
            validator,
            storage_path,
            max_file_size_mb,
        }
    }

    /// Upload and verify a document
    pub async fn verify_document(
        &self,
        request: DocumentUploadRequest,
    ) -> DocumentVerificationResult<DocumentVerificationResponse> {
        let start_time = std::time::Instant::now();

        // Validate file size
        let file_size = request.file_data.len();
        let max_size = self.max_file_size_mb * 1024 * 1024;
        if file_size > max_size {
            return Err(DocumentVerificationError::FileTooLarge(file_size, max_size));
        }

        // Generate document hash
        let document_hash = self.calculate_hash(&request.file_data);

        // Save file to storage
        let file_path = self.save_file(&request, &document_hash).await?;

        // Extract text using OCR
        let ocr_result = self.ocr_provider
            .extract_text(&file_path)
            .await
            .map_err(|e| DocumentVerificationError::OcrFailed(e.to_string()))?;

        // Validate and score
        let (verification_score, fraud_indicators, requires_manual_review) =
            self.validator.validate_and_score(&ocr_result.extracted_fields, ocr_result.confidence);

        // Determine status
        let status = self.validator.determine_status(verification_score, requires_manual_review);

        // Store in database
        let verification_id = self.store_verification(
            &request,
            &document_hash,
            file_size as i64,
            &ocr_result,
            verification_score,
            &fraud_indicators,
            &status,
        ).await?;

        let processing_time_ms = start_time.elapsed().as_millis() as i32;

        tracing::info!(
            verification_id = %verification_id,
            user_id = %request.user_id,
            score = verification_score,
            status = ?status,
            processing_time_ms = processing_time_ms,
            "Document verification completed"
        );

        Ok(DocumentVerificationResponse {
            verification_id,
            status,
            verification_score: Some(verification_score),
            confidence: Some(ocr_result.confidence),
            extracted_fields: Some(serde_json::to_value(&ocr_result.extracted_fields).unwrap()),
            fraud_indicators,
            requires_manual_review,
            processing_time_ms,
        })
    }

    /// Get verification by ID
    pub async fn get_verification(
        &self,
        verification_id: &Uuid,
    ) -> DocumentVerificationResult<DocumentVerification> {
        sqlx::query_as::<_, DocumentVerification>(
            "SELECT * FROM document_verifications WHERE id = $1"
        )
        .bind(verification_id)
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|e| DocumentVerificationError::DatabaseError(e.to_string()))?
        .ok_or_else(|| DocumentVerificationError::ValidationError("Verification not found".to_string()))
    }

    /// Get all verifications for a user
    pub async fn get_user_verifications(
        &self,
        user_id: &Uuid,
    ) -> DocumentVerificationResult<Vec<DocumentVerification>> {
        sqlx::query_as::<_, DocumentVerification>(
            "SELECT * FROM document_verifications WHERE user_id = $1 ORDER BY created_at DESC"
        )
        .bind(user_id)
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| DocumentVerificationError::DatabaseError(e.to_string()))
    }

    /// Calculate average verification score for a user
    pub async fn get_user_average_score(
        &self,
        user_id: &Uuid,
    ) -> DocumentVerificationResult<Option<i32>> {
        let result: Option<(Option<f64>,)> = sqlx::query_as(
            r#"
            SELECT AVG(verification_score) 
            FROM document_verifications 
            WHERE user_id = $1 
            AND verification_status = 'verified'
            AND verification_score IS NOT NULL
            "#
        )
        .bind(user_id)
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|e| DocumentVerificationError::DatabaseError(e.to_string()))?;

        Ok(result.and_then(|(avg,)| avg.map(|v| v.round() as i32)))
    }

    /// Calculate document hash
    fn calculate_hash(&self, data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    /// Save file to storage
    async fn save_file(
        &self,
        request: &DocumentUploadRequest,
        document_hash: &str,
    ) -> DocumentVerificationResult<PathBuf> {
        // Create storage directory if it doesn't exist
        tokio::fs::create_dir_all(&self.storage_path)
            .await
            .map_err(|e| DocumentVerificationError::StorageError(e.to_string()))?;

        // Generate unique filename
        let extension = request.file_name
            .split('.')
            .last()
            .unwrap_or("bin");
        let filename = format!("{}_{}.{}", request.user_id, document_hash, extension);
        let file_path = self.storage_path.join(filename);

        // Write file
        tokio::fs::write(&file_path, &request.file_data)
            .await
            .map_err(|e| DocumentVerificationError::StorageError(e.to_string()))?;

        Ok(file_path)
    }

    /// Store verification in database
    async fn store_verification(
        &self,
        request: &DocumentUploadRequest,
        document_hash: &str,
        file_size: i64,
        ocr_result: &OcrResult,
        verification_score: i32,
        fraud_indicators: &[FraudIndicator],
        status: &VerificationStatus,
    ) -> DocumentVerificationResult<Uuid> {
        let verification_id = Uuid::new_v4();
        let document_url = format!("file://{}/{}", self.storage_path.display(), document_hash);
        let mime_type = self.detect_mime_type(&request.file_name);

        let extracted_fields_json = serde_json::to_value(&ocr_result.extracted_fields)
            .map_err(|e| DocumentVerificationError::ValidationError(e.to_string()))?;

        let fraud_indicators_json = serde_json::to_value(fraud_indicators)
            .map_err(|e| DocumentVerificationError::ValidationError(e.to_string()))?;

        sqlx::query(
            r#"
            INSERT INTO document_verifications (
                id, user_id, document_type, document_url, document_hash,
                file_size_bytes, mime_type, extracted_text, extracted_fields,
                verification_status, verification_score, confidence, fraud_indicators,
                ocr_provider, processing_time_ms, verified_at, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, NOW(), NOW())
            "#
        )
        .bind(verification_id)
        .bind(request.user_id)
        .bind(request.document_type.as_str())
        .bind(&document_url)
        .bind(document_hash)
        .bind(file_size)
        .bind(&mime_type)
        .bind(&ocr_result.extracted_text)
        .bind(&extracted_fields_json)
        .bind(status)
        .bind(verification_score)
        .bind(ocr_result.confidence)
        .bind(&fraud_indicators_json)
        .bind(self.ocr_provider.provider_name())
        .bind(ocr_result.processing_time_ms)
        .bind(if matches!(status, VerificationStatus::Verified) { Some(Utc::now()) } else { None })
        .execute(&self.db_pool)
        .await
        .map_err(|e| DocumentVerificationError::DatabaseError(e.to_string()))?;

        Ok(verification_id)
    }

    /// Detect MIME type from filename
    fn detect_mime_type(&self, filename: &str) -> String {
        match filename.split('.').last() {
            Some("pdf") => "application/pdf",
            Some("png") => "image/png",
            Some("jpg") | Some("jpeg") => "image/jpeg",
            _ => "application/octet-stream",
        }.to_string()
    }
}

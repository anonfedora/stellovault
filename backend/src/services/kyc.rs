//! KYC/AML compliance service
//!
//! Placeholder service that can be swapped for real providers like Sumsub or Persona

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::ApiError;
use crate::models::{KycStatus, User};

/// KYC verification request
#[derive(Debug, Deserialize)]
pub struct KycVerificationRequest {
    pub user_id: Uuid,
    pub provider: Option<String>,
    pub reference_id: Option<String>,
}

/// KYC verification response
#[derive(Debug, Serialize)]
pub struct KycVerificationResponse {
    pub user_id: Uuid,
    pub kyc_status: KycStatus,
    pub kyc_expiry: Option<DateTime<Utc>>,
    pub message: String,
}

/// KYC status check response
#[derive(Debug, Serialize)]
pub struct KycStatusResponse {
    pub user_id: Uuid,
    pub kyc_status: KycStatus,
    pub kyc_expiry: Option<DateTime<Utc>>,
    pub is_valid: bool,
    pub requires_verification: bool,
}

/// KYC compliance service
#[derive(Clone)]
pub struct KycService {
    db: PgPool,
}

impl KycService {
    /// Create a new KYC service
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// Check if user meets KYC requirements for a transaction
    pub async fn check_kyc_compliance(
        &self,
        user_id: &Uuid,
        transaction_amount: i64,
    ) -> Result<bool, ApiError> {
        let user = self.get_user(user_id).await?;

        // Threshold: $10,000 (assuming amount is in cents)
        const KYC_THRESHOLD: i64 = 10_000_00;

        // If transaction is below threshold, no KYC required
        if transaction_amount < KYC_THRESHOLD {
            return Ok(true);
        }

        // For transactions >= $10,000, require verified KYC
        match user.kyc_status {
            KycStatus::Verified => {
                // Check if KYC is expired
                if let Some(expiry) = user.kyc_expiry {
                    if expiry < Utc::now() {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    /// Get KYC status for a user
    pub async fn get_kyc_status(&self, user_id: &Uuid) -> Result<KycStatusResponse, ApiError> {
        let user = self.get_user(user_id).await?;

        let is_valid = match user.kyc_status {
            KycStatus::Verified => {
                if let Some(expiry) = user.kyc_expiry {
                    expiry >= Utc::now()
                } else {
                    true
                }
            }
            _ => false,
        };

        let requires_verification = matches!(
            user.kyc_status,
            KycStatus::Unverified | KycStatus::Expired | KycStatus::Rejected
        );

        Ok(KycStatusResponse {
            user_id: user.id,
            kyc_status: user.kyc_status,
            kyc_expiry: user.kyc_expiry,
            is_valid,
            requires_verification,
        })
    }

    /// Initiate KYC verification (placeholder for real provider integration)
    pub async fn initiate_verification(
        &self,
        user_id: &Uuid,
    ) -> Result<KycVerificationResponse, ApiError> {
        sqlx::query(
            r#"UPDATE users SET kyc_status = 'pending'::kyc_status, updated_at = NOW() WHERE id = $1"#,
        )
        .bind(user_id)
        .execute(&self.db)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(KycVerificationResponse {
            user_id: *user_id,
            kyc_status: KycStatus::Pending,
            kyc_expiry: None,
            message: "KYC verification initiated. Please complete the verification process."
                .to_string(),
        })
    }

    /// Mock verification approval (for testing - replace with real provider webhook)
    pub async fn mock_approve_verification(
        &self,
        user_id: &Uuid,
        provider: Option<String>,
        reference_id: Option<String>,
    ) -> Result<KycVerificationResponse, ApiError> {
        let expiry = Utc::now() + Duration::days(365);

        sqlx::query(
            r#"UPDATE users SET kyc_status = 'verified'::kyc_status, kyc_expiry = $2,
               kyc_provider = $3, kyc_reference_id = $4, kyc_verified_at = NOW(), updated_at = NOW()
               WHERE id = $1"#,
        )
        .bind(user_id)
        .bind(expiry)
        .bind(provider.as_deref())
        .bind(reference_id.as_deref())
        .execute(&self.db)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(KycVerificationResponse {
            user_id: *user_id,
            kyc_status: KycStatus::Verified,
            kyc_expiry: Some(expiry),
            message: "KYC verification approved successfully.".to_string(),
        })
    }

    /// Mock verification rejection (for testing)
    pub async fn mock_reject_verification(
        &self,
        user_id: &Uuid,
        reason: Option<String>,
    ) -> Result<KycVerificationResponse, ApiError> {
        sqlx::query(
            r#"UPDATE users SET kyc_status = 'rejected'::kyc_status, updated_at = NOW() WHERE id = $1"#,
        )
        .bind(user_id)
        .execute(&self.db)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(KycVerificationResponse {
            user_id: *user_id,
            kyc_status: KycStatus::Rejected,
            kyc_expiry: None,
            message: reason.unwrap_or_else(|| "KYC verification rejected.".to_string()),
        })
    }

    /// Expire KYC verification
    pub async fn expire_verification(&self, user_id: &Uuid) -> Result<(), ApiError> {
        sqlx::query(
            r#"UPDATE users SET kyc_status = 'expired'::kyc_status, updated_at = NOW()
               WHERE id = $1 AND kyc_status = 'verified'::kyc_status"#,
        )
        .bind(user_id)
        .execute(&self.db)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Bulk expire all expired KYC verifications
    pub async fn expire_all_expired(&self) -> Result<u64, ApiError> {
        let result = sqlx::query(
            r#"UPDATE users SET kyc_status = 'expired'::kyc_status
               WHERE kyc_status = 'verified'::kyc_status
                 AND kyc_expiry IS NOT NULL AND kyc_expiry < NOW()"#,
        )
        .execute(&self.db)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(result.rows_affected())
    }

    // Helper method to get user
    async fn get_user(&self, user_id: &Uuid) -> Result<User, ApiError> {
        sqlx::query_as::<_, User>(
            r#"SELECT id, primary_wallet_address, email, name,
                      role as "role: _", risk_score,
                      kyc_status as "kyc_status: _", kyc_expiry,
                      kyc_provider, kyc_verified_at, kyc_reference_id,
                      created_at, updated_at
               FROM users WHERE id = $1"#,
        )
        .bind(user_id)
        .fetch_optional(&self.db)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?
        .ok_or(ApiError::NotFound("User not found".to_string()))
    }
}

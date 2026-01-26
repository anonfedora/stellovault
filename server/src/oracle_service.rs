//! Oracle service for managing oracle providers and confirmations

use crate::models::{Oracle, OracleConfirmation, OracleConfirmationRequest, OracleRegistrationRequest, VerificationStatus, OracleMetrics};
use sqlx::{PgPool, Error};
use uuid::Uuid;
use std::collections::HashMap;

/// Oracle service for managing oracle providers and confirmations
pub struct OracleService {
    pool: PgPool,
}

impl OracleService {
    /// Create a new oracle service instance
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Register a new oracle provider
    pub async fn register_oracle(&self, request: OracleRegistrationRequest, added_by: Option<Uuid>) -> Result<Oracle, Error> {
        let oracle = sqlx::query_as::<_, Oracle>(
            r#"
            INSERT INTO oracles (address, name, endpoint_url, public_key, added_by)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, address, name, endpoint_url, public_key, is_active,
                      reputation_score, total_confirmations, successful_confirmations,
                      added_at, added_by, updated_at
            "#
        )
        .bind(request.address)
        .bind(request.name)
        .bind(request.endpoint_url)
        .bind(request.public_key)
        .bind(added_by)
        .fetch_one(&self.pool)
        .await?;

        Ok(oracle)
    }

    /// Get oracle by address
    pub async fn get_oracle_by_address(&self, address: &str) -> Result<Option<Oracle>, Error> {
        let oracle = sqlx::query_as::<_, Oracle>(
            r#"
            SELECT id, address, name, endpoint_url, public_key, is_active,
                   reputation_score, total_confirmations, successful_confirmations,
                   added_at, added_by, updated_at
            FROM oracles
            WHERE address = $1
            "#
        )
        .bind(address)
        .fetch_optional(&self.pool)
        .await?;

        Ok(oracle)
    }

    /// Get all active oracles
    pub async fn get_active_oracles(&self) -> Result<Vec<Oracle>, Error> {
        let oracles = sqlx::query_as::<_, Oracle>(
            r#"
            SELECT id, address, name, endpoint_url, public_key, is_active,
                   reputation_score, total_confirmations, successful_confirmations,
                   added_at, added_by, updated_at
            FROM oracles
            WHERE is_active = true
            ORDER BY reputation_score DESC NULLS LAST
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(oracles)
    }

    /// Deactivate an oracle
    pub async fn deactivate_oracle(&self, address: &str) -> Result<(), Error> {
        sqlx::query(
            r#"
            UPDATE oracles
            SET is_active = false, updated_at = NOW()
            WHERE address = $1
            "#
        )
        .bind(address)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Submit oracle confirmation
    pub async fn submit_confirmation(&self, request: OracleConfirmationRequest, oracle_address: &str) -> Result<OracleConfirmation, Error> {
        // Check if confirmation already exists (prevent replay attacks)
        let existing = sqlx::query(
            r#"SELECT id FROM oracle_confirmations WHERE escrow_id = $1 AND oracle_address = $2"#
        )
        .bind(&request.escrow_id)
        .bind(oracle_address)
        .fetch_optional(&self.pool)
        .await?;

        if existing.is_some() {
            return Err(Error::RowNotFound);
        }

        // Validate event type
        if request.event_type < 1 || request.event_type > 4 {
            return Err(Error::Protocol("Invalid event type".to_string()));
        }

        // TODO: Verify signature against oracle's public key
        // For now, we'll mark as verified
        let verification_status = VerificationStatus::Verified;

        let confirmation = sqlx::query_as::<_, OracleConfirmation>(
            r#"
            INSERT INTO oracle_confirmations (escrow_id, oracle_address, event_type, result, signature, verification_status)
            VALUES ($1, $2, $3, $4, $5, $6::verification_status)
            RETURNING id, escrow_id, oracle_address, event_type, result, signature,
                      transaction_hash, block_number, gas_used, confirmed_at,
                      verification_status as "verification_status: VerificationStatus", error_message
            "#
        )
        .bind(request.escrow_id)
        .bind(oracle_address)
        .bind(request.event_type)
        .bind(request.result)
        .bind(request.signature)
        .bind(verification_status as VerificationStatus)
        .fetch_one(&self.pool)
        .await?;

        // Update oracle statistics
        self.update_oracle_stats(oracle_address).await?;

        Ok(confirmation)
    }

    /// Get confirmations for an escrow
    pub async fn get_confirmations_for_escrow(&self, escrow_id: &str) -> Result<Vec<OracleConfirmation>, Error> {
        let confirmations = sqlx::query_as::<_, OracleConfirmation>(
            r#"
            SELECT id, escrow_id, oracle_address, event_type, result, signature,
                   transaction_hash, block_number, gas_used, confirmed_at,
                   verification_status as "verification_status: VerificationStatus", error_message
            FROM oracle_confirmations
            WHERE escrow_id = $1
            ORDER BY confirmed_at DESC
            "#
        )
        .bind(escrow_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(confirmations)
    }

    /// Get oracle metrics for dashboard
    pub async fn get_oracle_metrics(&self) -> Result<OracleMetrics, Error> {
        #[derive(sqlx::FromRow)]
        struct MetricsRow {
            total_oracles: Option<i64>,
            active_oracles: Option<i64>,
            total_confirmations: Option<i64>,
            successful_confirmations: Option<i64>,
            average_reputation_score: Option<f64>,
        }

        let metrics: MetricsRow = sqlx::query_as::<_, MetricsRow>(
            r#"
            SELECT
                (SELECT COUNT(*) FROM oracles) as total_oracles,
                (SELECT COUNT(*) FROM oracles WHERE is_active = true) as active_oracles,
                (SELECT COALESCE(SUM(total_confirmations), 0) FROM oracles) as total_confirmations,
                (SELECT COALESCE(SUM(successful_confirmations), 0) FROM oracles) as successful_confirmations,
                (SELECT COALESCE(AVG(reputation_score), 0) FROM oracles WHERE reputation_score IS NOT NULL) as average_reputation_score
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(OracleMetrics {
            total_oracles: metrics.total_oracles.unwrap_or(0),
            active_oracles: metrics.active_oracles.unwrap_or(0),
            total_confirmations: metrics.total_confirmations.unwrap_or(0),
            successful_confirmations: metrics.successful_confirmations.unwrap_or(0),
            average_reputation_score: metrics.average_reputation_score.unwrap_or(0.0),
        })
    }

    /// Verify oracle signature (placeholder - implement cryptographic verification)
    pub async fn verify_signature(&self, message: &[u8], signature: &str, oracle_address: &str) -> Result<bool, Error> {
        // Get oracle's public key
        let oracle = self.get_oracle_by_address(oracle_address).await?;
        let oracle = match oracle {
            Some(o) => o,
            None => return Ok(false),
        };

        let public_key = match oracle.public_key {
            Some(pk) => pk,
            None => return Ok(false),
        };

        // TODO: Implement proper cryptographic signature verification
        // For now, return true if public key exists and signature is not empty
        Ok(!signature.is_empty() && !public_key.is_empty())
    }

    /// Update oracle statistics after confirmation
    async fn update_oracle_stats(&self, oracle_address: &str) -> Result<(), Error> {
        sqlx::query(
            r#"
            UPDATE oracles
            SET
                total_confirmations = total_confirmations + 1,
                successful_confirmations = successful_confirmations + 1,
                reputation_score = LEAST(100.0, reputation_score + 1.0),
                updated_at = NOW()
            WHERE address = $1
            "#
        )
        .bind(oracle_address)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get confirmations grouped by event type for an escrow
    pub async fn get_confirmations_by_event_type(&self, escrow_id: &str) -> Result<HashMap<i32, Vec<OracleConfirmation>>, Error> {
        let confirmations = self.get_confirmations_for_escrow(escrow_id).await?;
        let mut grouped = HashMap::new();

        for confirmation in confirmations {
            grouped.entry(confirmation.event_type)
                .or_insert_with(Vec::new)
                .push(confirmation);
        }

        Ok(grouped)
    }

    /// Check if escrow has required confirmations for a specific event type
    pub async fn has_required_confirmations(&self, escrow_id: &str, event_type: i32, required_count: usize) -> Result<bool, Error> {
        let confirmations = self.get_confirmations_for_escrow(escrow_id).await?;
        let count = confirmations.iter()
            .filter(|c| c.event_type == event_type && c.verification_status == VerificationStatus::Verified)
            .count();

        Ok(count >= required_count)
    }
}
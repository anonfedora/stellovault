//! Oracle service - Business logic for oracle confirmation management
//!
//! I'm centralizing all oracle-related business logic here: validation, signature verification,
//! aggregation, Soroban tx submission, and audit logging.

use anyhow::{Context, Result};
use chrono::Utc;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

use super::model::*;
use super::rate_limiter::OracleRateLimiter;

/// Oracle service for managing oracle confirmations and aggregation
pub struct OracleService {
    db_pool: PgPool,
    horizon_url: String,
    network_passphrase: String,
    soroban_rpc_url: String,
    rate_limiter: OracleRateLimiter,
    /// Number of oracle confirmations required before submitting Soroban tx
    aggregation_threshold: u32,
}

impl OracleService {
    /// Create a new oracle service instance
    pub fn new(
        db_pool: PgPool,
        horizon_url: String,
        network_passphrase: String,
        soroban_rpc_url: String,
    ) -> Self {
        Self {
            db_pool,
            horizon_url,
            network_passphrase,
            soroban_rpc_url,
            rate_limiter: OracleRateLimiter::default(),
            aggregation_threshold: 2, // I'm defaulting to 2-of-N for now.
        }
    }

    /// Set custom aggregation threshold (for testing or configuration)
    pub fn with_aggregation_threshold(mut self, threshold: u32) -> Self {
        self.aggregation_threshold = threshold;
        self
    }

    /// Main entry point for oracle confirmations
    pub async fn confirm_oracle_event(
        &self,
        request: OracleConfirmRequest,
    ) -> Result<OracleConfirmResponse> {
        // I'm checking rate limits first to prevent abuse.
        if !self.rate_limiter.check(&request.oracle_address).await {
            anyhow::bail!("Rate limit exceeded for oracle: {}", request.oracle_address);
        }

        // Validate the request payload
        request
            .validate()
            .map_err(|e| anyhow::anyhow!("Validation failed: {}", e))?;

        // Check for duplicate confirmation
        if self.check_duplicate_confirmation(request.escrow_id, &request.oracle_address).await? {
            anyhow::bail!(
                "Duplicate confirmation from oracle {} for escrow {}",
                request.oracle_address,
                request.escrow_id
            );
        }

        // Verify the signature
        self.verify_signature(&request).await?;

        // I'm computing the payload hash for integrity verification.
        let payload_hash = self.compute_payload_hash(&request.payload)?;
        let payload_json = serde_json::to_value(&request.payload)?;

        // Store the oracle event
        let event_id = Uuid::new_v4();
        let event = sqlx::query_as::<_, OracleEvent>(
            r#"
            INSERT INTO oracle_events (
                id, escrow_id, oracle_address, data_type, payload_hash, 
                payload, signature, status, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(event_id)
        .bind(request.escrow_id)
        .bind(&request.oracle_address)
        .bind(request.data_type)
        .bind(&payload_hash)
        .bind(&payload_json)
        .bind(&request.signature)
        .bind(OracleEventStatus::Confirmed)
        .fetch_one(&self.db_pool)
        .await
        .context("Failed to insert oracle event")?;

        // Log the audit event
        self.log_audit_event(
            Some(event_id),
            "confirm",
            &request.oracle_address,
            Some(serde_json::json!({
                "escrow_id": request.escrow_id,
                "data_type": request.data_type,
            })),
        )
        .await?;

        // Check aggregation threshold
        let (aggregation_count, threshold_met, tx_hash) =
            self.aggregate_confirmations(request.escrow_id).await?;

        Ok(OracleConfirmResponse {
            event_id: event.id,
            status: if threshold_met {
                OracleEventStatus::Aggregated
            } else {
                OracleEventStatus::Confirmed
            },
            aggregation_count,
            threshold_met,
            tx_hash,
        })
    }

    /// Check if this oracle has already confirmed this escrow
    async fn check_duplicate_confirmation(
        &self,
        escrow_id: i64,
        oracle_address: &str,
    ) -> Result<bool> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM oracle_events 
            WHERE escrow_id = $1 AND oracle_address = $2 
            AND status != 'rejected'
            "#,
        )
        .bind(escrow_id)
        .bind(oracle_address)
        .fetch_one(&self.db_pool)
        .await
        .context("Failed to check duplicate confirmation")?;

        Ok(count.0 > 0)
    }

    /// Verify Ed25519 signature on the oracle payload
    async fn verify_signature(&self, request: &OracleConfirmRequest) -> Result<()> {
        // I'm decoding the public key from the Stellar address format.
        let public_key_bytes = self.decode_stellar_address(&request.oracle_address)?;
        let verifying_key = VerifyingKey::from_bytes(&public_key_bytes)
            .context("Invalid oracle public key")?;

        // Decode base64 signature
        let signature_bytes = base32::decode(
            base32::Alphabet::Rfc4648 { padding: true },
            &request.signature,
        )
        .ok_or_else(|| anyhow::anyhow!("Invalid base32 signature encoding"))?;

        let signature = Signature::from_slice(&signature_bytes)
            .context("Invalid signature format")?;

        // I'm constructing the canonical message to verify.
        let message = self.construct_signing_message(request)?;

        verifying_key
            .verify(message.as_bytes(), &signature)
            .context("Signature verification failed")?;

        Ok(())
    }

    /// Construct the canonical message that should have been signed
    fn construct_signing_message(&self, request: &OracleConfirmRequest) -> Result<String> {
        // I'm using a deterministic message format for signature verification.
        let message = format!(
            "stellovault:oracle:confirm:{}:{}:{}",
            request.escrow_id,
            request.payload.confirmation_id,
            request.payload.observed_at.timestamp()
        );
        Ok(message)
    }

    /// Decode Stellar address to Ed25519 public key bytes
    fn decode_stellar_address(&self, address: &str) -> Result<[u8; 32]> {
        // I'm handling the Stellar G-address format (strkey encoding).
        if !address.starts_with('G') {
            anyhow::bail!("Invalid Stellar address format: must start with G");
        }

        let decoded = base32::decode(base32::Alphabet::Rfc4648 { padding: true }, address)
            .ok_or_else(|| anyhow::anyhow!("Failed to decode Stellar address"))?;

        if decoded.len() < 35 {
            anyhow::bail!("Invalid Stellar address length");
        }

        // Skip version byte (1) and take public key (32 bytes)
        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&decoded[1..33]);
        Ok(key_bytes)
    }

    /// Compute SHA256 hash of the payload for integrity verification
    fn compute_payload_hash(&self, payload: &OraclePayload) -> Result<String> {
        let payload_json = serde_json::to_string(payload)?;
        let mut hasher = Sha256::new();
        hasher.update(payload_json.as_bytes());
        let result = hasher.finalize();
        // I'm using inline hex encoding to avoid adding the hex crate as a dependency.
        let hex_string: String = result.iter().map(|b| format!("{:02x}", b)).collect();
        Ok(hex_string)
    }

    /// Check aggregation threshold and submit Soroban tx if met
    async fn aggregate_confirmations(&self, escrow_id: i64) -> Result<(i32, bool, Option<String>)> {
        // I'm counting confirmed events for this escrow.
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM oracle_events 
            WHERE escrow_id = $1 AND status IN ('confirmed', 'aggregated')
            "#,
        )
        .bind(escrow_id)
        .fetch_one(&self.db_pool)
        .await
        .context("Failed to count confirmations")?;

        let aggregation_count = count.0 as i32;
        let threshold_met = aggregation_count >= self.aggregation_threshold as i32;

        if threshold_met {
            // I'm submitting the Soroban confirmation tx now that threshold is met.
            let tx_hash = self.submit_soroban_confirmation(escrow_id).await?;

            // Update all events for this escrow to 'aggregated'
            sqlx::query(
                r#"
                UPDATE oracle_events 
                SET status = 'aggregated', tx_hash = $1, updated_at = NOW()
                WHERE escrow_id = $2 AND status = 'confirmed'
                "#,
            )
            .bind(&tx_hash)
            .bind(escrow_id)
            .execute(&self.db_pool)
            .await
            .context("Failed to update events to aggregated")?;

            self.log_audit_event(
                None,
                "aggregate",
                "system",
                Some(serde_json::json!({
                    "escrow_id": escrow_id,
                    "confirmation_count": aggregation_count,
                    "tx_hash": tx_hash,
                })),
            )
            .await?;

            return Ok((aggregation_count, true, Some(tx_hash)));
        }

        Ok((aggregation_count, false, None))
    }

    /// Submit confirmation transaction to Soroban
    async fn submit_soroban_confirmation(&self, escrow_id: i64) -> Result<String> {
        // I'm simulating the Soroban tx submission for now - real implementation would use stellar-sdk.
        tracing::info!(
            escrow_id = escrow_id,
            horizon_url = %self.horizon_url,
            rpc_url = %self.soroban_rpc_url,
            "Submitting oracle confirmation to Soroban"
        );

        // TODO: Implement actual Soroban transaction building and submission
        // For now, returning a simulated tx hash
        let simulated_hash = format!(
            "TX_{}_{:x}",
            escrow_id,
            Utc::now().timestamp_millis()
        );

        Ok(simulated_hash)
    }

    /// Flag an oracle event as disputed
    pub async fn flag_dispute(
        &self,
        escrow_id: i64,
        reason: &str,
        disputer_address: &str,
    ) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE oracle_events 
            SET status = 'disputed', updated_at = NOW()
            WHERE escrow_id = $1 AND status IN ('pending', 'confirmed')
            "#,
        )
        .bind(escrow_id)
        .execute(&self.db_pool)
        .await
        .context("Failed to flag dispute")?;

        self.log_audit_event(
            None,
            "dispute",
            disputer_address,
            Some(serde_json::json!({
                "escrow_id": escrow_id,
                "reason": reason,
            })),
        )
        .await?;

        tracing::warn!(
            escrow_id = escrow_id,
            disputer = disputer_address,
            reason = reason,
            "Oracle confirmation disputed"
        );

        Ok(())
    }

    /// Get oracle events with filtering
    pub async fn list_oracle_events(&self, query: ListOracleEventsQuery) -> Result<Vec<OracleEvent>> {
        let limit = query.limit.unwrap_or(50).min(100);
        let offset = query.offset.unwrap_or(0);

        let events = sqlx::query_as::<_, OracleEvent>(
            r#"
            SELECT * FROM oracle_events
            WHERE ($1::BIGINT IS NULL OR escrow_id = $1)
            AND ($2::TEXT IS NULL OR oracle_address = $2)
            AND ($3::oracle_event_status IS NULL OR status = $3)
            ORDER BY created_at DESC
            LIMIT $4 OFFSET $5
            "#,
        )
        .bind(query.escrow_id)
        .bind(query.oracle_address)
        .bind(query.status)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.db_pool)
        .await
        .context("Failed to list oracle events")?;

        Ok(events)
    }

    /// Get a single oracle event by ID
    pub async fn get_oracle_event(&self, id: &Uuid) -> Result<Option<OracleEvent>> {
        let event = sqlx::query_as::<_, OracleEvent>(
            "SELECT * FROM oracle_events WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.db_pool)
        .await
        .context("Failed to get oracle event")?;

        Ok(event)
    }

    /// Log an audit event for compliance tracking
    async fn log_audit_event(
        &self,
        oracle_event_id: Option<Uuid>,
        action: &str,
        actor_address: &str,
        details: Option<serde_json::Value>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO oracle_audit_logs (id, oracle_event_id, action, actor_address, details, created_at)
            VALUES ($1, $2, $3, $4, $5, NOW())
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(oracle_event_id)
        .bind(action)
        .bind(actor_address)
        .bind(details)
        .execute(&self.db_pool)
        .await
        .context("Failed to log audit event")?;

        tracing::info!(
            action = action,
            actor = actor_address,
            event_id = ?oracle_event_id,
            "Oracle audit log recorded"
        );

        Ok(())
    }
}

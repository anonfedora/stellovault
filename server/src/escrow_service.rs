//! Escrow service layer - Business logic for escrow management

use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use sqlx::PgPool;
use rand::Rng;
use uuid::Uuid;

use crate::escrow::{
    CreateEscrowRequest, CreateEscrowResponse, Escrow, EscrowEvent, EscrowStatus,
    EscrowWithCollateral, ListEscrowsQuery,
};
use crate::models::{CollateralToken, TokenStatus};

/// Escrow service for managing escrow lifecycle
pub struct EscrowService {
    db_pool: PgPool,
    _horizon_url: String,
    _network_passphrase: String,
}

impl EscrowService {
    /// Create new escrow service instance
    pub fn new(db_pool: PgPool, horizon_url: String, network_passphrase: String) -> Self {
        Self {
            db_pool,
            _horizon_url: horizon_url,
            _network_passphrase: network_passphrase,
        }
    }

    /// Create an escrow on-chain and in database
    pub async fn create_escrow(
        &self,
        request: CreateEscrowRequest,
    ) -> Result<CreateEscrowResponse> {
        // Validate collateral exists and is not locked
        let collateral = self.get_collateral(&request.collateral_id).await?;
        if collateral.status != TokenStatus::Active {
            anyhow::bail!("Collateral is not available for escrow");
        }

        // Calculate timeout
        let timeout_at = request
            .timeout_hours
            .map(|hours| Utc::now() + Duration::hours(hours));

        // Create escrow on-chain via Soroban contract
        let token_id_u64 = collateral.token_id.parse::<u64>()
            .map_err(|e| anyhow::anyhow!("Invalid token_id: {}. Error: {}", collateral.token_id, e))?;

        let (escrow_id, tx_hash) = self
            .create_on_chain_escrow(
                &request.buyer_id,
                &request.seller_id,
                &request.lender_id,
                token_id_u64,
                request.amount,
                &request.oracle_address,
                &request.release_conditions,
                timeout_at,
            )
            .await?;

        // Store escrow in database
        let db_id = Uuid::new_v4();
        let escrow = sqlx::query_as::<_, Escrow>(
            r#"
            INSERT INTO escrows (
                id, escrow_id, buyer_id, seller_id, lender_id, collateral_id, amount,
                status, oracle_address, release_conditions, timeout_at, disputed,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            RETURNING *
            "#,
        )
        .bind(db_id)
        .bind(escrow_id as i64)
        .bind(request.buyer_id)
        .bind(request.seller_id)
        .bind(request.lender_id)
        .bind(request.collateral_id)
        .bind(request.amount)
        .bind(EscrowStatus::Pending)
        .bind(&request.oracle_address)
        .bind(&request.release_conditions)
        .bind(timeout_at)
        .bind(false)
        .bind(Utc::now())
        .bind(Utc::now())
        .fetch_one(&self.db_pool)
        .await
        .context("Failed to insert escrow into database")?;

        // Lock the collateral
        self.lock_collateral(&request.collateral_id).await?;

        Ok(CreateEscrowResponse {
            id: escrow.id,
            escrow_id,
            status: EscrowStatus::Pending,
            tx_hash,
        })
    }

    /// Get a single escrow by ID
    pub async fn get_escrow(&self, id: &Uuid) -> Result<Option<Escrow>> {
        let escrow = sqlx::query_as::<_, Escrow>("SELECT * FROM escrows WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.db_pool)
            .await?;

        Ok(escrow)
    }

    /// Get escrow with collateral details
    pub async fn get_escrow_with_collateral(
        &self,
        id: &Uuid,
    ) -> Result<Option<EscrowWithCollateral>> {
        let escrow = sqlx::query_as::<_, EscrowWithCollateral>(
            r#"
            SELECT 
                e.*,
                c.token_id,
                c.asset_type::text,
                c.asset_value
            FROM escrows e
            JOIN collateral_tokens c ON e.collateral_id = c.id
            WHERE e.id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.db_pool)
        .await?;

        Ok(escrow)
    }

    /// List escrows with filtering and pagination
    pub async fn list_escrows(&self, query: ListEscrowsQuery) -> Result<Vec<Escrow>> {
        let page = query.page.unwrap_or(1).max(1);
        let limit = query.limit.unwrap_or(20).clamp(1, 100);
        let offset = (page - 1) * limit;

        let mut query_builder: sqlx::QueryBuilder<sqlx::Postgres> = 
            sqlx::QueryBuilder::new("SELECT * FROM escrows WHERE 1=1");

        if let Some(status) = query.status {
            query_builder.push(" AND status = ");
            query_builder.push_bind(status);
        }
        if let Some(buyer_id) = query.buyer_id {
            query_builder.push(" AND buyer_id = ");
            query_builder.push_bind(buyer_id);
        }
        if let Some(seller_id) = query.seller_id {
            query_builder.push(" AND seller_id = ");
            query_builder.push_bind(seller_id);
        }

        query_builder.push(" ORDER BY created_at DESC LIMIT ");
        query_builder.push_bind(limit as i64);
        query_builder.push(" OFFSET ");
        query_builder.push_bind(offset as i64);

        let escrows = query_builder
            .build_query_as::<Escrow>()
            .fetch_all(&self.db_pool)
            .await?;

        Ok(escrows)
    }

    /// Track and update escrow status from on-chain state
    pub async fn track_escrow_status(&self, escrow_id: i64) -> Result<EscrowStatus> {
        // Query on-chain escrow status
        let on_chain_status = self.query_on_chain_status(escrow_id).await?;

        // Update database if status changed
        sqlx::query(
            r#"
            UPDATE escrows 
            SET status = $1, updated_at = $2 
            WHERE escrow_id = $3 AND status != $1
            "#,
        )
        .bind(on_chain_status)
        .bind(Utc::now())
        .bind(escrow_id as i64)
        .execute(&self.db_pool)
        .await?;

        Ok(on_chain_status)
    }

    /// Process escrow event from Soroban
    pub async fn process_escrow_event(&self, event: EscrowEvent) -> Result<()> {
        match event {
            EscrowEvent::Created { escrow_id, .. } => {
                tracing::info!("Escrow created event: {}", escrow_id);
                // Event already processed during creation
                Ok(())
            }
            EscrowEvent::Activated { escrow_id } => {
                self.update_escrow_status(escrow_id, EscrowStatus::Active)
                    .await?;
                tracing::info!("Escrow {} activated", escrow_id);
                Ok(())
            }
            EscrowEvent::Released { escrow_id } => {
                self.update_escrow_status(escrow_id, EscrowStatus::Released)
                    .await?;
                tracing::info!("Escrow {} released", escrow_id);
                Ok(())
            }
            EscrowEvent::Cancelled { escrow_id } => {
                self.update_escrow_status(escrow_id, EscrowStatus::Cancelled)
                    .await?;
                tracing::info!("Escrow {} cancelled", escrow_id);
                Ok(())
            }
            EscrowEvent::TimedOut { escrow_id } => {
                self.update_escrow_status(escrow_id, EscrowStatus::TimedOut)
                    .await?;
                tracing::info!("Escrow {} timed out", escrow_id);
                Ok(())
            }
            EscrowEvent::Disputed { escrow_id, reason } => {
                self.mark_disputed(escrow_id, &reason).await?;
                tracing::warn!("Escrow {} disputed: {}", escrow_id, reason);
                Ok(())
            }
            EscrowEvent::StatusUpdated { escrow_id, status } => {
                self.update_escrow_status(escrow_id, status).await?;
                Ok(())
            }
        }
    }

    /// Detect and handle timed-out escrows
    pub async fn detect_timeouts(&self) -> Result<Vec<i64>> {
        let timed_out = sqlx::query_as::<_, (i64,)>(
            r#"
            UPDATE escrows 
            SET status = 'timedout', updated_at = $1
            WHERE timeout_at IS NOT NULL 
              AND timeout_at < $1 
              AND status IN ('pending', 'active')
            RETURNING escrow_id
            "#,
        )
        .bind(Utc::now())
        .fetch_all(&self.db_pool)
        .await?;

        let escrow_ids: Vec<i64> = timed_out.iter().map(|(id,)| *id as i64).collect();

        for escrow_id in &escrow_ids {
            tracing::warn!("Escrow {} has timed out", escrow_id);
        }

        Ok(escrow_ids)
    }

    // ===== Private Helper Methods =====

    /// Create escrow on Soroban smart contract
    async fn create_on_chain_escrow(
        &self,
        _buyer_id: &Uuid,
        _seller_id: &Uuid,
        _lender_id: &Uuid,
        collateral_token_id: u64,
        amount: i64,
        oracle_address: &str,
        _release_conditions: &str,
        timeout_at: Option<DateTime<Utc>>,
    ) -> Result<(i64, String)> {
        // TODO: Implement actual Soroban contract interaction
        // For now, simulate contract call
        tracing::info!(
            "Creating on-chain escrow: collateral={}, amount={}, oracle={}",
            collateral_token_id,
            amount,
            oracle_address
        );

        // Simulated response
        let escrow_id = rand::thread_rng().gen_range(1..i64::MAX);
        let tx_hash = format!("sim_{}", Uuid::new_v4().to_string().replace("-", ""));

        tracing::warn!("Using simulated on-chain escrow creation - implement Soroban SDK integration");

        Ok((escrow_id, tx_hash))
    }

    /// Query on-chain escrow status from Soroban
    async fn query_on_chain_status(&self, escrow_id: i64) -> Result<EscrowStatus> {
        // TODO: Implement actual Soroban contract query
        tracing::info!("Querying on-chain status for escrow {}", escrow_id);

        // Simulate query
        let status = sqlx::query_as::<_, (EscrowStatus,)>(
            "SELECT status FROM escrows WHERE escrow_id = $1",
        )
        .bind(escrow_id as i64)
        .fetch_one(&self.db_pool)
        .await?;

        Ok(status.0)
    }

    /// Update escrow status in database
    async fn update_escrow_status(&self, escrow_id: i64, status: EscrowStatus) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE escrows 
            SET status = $1, updated_at = $2 
            WHERE escrow_id = $3
            "#,
        )
        .bind(status)
        .bind(Utc::now())
        .bind(escrow_id as i64)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    /// Mark escrow as disputed
    async fn mark_disputed(&self, escrow_id: i64, _reason: &str) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE escrows 
            SET status = 'disputed', disputed = true, updated_at = $1
            WHERE escrow_id = $2
            "#,
        )
        .bind(Utc::now())
        .bind(escrow_id as i64)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    /// Get collateral by ID
    async fn get_collateral(&self, id: &Uuid) -> Result<CollateralToken> {
        let collateral = sqlx::query_as::<_, CollateralToken>(
            "SELECT * FROM collateral_tokens WHERE id = $1",
        )
        .bind(id)
        .fetch_one(&self.db_pool)
        .await
        .context("Collateral not found")?;

        Ok(collateral)
    }

    /// Lock collateral when used in escrow
    async fn lock_collateral(&self, id: &Uuid) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE collateral_tokens 
            SET status = 'locked', updated_at = $1 
            WHERE id = $2
            "#,
        )
        .bind(Utc::now())
        .bind(id)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }
}

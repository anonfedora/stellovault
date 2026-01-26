//! Collateral service module

use sqlx::PgPool;
use uuid::Uuid;
use crate::models::{Collateral, CollateralStatus, CreateCollateralRequest, ListCollateralQuery};
use anyhow::Result;

pub mod indexer;

#[allow(dead_code)]
pub struct CollateralService {
    pool: PgPool,
    horizon_url: String,
    network_passphrase: String,
    contract_id: String,
}

impl CollateralService {
    pub fn new(pool: PgPool, horizon_url: String, network_passphrase: String, contract_id: String) -> Self {
        Self {
            pool,
            horizon_url,
            network_passphrase,
            contract_id,
        }
    }

    pub async fn create_collateral(&self, req: CreateCollateralRequest) -> Result<Collateral> {
        // 1. Validate inputs (handled by validator in handler)
        
        // 2. Generate IDs
        let id = Uuid::new_v4();
        // For now, generate a random token ID or derive it. 
        let token_id = Uuid::new_v4().to_string(); 

        // 3. Register on-chain (Simulated)
        let tx_hash = self.register_on_chain(&token_id, &req).await?;

        // 4. Store in DB
        let collateral = sqlx::query_as::<_, Collateral>(
            r#"
            INSERT INTO collateral (
                id, token_id, owner_id, asset_type, asset_value, 
                metadata_hash, fractional_shares, status, tx_hash, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW(), NOW())
            RETURNING id, token_id, owner_id, asset_type, asset_value, metadata_hash, fractional_shares, status, tx_hash, created_at, updated_at
            "#
        )
        .bind(id)
        .bind(token_id)
        .bind(req.owner_id)
        .bind(req.asset_type)
        .bind(req.asset_value)
        .bind(req.metadata_hash)
        .bind(req.fractional_shares)
        .bind(CollateralStatus::Active)
        .bind(tx_hash)
        .fetch_one(&self.pool)
        .await?;

        Ok(collateral)
    }

    pub async fn get_collateral(&self, id: Uuid) -> Result<Option<Collateral>> {
        let collateral = sqlx::query_as::<_, Collateral>(
            r#"
            SELECT 
                id, token_id, owner_id, asset_type, 
                asset_value, metadata_hash, fractional_shares, 
                status, tx_hash, created_at, updated_at
            FROM collateral
            WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(collateral)
    }

    pub async fn get_collateral_by_metadata(&self, metadata_hash: &str) -> Result<Option<Collateral>> {
        let collateral = sqlx::query_as::<_, Collateral>(
            r#"
            SELECT 
                id, token_id, owner_id, asset_type, 
                asset_value, metadata_hash, fractional_shares, 
                status, tx_hash, created_at, updated_at
            FROM collateral
            WHERE metadata_hash = $1
            "#
        )
        .bind(metadata_hash)
        .fetch_optional(&self.pool)
        .await?;

        Ok(collateral)
    }

    pub async fn get_collateral_by_token_id(&self, token_id: &str) -> Result<Option<Collateral>> {
        let collateral = sqlx::query_as::<_, Collateral>(
            r#"
            SELECT 
                id, token_id, owner_id, asset_type, 
                asset_value, metadata_hash, fractional_shares, 
                status, tx_hash, created_at, updated_at
            FROM collateral
            WHERE token_id = $1
            "#
        )
        .bind(token_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(collateral)
    }

    pub async fn update_lock_status(&self, token_id: &str, locked: bool) -> Result<()> {
        let status = if locked {
            CollateralStatus::Locked
        } else {
            CollateralStatus::Active
        };

        sqlx::query(
            "UPDATE collateral SET status = $1, updated_at = NOW() WHERE token_id = $2"
        )
        .bind(status)
        .bind(token_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn update_status(&self, id: Uuid, status: CollateralStatus) -> Result<()> {
        sqlx::query(
            "UPDATE collateral SET status = $1, updated_at = NOW() WHERE id = $2"
        )
        .bind(status)
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn list_collateral(&self, query: ListCollateralQuery) -> Result<Vec<Collateral>> {
        let limit = query.limit.unwrap_or(10);
        let offset = (query.page.unwrap_or(1) - 1) * limit;

        let collaterals = sqlx::query_as::<_, Collateral>(
            r#"
            SELECT 
                id, token_id, owner_id, asset_type, 
                asset_value, metadata_hash, fractional_shares, 
                status, tx_hash, created_at, updated_at
            FROM collateral
            WHERE ($1::uuid IS NULL OR owner_id = $1)
            AND ($2::token_status IS NULL OR status = $2)
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
            "#
        )
        .bind(query.owner_id)
        .bind(query.status)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(collaterals)
    }

    /// Simulate registering collateral on the Soroban contract
    async fn register_on_chain(&self, token_id: &str, _req: &CreateCollateralRequest) -> Result<String> {
        // TODO: Implement actual Soroban invocation
        // For now, return a mock transaction hash
        tracing::info!("Simulating on-chain registration for token_id: {}", token_id);
        Ok(format!("tx_simulated_{}", token_id))
    }
}

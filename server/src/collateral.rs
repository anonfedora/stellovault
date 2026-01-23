//! Collateral registry service for StelloVault backend
//!
//! This module provides business logic for collateral management,
//! including double-collateralization protection and metadata hash mapping.

use crate::models::{Collateral, CollateralStatus};
use crate::state::AppState;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

/// Collateral service error
#[derive(Debug, thiserror::Error)]
pub enum CollateralError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Collateral not found")]
    NotFound,
    #[error("Duplicate metadata hash")]
    DuplicateMetadata,
    #[error("Invalid collateral data")]
    InvalidData,
    #[error("Unauthorized operation")]
    Unauthorized,
}

/// Create collateral request
#[derive(Debug, Deserialize)]
pub struct CreateCollateralRequest {
    pub owner_id: Uuid,
    pub collateral_id: String,
    pub face_value: i64,
    pub expiry_ts: i64,
    pub metadata_hash: String,
    pub registered_at: DateTime<Utc>,
}

/// Collateral service
pub struct CollateralService {
    pool: Arc<PgPool>,
}

impl CollateralService {
    /// Create a new collateral service
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    /// Create collateral record (mirror from Soroban contract)
    ///
    /// # Arguments
    /// * `req` - Collateral creation request
    ///
    /// # Returns
    /// The created collateral record
    ///
    /// # Errors
    /// Returns error if metadata hash already exists (double-collateralization protection)
    pub async fn create_collateral(
        &self,
        req: CreateCollateralRequest,
    ) -> Result<Collateral, CollateralError> {
        // Check for duplicate metadata hash (double-collateralization protection)
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM collateral WHERE metadata_hash = $1")
                .bind(&req.metadata_hash)
                .fetch_one(&*self.pool)
                .await?;

        if count > 0 {
            return Err(CollateralError::DuplicateMetadata);
        }

        // Validate expiry timestamp
        let now = Utc::now().timestamp();
        if req.expiry_ts <= now {
            return Err(CollateralError::InvalidData);
        }

        // Insert collateral record
        let collateral = sqlx::query_as::<_, Collateral>(
            r#"
            INSERT INTO collateral (
                collateral_id, owner_id, face_value, expiry_ts,
                metadata_hash, registered_at, locked, status
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, collateral_id, owner_id, face_value, expiry_ts,
                      metadata_hash, registered_at, locked, status,
                      created_at, updated_at
            "#,
        )
        .bind(&req.collateral_id)
        .bind(&req.owner_id)
        .bind(&req.face_value)
        .bind(&req.expiry_ts)
        .bind(&req.metadata_hash)
        .bind(&req.registered_at)
        .bind(false) // locked
        .bind(CollateralStatus::Active as CollateralStatus)
        .fetch_one(&*self.pool)
        .await?;

        Ok(collateral)
    }

    /// Get collateral by ID
    ///
    /// # Arguments
    /// * `collateral_id` - The collateral ID to look up
    ///
    /// # Returns
    /// Option containing collateral if found
    pub async fn get_collateral(
        &self,
        collateral_id: &str,
    ) -> Result<Option<Collateral>, CollateralError> {
        let collateral =
            sqlx::query_as::<_, Collateral>("SELECT * FROM collateral WHERE collateral_id = $1")
                .bind(collateral_id)
                .fetch_optional(&*self.pool)
                .await?;

        Ok(collateral)
    }

    /// Get collateral by metadata hash
    ///
    /// # Arguments
    /// * `metadata_hash` - The metadata hash to look up
    ///
    /// # Returns
    /// Option containing collateral if found
    pub async fn get_collateral_by_metadata(
        &self,
        metadata_hash: &str,
    ) -> Result<Option<Collateral>, CollateralError> {
        let collateral =
            sqlx::query_as::<_, Collateral>("SELECT * FROM collateral WHERE metadata_hash = $1")
                .bind(metadata_hash)
                .fetch_optional(&*self.pool)
                .await?;

        Ok(collateral)
    }

    /// Update collateral lock status
    ///
    /// # Arguments
    /// * `collateral_id` - The collateral ID to update
    /// * `locked` - New lock status
    ///
    /// # Returns
    /// Updated collateral record
    pub async fn update_lock_status(
        &self,
        collateral_id: &str,
        locked: bool,
    ) -> Result<Collateral, CollateralError> {
        let status = if locked {
            CollateralStatus::Locked
        } else {
            CollateralStatus::Active
        };

        let collateral = sqlx::query_as::<_, Collateral>(
            r#"
            UPDATE collateral
            SET locked = $1, status = $2, updated_at = NOW()
            WHERE collateral_id = $3
            RETURNING id, collateral_id, owner_id, face_value, expiry_ts,
                      metadata_hash, registered_at, locked, status,
                      created_at, updated_at
            "#,
        )
        .bind(locked)
        .bind(status as CollateralStatus)
        .bind(collateral_id)
        .fetch_optional(&*self.pool)
        .await?
        .ok_or(CollateralError::NotFound)?;

        Ok(collateral)
    }

    /// List collateral for a user
    ///
    /// # Arguments
    /// * `owner_id` - The owner ID to filter by
    /// * `limit` - Maximum number of records to return
    /// * `offset` - Number of records to skip
    ///
    /// # Returns
    /// Vector of collateral records
    pub async fn list_user_collateral(
        &self,
        owner_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Collateral>, CollateralError> {
        let collateral = sqlx::query_as::<_, Collateral>(
            r#"
            SELECT * FROM collateral
            WHERE owner_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(owner_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&*self.pool)
        .await?;

        Ok(collateral)
    }

    /// Check if metadata hash is already registered (double-collateralization check)
    ///
    /// # Arguments
    /// * `metadata_hash` - The metadata hash to check
    ///
    /// # Returns
    /// True if hash is already registered
    pub async fn is_metadata_registered(
        &self,
        metadata_hash: &str,
    ) -> Result<bool, CollateralError> {
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM collateral WHERE metadata_hash = $1")
                .bind(metadata_hash)
                .fetch_one(&*self.pool)
                .await?;

        Ok(count > 0)
    }

    /// Get expired collateral
    ///
    /// # Arguments
    /// * `current_ts` - Current timestamp to compare against expiry
    ///
    /// # Returns
    /// Vector of expired collateral IDs
    pub async fn get_expired_collateral(
        &self,
        current_ts: i64,
    ) -> Result<Vec<String>, CollateralError> {
        let expired: Vec<(String,)> = sqlx::query_as(
            r#"
            SELECT collateral_id FROM collateral
            WHERE expiry_ts <= $1 AND status = 'active'
            "#,
        )
        .bind(current_ts)
        .fetch_all(&*self.pool)
        .await?;

        let ids = expired
            .into_iter()
            .map(|(collateral_id,)| collateral_id)
            .collect();

        Ok(ids)
    }
}

/// Get collateral service from app state
pub fn get_collateral_service(state: &AppState) -> Arc<CollateralService> {
    Arc::clone(&state.collateral_service)
}

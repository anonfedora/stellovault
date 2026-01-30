use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::collateral::model::{Collateral, CollateralFilter, CreateCollateralRequest, PaginatedResponse};
use crate::error::ApiError;
use crate::models::CollateralStatus;

#[derive(Clone)]
pub struct CollateralService {
    db_pool: PgPool,
    rpc_url: String,
    contract_id: String,
}

impl CollateralService {
    pub fn new(db_pool: PgPool, rpc_url: String, contract_id: String) -> Self {
        Self {
            db_pool,
            rpc_url,
            contract_id,
        }
    }

    pub async fn create_collateral(
        &self,
        request: CreateCollateralRequest,
    ) -> Result<Collateral, ApiError> {
        // 1. Validate inputs
        if request.face_value <= 0 {
            return Err(ApiError::BadRequest("Face value must be positive".to_string()));
        }

        // 2. Register on-chain (Simulated for now)
        tracing::info!(
            "Registering collateral on Soroban: contract={}, id={}, value={}",
            self.contract_id,
            request.collateral_id,
            request.face_value
        );
        // In a real implementation, we would call the Soroban contract here.
        // let tx_hash = soroban_client.invoke(...).await?;
        let tx_hash = format!("tx_{}", Uuid::new_v4().simple());

        // 3. Store in DB
        let collateral = sqlx::query_as::<_, Collateral>(
            r#"
            INSERT INTO collateral (
                collateral_id, owner_id, face_value, expiry_ts, metadata_hash, 
                status, registered_at, locked, tx_hash
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
        )
        .bind(&request.collateral_id)
        .bind(request.owner_id)
        .bind(request.face_value)
        .bind(request.expiry_ts)
        .bind(&request.metadata_hash)
        .bind(CollateralStatus::Active)
        .bind(Utc::now())
        .bind(false)
        .bind(Some(tx_hash))
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(collateral)
    }

    pub async fn get_collateral(&self, id: Uuid) -> Result<Collateral, ApiError> {
        let collateral = sqlx::query_as::<_, Collateral>(
            "SELECT * FROM collateral WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?
        .ok_or(ApiError::NotFound("Collateral not found".to_string()))?;

        Ok(collateral)
    }

    pub async fn get_collateral_by_id_string(&self, id: &str) -> Result<Collateral, ApiError> {
        let collateral = sqlx::query_as::<_, Collateral>(
            "SELECT * FROM collateral WHERE collateral_id = $1",
        )
        .bind(id)
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?
        .ok_or(ApiError::NotFound("Collateral not found".to_string()))?;

        Ok(collateral)
    }

    pub async fn get_collateral_by_metadata(&self, hash: &str) -> Result<Collateral, ApiError> {
        let collateral = sqlx::query_as::<_, Collateral>(
            "SELECT * FROM collateral WHERE metadata_hash = $1",
        )
        .bind(hash)
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?
        .ok_or(ApiError::NotFound("Collateral not found".to_string()))?;

        Ok(collateral)
    }

    pub async fn list_collateral(
        &self,
        filter: CollateralFilter,
    ) -> Result<PaginatedResponse<Collateral>, ApiError> {
        let page = filter.page.unwrap_or(1).max(1);
        let limit = filter.limit.unwrap_or(20).max(1).min(100);
        let offset = (page - 1) * limit;

        let mut query_builder = sqlx::QueryBuilder::new("SELECT * FROM collateral WHERE 1=1");
        let mut count_builder = sqlx::QueryBuilder::new("SELECT COUNT(*) FROM collateral WHERE 1=1");

        if let Some(owner_id) = filter.owner_id {
            query_builder.push(" AND owner_id = ");
            query_builder.push_bind(owner_id);
            count_builder.push(" AND owner_id = ");
            count_builder.push_bind(owner_id);
        }

        if let Some(status) = filter.status {
            query_builder.push(" AND status = ");
            query_builder.push_bind(status);
            count_builder.push(" AND status = ");
            count_builder.push_bind(status);
        }

        let total_count: i64 = count_builder
            .build_query_scalar()
            .fetch_one(&self.db_pool)
            .await
            .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        query_builder.push(" ORDER BY created_at DESC LIMIT ");
        query_builder.push_bind(limit);
        query_builder.push(" OFFSET ");
        query_builder.push_bind(offset);

        let items = query_builder
            .build_query_as::<Collateral>()
            .fetch_all(&self.db_pool)
            .await
            .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(PaginatedResponse {
            data: items,
            total: total_count,
            page: page as i32,
            limit: limit as i32,
        })
    }

    pub async fn update_lock_status(&self, collateral_id: &str, locked: bool) -> Result<(), ApiError> {
        let status = if locked { CollateralStatus::Locked } else { CollateralStatus::Active };
        
        sqlx::query(
            "UPDATE collateral SET locked = $1, status = $2 WHERE collateral_id = $3"
        )
        .bind(locked)
        .bind(status)
        .bind(collateral_id)
        .execute(&self.db_pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(())
    }
}

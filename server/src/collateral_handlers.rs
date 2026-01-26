use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::collateral::{CreateCollateralRequest, CreateCollateralResponse, ListCollateralQuery};
use crate::collateral_service::CollateralService;

/// Create new collateral
pub async fn create_collateral(
    State(collateral_service): State<Arc<CollateralService>>,
    Json(payload): Json<CreateCollateralRequest>,
) -> Result<Json<CreateCollateralResponse>, (StatusCode, String)> {
    // Validate request
    if let Err(e) = payload.validate() {
        return Err((StatusCode::BAD_REQUEST, format!("Validation error: {}", e)));
    }

    match collateral_service.create_collateral(payload).await {
        Ok(collateral) => Ok(Json(CreateCollateralResponse {
            id: collateral.id,
            token_id: collateral.token_id,
            status: collateral.status,
            tx_hash: collateral.tx_hash,
        })),
        Err(e) => {
            tracing::error!("Failed to create collateral: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to create collateral".to_string()))
        }
    }
}

/// Get collateral by ID
pub async fn get_collateral(
    State(collateral_service): State<Arc<CollateralService>>,
    Path(id): Path<Uuid>,
) -> Result<Json<crate::collateral::Collateral>, (StatusCode, String)> {
    match collateral_service.get_collateral(id).await {
        Ok(Some(collateral)) => Ok(Json(collateral)),
        Ok(None) => Err((StatusCode::NOT_FOUND, "Collateral not found".to_string())),
        Err(e) => {
            tracing::error!("Failed to fetch collateral: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch collateral".to_string()))
        }
    }
}

/// List collateral
pub async fn list_collateral(
    State(collateral_service): State<Arc<CollateralService>>,
    Query(query): Query<ListCollateralQuery>,
) -> Result<Json<Vec<crate::collateral::Collateral>>, (StatusCode, String)> {
    match collateral_service.list_collateral(query).await {
        Ok(collaterals) => Ok(Json(collaterals)),
        Err(e) => {
            tracing::error!("Failed to list collateral: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to list collateral".to_string()))
        }
    }
}

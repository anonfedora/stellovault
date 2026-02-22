use axum::{
    extract::{Path, Query, State},
    Json,
};
use uuid::Uuid;
use std::sync::Arc;

use crate::collateral::{CollateralFilter, CreateCollateralRequest, CollateralService};
use crate::models::{ApiResponse, Collateral, PaginatedResponse};
use crate::error::ApiError;

pub async fn create_collateral(
    State(service): State<Arc<CollateralService>>,
    Json(request): Json<CreateCollateralRequest>,
) -> Result<Json<ApiResponse<Collateral>>, ApiError> {
    let collateral = service.create_collateral(request).await?;
    
    Ok(Json(ApiResponse {
        success: true,
        data: Some(collateral),
        error: None,
    }))
}

pub async fn get_collateral(
    State(service): State<Arc<CollateralService>>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<Collateral>>, ApiError> {
    let collateral = service.get_collateral(id).await?;
    
    Ok(Json(ApiResponse {
        success: true,
        data: Some(collateral),
        error: None,
    }))
}

pub async fn get_collateral_by_metadata(
    State(service): State<Arc<CollateralService>>,
    Path(hash): Path<String>,
) -> Result<Json<ApiResponse<Collateral>>, ApiError> {
    let collateral = service.get_collateral_by_metadata(&hash).await?;
    
    Ok(Json(ApiResponse {
        success: true,
        data: Some(collateral),
        error: None,
    }))
}

pub async fn list_collateral(
    State(service): State<Arc<CollateralService>>,
    Query(filter): Query<CollateralFilter>,
) -> Result<Json<ApiResponse<PaginatedResponse<Collateral>>>, ApiError> {
    let result = service.list_collateral(filter).await?;
    
    Ok(Json(ApiResponse {
        success: true,
        data: Some(result),
        error: None,
    }))
}

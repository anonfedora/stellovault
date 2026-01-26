use axum::{
    extract::{Path, Query, State},
    Json,
};
use uuid::Uuid;
use crate::models::{
    ApiResponse, Collateral, CreateCollateralRequest, CreateCollateralResponse, ListCollateralQuery,
};
use crate::state::AppState;
use validator::Validate;

pub async fn create_collateral(
    State(state): State<AppState>,
    Json(payload): Json<CreateCollateralRequest>,
) -> Json<ApiResponse<CreateCollateralResponse>> {
    // Validate payload
    if let Err(e) = payload.validate() {
        return Json(ApiResponse {
            success: false,
            data: None,
            error: Some(format!("Validation error: {}", e)),
        });
    }

    match state.collateral_service.create_collateral(payload).await {
        Ok(collateral) => Json(ApiResponse {
            success: true,
            data: Some(CreateCollateralResponse {
                id: collateral.id,
                token_id: collateral.token_id,
                status: collateral.status,
                tx_hash: collateral.tx_hash,
            }),
            error: None,
        }),
        Err(e) => Json(ApiResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        }),
    }
}

pub async fn list_collateral(
    State(state): State<AppState>,
    Query(query): Query<ListCollateralQuery>,
) -> Json<ApiResponse<Vec<Collateral>>> {
    match state.collateral_service.list_collateral(query).await {
        Ok(collaterals) => Json(ApiResponse {
            success: true,
            data: Some(collaterals),
            error: None,
        }),
        Err(e) => Json(ApiResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        }),
    }
}

pub async fn get_collateral(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Json<ApiResponse<Collateral>> {
    match state.collateral_service.get_collateral(id).await {
        Ok(Some(collateral)) => Json(ApiResponse {
            success: true,
            data: Some(collateral),
            error: None,
        }),
        Ok(None) => Json(ApiResponse {
            success: false,
            data: None,
            error: Some("Collateral not found".to_string()),
        }),
        Err(e) => Json(ApiResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        }),
    }
}

pub async fn get_collateral_by_metadata(
    State(state): State<AppState>,
    Path(hash): Path<String>,
) -> Json<ApiResponse<Collateral>> {
    match state.collateral_service.get_collateral_by_metadata(&hash).await {
        Ok(Some(collateral)) => Json(ApiResponse {
            success: true,
            data: Some(collateral),
            error: None,
        }),
        Ok(None) => Json(ApiResponse {
            success: false,
            data: None,
            error: Some("Collateral not found".to_string()),
        }),
        Err(e) => Json(ApiResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        }),
    }
}

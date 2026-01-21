//! API handlers for StelloVault backend

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde_json::json;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::escrow::{CreateEscrowRequest, CreateEscrowResponse, Escrow, ListEscrowsQuery};
use crate::models::{ApiResponse, User};


// Placeholder handlers - to be implemented

pub async fn get_user(Path(_user_id): Path<String>) -> Json<ApiResponse<User>> {
    // TODO: Implement user retrieval logic
    Json(ApiResponse {
        success: false,
        data: None,
        error: Some("Not implemented yet".to_string()),
    })
}

pub async fn create_user() -> Json<ApiResponse<User>> {
    // TODO: Implement user creation logic
    Json(ApiResponse {
        success: false,
        data: None,
        error: Some("Not implemented yet".to_string()),
    })
}

pub async fn get_analytics() -> Json<ApiResponse<serde_json::Value>> {
    // TODO: Implement analytics logic
    Json(ApiResponse {
        success: true,
        data: Some(json!({
            "total_trades": 0,
            "active_escrows": 0,
            "total_volume": 0
        })),
        error: None,
    })
}

// ===== Escrow Handlers =====

/// Create a new escrow
pub async fn create_escrow(
    State(app_state): State<AppState>,
    Json(request): Json<CreateEscrowRequest>,
) -> Result<Json<ApiResponse<CreateEscrowResponse>>, (StatusCode, Json<ApiResponse<CreateEscrowResponse>>)> {
    // Validate request
    if let Err(e) = request.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                success: false,
                data: None,
                error: Some(format!("Validation error: {}", e)),
            }),
        ));
    }

    // Create escrow via service
    match app_state.escrow_service.create_escrow(request).await {
        Ok(response) => {
            // Broadcast creation event
            app_state.ws_state
                .broadcast_event(crate::escrow::EscrowEvent::Created {
                    escrow_id: response.escrow_id,
                    buyer_id: response.id,
                    seller_id: response.id,
                })
                .await;

            Ok(Json(ApiResponse {
                success: true,
                data: Some(response),
                error: None,
            }))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse {
                success: false,
                data: None,
                error: Some(format!("Failed to create escrow: {}", e)),
            }),
        )),
    }
}

/// Get a single escrow by ID
pub async fn get_escrow(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<Escrow>>, (StatusCode, Json<ApiResponse<Escrow>>)> {
    match app_state.escrow_service.get_escrow(&id).await {
        Ok(Some(escrow)) => Ok(Json(ApiResponse {
            success: true,
            data: Some(escrow),
            error: None,
        })),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse {
                success: false,
                data: None,
                error: Some("Escrow not found".to_string()),
            }),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse {
                success: false,
                data: None,
                error: Some(format!("Database error: {}", e)),
            }),
        )),
    }
}

/// List escrows with filtering and pagination
pub async fn list_escrows(
    State(app_state): State<AppState>,
    Query(query): Query<ListEscrowsQuery>,
) -> Result<Json<ApiResponse<Vec<Escrow>>>, (StatusCode, Json<ApiResponse<Vec<Escrow>>>)> {
    match app_state.escrow_service.list_escrows(query).await {
        Ok(escrows) => Ok(Json(ApiResponse {
            success: true,
            data: Some(escrows),
            error: None,
        })),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse {
                success: false,
                data: None,
                error: Some(format!("Failed to list escrows: {}", e)),
            }),
        )),
    }
}

/// Webhook endpoint for escrow status updates
pub async fn webhook_escrow_update(
    State(app_state): State<AppState>,
    Json(payload): Json<crate::escrow::WebhookPayload>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Process webhook payload
    if let Some(status) = payload.status {
        let event = crate::escrow::EscrowEvent::StatusUpdated {
            escrow_id: payload.escrow_id,
            status,
        };

        if let Err(e) = app_state.escrow_service.process_escrow_event(event.clone()).await {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse {
                    success: false,
                    data: None,
                    error: Some(format!("Failed to process event: {}", e)),
                }),
            ));
        }

        // Broadcast update
        app_state.ws_state.broadcast_event(event).await;
    }

    Ok(Json(ApiResponse {
        success: true,
        data: Some(()),
        error: None,
    }))
}
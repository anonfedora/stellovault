//! API handlers for StelloVault backend

use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::collateral::CreateCollateralRequest;
use crate::escrow::{CreateEscrowRequest, CreateEscrowResponse, Escrow, ListEscrowsQuery};
use crate::models::{ApiResponse, Collateral, Oracle, OracleConfirmation, OracleConfirmationRequest, OracleMetrics, OracleRegistrationRequest, User};


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

    // Capture IDs before moving request
    let buyer_id = request.buyer_id;
    let seller_id = request.seller_id;

    // Create escrow via service
    match app_state.escrow_service.create_escrow(request).await {
        Ok(response) => {
            // Broadcast creation event
            app_state.ws_state
                .broadcast_event(crate::escrow::EscrowEvent::Created {
                    escrow_id: response.escrow_id,
                    buyer_id,
                    seller_id,
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
    headers: HeaderMap,
    Json(payload): Json<crate::escrow::WebhookPayload>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Authenticate webhook
    match &app_state.webhook_secret {
        Some(secret) if !secret.is_empty() => {
            let auth_header = headers.get("X-Webhook-Secret")
                .and_then(|h| h.to_str().ok())
                .unwrap_or_default();

            if auth_header != secret {
                return Err((
                    StatusCode::UNAUTHORIZED,
                    Json(ApiResponse {
                        success: false,
                        data: None,
                        error: Some("Unauthorized webhook request".to_string()),
                    }),
                ));
            }
        }
        _ => {
            // Fail-closed: if secret is not configured or empty, reject all requests
            tracing::error!("Webhook secret not configured - rejecting request");
            return Err((
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ApiResponse {
                    success: false,
                    data: None,
                    error: Some("Webhook endpoint is not configured".to_string()),
                }),
            ));
        }
    }
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

// ===== Collateral Handlers =====

pub async fn create_collateral(
    State(app_state): State<AppState>,
    Json(req): Json<CreateCollateralRequest>,
) -> Json<ApiResponse<Collateral>> {
    match app_state.collateral_service.create_collateral(req).await {
        Ok(collateral) => Json(ApiResponse {
            success: true,
            data: Some(collateral),
            error: None,
        }),
        Err(e) => Json(ApiResponse {
            success: false,
            data: None,
            error: Some(format!("Failed to create collateral: {}", e)),
        }),
    }
}

pub async fn get_collateral(
    State(app_state): State<AppState>,
    Path(collateral_id): Path<String>,
) -> Json<ApiResponse<Collateral>> {
    match app_state.collateral_service.get_collateral(&collateral_id).await {
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
            error: Some(format!("Database error: {}", e)),
        }),
    }
}

pub async fn get_collateral_by_metadata(
    State(app_state): State<AppState>,
    Path(metadata_hash): Path<String>,
) -> Json<ApiResponse<Collateral>> {
    match app_state.collateral_service.get_collateral_by_metadata(&metadata_hash).await {
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
            error: Some(format!("Database error: {}", e)),
        }),
    }
}

#[derive(Deserialize)]
pub struct ListCollateralQuery {
    pub owner_id: Option<Uuid>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list_collateral(
    State(app_state): State<AppState>,
    Query(query): Query<ListCollateralQuery>,
) -> Json<ApiResponse<Vec<Collateral>>> {
    let limit = query.limit.unwrap_or(50).min(100); // Max 100 items
    let offset = query.offset.unwrap_or(0);

    match query.owner_id {
        Some(owner_id) => {
            match app_state.collateral_service.list_user_collateral(owner_id, limit, offset).await {
                Ok(collateral) => Json(ApiResponse {
                    success: true,
                    data: Some(collateral),
                    error: None,
                }),
                Err(e) => Json(ApiResponse {
                    success: false,
                    data: None,
                    error: Some(format!("Database error: {}", e)),
                }),
            }
        }
        None => Json(ApiResponse {
            success: false,
            data: None,
            error: Some("owner_id parameter is required".to_string()),
        }),
    }
}

// ===== ORACLE HANDLERS =====

// Register a new oracle
pub async fn register_oracle(
    State(app_state): State<AppState>,
    Json(request): Json<OracleRegistrationRequest>,
) -> Json<ApiResponse<Oracle>> {
    match app_state.oracle_service.register_oracle(request, None).await {
        Ok(oracle) => Json(ApiResponse {
            success: true,
            data: Some(oracle),
            error: None,
        }),
        Err(e) => Json(ApiResponse {
            success: false,
            data: None,
            error: Some(format!("Failed to register oracle: {}", e)),
        }),
    }
}

// Get oracle by address
pub async fn get_oracle(
    State(app_state): State<AppState>,
    Path(address): Path<String>,
) -> Json<ApiResponse<Option<Oracle>>> {
    match app_state.oracle_service.get_oracle_by_address(&address).await {
        Ok(oracle) => Json(ApiResponse {
            success: true,
            data: Some(oracle),
            error: None,
        }),
        Err(e) => Json(ApiResponse {
            success: false,
            data: None,
            error: Some(format!("Database error: {}", e)),
        }),
    }
}

// List all active oracles
pub async fn list_oracles(
    State(app_state): State<AppState>,
) -> Json<ApiResponse<Vec<Oracle>>> {
    match app_state.oracle_service.get_active_oracles().await {
        Ok(oracles) => Json(ApiResponse {
            success: true,
            data: Some(oracles),
            error: None,
        }),
        Err(e) => Json(ApiResponse {
            success: false,
            data: None,
            error: Some(format!("Database error: {}", e)),
        }),
    }
}

// Deactivate an oracle
pub async fn deactivate_oracle(
    State(app_state): State<AppState>,
    Path(address): Path<String>,
) -> Json<ApiResponse<String>> {
    match app_state.oracle_service.deactivate_oracle(&address).await {
        Ok(_) => Json(ApiResponse {
            success: true,
            data: Some(format!("Oracle {} deactivated", address)),
            error: None,
        }),
        Err(e) => Json(ApiResponse {
            success: false,
            data: None,
            error: Some(format!("Failed to deactivate oracle: {}", e)),
        }),
    }
}

// Submit oracle confirmation
pub async fn submit_confirmation(
    State(app_state): State<AppState>,
    Json(request): Json<OracleConfirmationRequest>,
) -> Json<ApiResponse<OracleConfirmation>> {
    // Get oracle from request context (TODO: implement proper authentication)
    // For now, we'll use a placeholder oracle address
    let oracle_address = "oracle_placeholder_address";

    match app_state.oracle_service.submit_confirmation(request, oracle_address).await {
        Ok(confirmation) => Json(ApiResponse {
            success: true,
            data: Some(confirmation),
            error: None,
        }),
        Err(e) => Json(ApiResponse {
            success: false,
            data: None,
            error: Some(format!("Failed to submit confirmation: {}", e)),
        }),
    }
}

// Get confirmations for an escrow
pub async fn get_confirmations(
    State(app_state): State<AppState>,
    Path(escrow_id): Path<String>,
) -> Json<ApiResponse<Vec<OracleConfirmation>>> {
    match app_state.oracle_service.get_confirmations_for_escrow(&escrow_id).await {
        Ok(confirmations) => Json(ApiResponse {
            success: true,
            data: Some(confirmations),
            error: None,
        }),
        Err(e) => Json(ApiResponse {
            success: false,
            data: None,
            error: Some(format!("Database error: {}", e)),
        }),
    }
}

// Get oracle metrics
pub async fn get_oracle_metrics(
    State(app_state): State<AppState>,
) -> Json<ApiResponse<OracleMetrics>> {
    match app_state.oracle_service.get_oracle_metrics().await {
        Ok(metrics) => Json(ApiResponse {
            success: true,
            data: Some(metrics),
            error: None,
        }),
        Err(e) => Json(ApiResponse {
            success: false,
            data: None,
            error: Some(format!("Database error: {}", e)),
        }),
    }
}
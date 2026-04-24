//! Escrow-related API handlers

use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use serde_json::json;
use uuid::Uuid;

use crate::escrow::{
    CreateEscrowRequest, CreateEscrowResponse, DisputeEscrowRequest, Escrow, EscrowEvent,
    EscrowHistoryEntry, FundEscrowRequest, ListEscrowsQuery, ReleaseEscrowRequest,
    UpdateEscrowStatusRequest, WebhookPayload,
};
use crate::websocket::WsEvent;
use crate::loan::{CreateLoanRequest, ListLoansQuery, Loan, Repayment, RepaymentRequest};
use crate::websocket::{WsEvent, EscrowEvent as WsEscrowEvent};
use crate::models::{ApiResponse, User};
use crate::state::AppState;

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
) -> Result<
    Json<ApiResponse<CreateEscrowResponse>>,
    (StatusCode, Json<ApiResponse<CreateEscrowResponse>>),
> {
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
            app_state
                .ws_state
                .broadcast_event(WsEvent::Escrow(WsEscrowEvent::Created {
                    escrow_id: response.escrow_id,
                    data: serde_json::json!({ "buyer_id": buyer_id, "seller_id": seller_id }),
                }))
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
    Json(payload): Json<WebhookPayload>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Authenticate webhook
    match &app_state.webhook_secret {
        Some(secret) if !secret.is_empty() => {
            let auth_header = headers
                .get("X-Webhook-Secret")
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
        let event = EscrowEvent::StatusUpdated {
            escrow_id: payload.escrow_id,
            status,
        };

        if let Err(e) = app_state
            .escrow_service
            .process_escrow_event(event.clone())
            .await
        {
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
        let ws_event = WsEvent::Escrow(WsEscrowEvent::from(event));
        app_state.ws_state.broadcast_event(ws_event).await;
    }

    Ok(Json(ApiResponse {
        success: true,
        data: Some(()),
        error: None,
    }))
}

/// Update escrow status (state machine transition)
pub async fn update_escrow_status(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateEscrowStatusRequest>,
) -> Result<Json<ApiResponse<Escrow>>, (StatusCode, Json<ApiResponse<Escrow>>)> {
    match app_state
        .escrow_service
        .update_status(&id, request.status, request.reason, None)
        .await
    {
        Ok(escrow) => {
            app_state
                .ws_state
                .broadcast_event(WsEvent::Escrow(
                    EscrowEvent::StatusUpdated {
                        escrow_id: escrow.escrow_id,
                        status: escrow.status,
                    }
                    .into(),
                ))
                .await;
            Ok(Json(ApiResponse {
                success: true,
                data: Some(escrow),
                error: None,
            }))
        }
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                success: false,
                data: None,
                error: Some(e.to_string()),
            }),
        )),
    }
}

/// Fund an escrow (transition Pending -> Active)
pub async fn fund_escrow(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<FundEscrowRequest>,
) -> Result<Json<ApiResponse<Escrow>>, (StatusCode, Json<ApiResponse<Escrow>>)> {
    match app_state.escrow_service.fund_escrow(&id, request).await {
        Ok(escrow) => {
            app_state
                .ws_state
                .broadcast_event(WsEvent::Escrow(
                    EscrowEvent::Activated {
                        escrow_id: escrow.escrow_id,
                    }
                    .into(),
                ))
                .await;
            Ok(Json(ApiResponse {
                success: true,
                data: Some(escrow),
                error: None,
            }))
        }
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                success: false,
                data: None,
                error: Some(e.to_string()),
            }),
        )),
    }
}

/// Release funds from an escrow
pub async fn release_escrow(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<ReleaseEscrowRequest>,
) -> Result<Json<ApiResponse<Escrow>>, (StatusCode, Json<ApiResponse<Escrow>>)> {
    match app_state.escrow_service.release_escrow(&id, request).await {
        Ok(escrow) => {
            app_state
                .ws_state
                .broadcast_event(WsEvent::Escrow(
                    EscrowEvent::Released {
                        escrow_id: escrow.escrow_id,
                    }
                    .into(),
                ))
                .await;
            Ok(Json(ApiResponse {
                success: true,
                data: Some(escrow),
                error: None,
            }))
        }
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                success: false,
                data: None,
                error: Some(e.to_string()),
            }),
        )),
    }
}

/// Raise a dispute on an escrow
pub async fn dispute_escrow(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<DisputeEscrowRequest>,
) -> Result<Json<ApiResponse<Escrow>>, (StatusCode, Json<ApiResponse<Escrow>>)> {
    match app_state.escrow_service.dispute_escrow(&id, request).await {
        Ok(escrow) => {
            app_state
                .ws_state
                .broadcast_event(WsEvent::Escrow(
                    EscrowEvent::Disputed {
                        escrow_id: escrow.escrow_id,
                        reason: "Dispute raised via API".to_string(),
                    }
                    .into(),
                ))
                .await;
            Ok(Json(ApiResponse {
                success: true,
                data: Some(escrow),
                error: None,
            }))
        }
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                success: false,
                data: None,
                error: Some(e.to_string()),
            }),
        )),
    }
}

/// Get escrow transaction history
pub async fn get_escrow_history(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<
    Json<ApiResponse<Vec<EscrowHistoryEntry>>>,
    (StatusCode, Json<ApiResponse<Vec<EscrowHistoryEntry>>>),
> {
    match app_state.escrow_service.get_escrow_history(&id).await {
        Ok(history) => Ok(Json(ApiResponse {
            success: true,
            data: Some(history),
            error: None,
        })),
        Err(e) => {
            let status = if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            Err((
                status,
                Json(ApiResponse {
                    success: false,
                    data: None,
                    error: Some(e.to_string()),
                }),
            ))
        }
    }
}

// ===== Collateral Handlers =====
// Moved to src/handlers/collateral.rs

// ===== Loan Handlers =====

/// Get list of loans
pub async fn list_loans(
    State(app_state): State<AppState>,
    Query(query): Query<ListLoansQuery>,
) -> Json<ApiResponse<Vec<Loan>>> {
    match app_state
        .loan_service
        .list_loans(query.borrower_id, query.lender_id, query.status)
        .await
    {
        Ok(loans) => Json(ApiResponse {
            success: true,
            data: Some(loans),
            error: None,
        }),
        Err(e) => Json(ApiResponse {
            success: false,
            data: None,
            error: Some(format!("Failed to list loans: {}", e)),
        }),
    }
}

/// Get a single loan by ID
pub async fn get_loan(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Json<ApiResponse<Loan>> {
    match app_state.loan_service.get_loan(&id).await {
        Ok(Some(loan)) => Json(ApiResponse {
            success: true,
            data: Some(loan),
            error: None,
        }),
        Ok(None) => Json(ApiResponse {
            success: false,
            data: None,
            error: Some("Loan not found".to_string()),
        }),
        Err(e) => Json(ApiResponse {
            success: false,
            data: None,
            error: Some(format!("Database error: {}", e)),
        }),
    }
}

/// Issue a new loan
pub async fn create_loan(
    State(app_state): State<AppState>,
    Json(req): Json<CreateLoanRequest>,
) -> Json<ApiResponse<Loan>> {
    match app_state.loan_service.issue_loan(req).await {
        Ok(loan) => Json(ApiResponse {
            success: true,
            data: Some(loan),
            error: None,
        }),
        Err(e) => Json(ApiResponse {
            success: false,
            data: None,
            error: Some(format!("Failed to issue loan: {}", e)),
        }),
    }
}

/// Record a repayment
pub async fn record_repayment(
    State(app_state): State<AppState>,
    Json(req): Json<RepaymentRequest>,
) -> Json<ApiResponse<Repayment>> {
    match app_state.loan_service.record_repayment(req).await {
        Ok(repayment) => Json(ApiResponse {
            success: true,
            data: Some(repayment),
            error: None,
        }),
        Err(e) => Json(ApiResponse {
            success: false,
            data: None,
            error: Some(format!("Failed to record repayment: {}", e)),
        }),
    }
}

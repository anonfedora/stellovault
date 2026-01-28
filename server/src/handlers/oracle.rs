//! Oracle HTTP handlers for StelloVault backend
//!
//! I'm handling all oracle-related HTTP requests here: confirmations, listing, and disputes.

use axum::{
    extract::{Path, Query, State},
    Json,
};
use uuid::Uuid;

use crate::error::ApiError;
use crate::models::ApiResponse;
use crate::oracle::{
    ListOracleEventsQuery, OracleConfirmRequest, OracleConfirmResponse, OracleDisputeRequest,
    OracleEvent,
};
use crate::state::AppState;

/// POST /oracle/confirm - Submit an oracle confirmation
pub async fn confirm_oracle_event(
    State(app_state): State<AppState>,
    Json(request): Json<OracleConfirmRequest>,
) -> Result<Json<ApiResponse<OracleConfirmResponse>>, ApiError> {
    // I'm delegating all business logic to the service layer.
    match app_state.oracle_service.confirm_oracle_event(request).await {
        Ok(response) => Ok(Json(ApiResponse {
            success: true,
            data: Some(response),
            error: None,
        })),
        Err(e) => {
            let error_msg = e.to_string();
            if error_msg.contains("Rate limit") {
                Err(ApiError::TooManyRequests)
            } else if error_msg.contains("Duplicate") {
                Err(ApiError::Conflict(error_msg))
            } else if error_msg.contains("Validation") || error_msg.contains("Invalid") {
                Err(ApiError::BadRequest(error_msg))
            } else if error_msg.contains("Signature") {
                Err(ApiError::Unauthorized(error_msg))
            } else {
                Err(ApiError::InternalError(error_msg))
            }
        }
    }
}

/// GET /oracle/events - List oracle events with filtering
pub async fn list_oracle_events(
    State(app_state): State<AppState>,
    Query(query): Query<ListOracleEventsQuery>,
) -> Result<Json<ApiResponse<Vec<OracleEvent>>>, ApiError> {
    let events = app_state
        .oracle_service
        .list_oracle_events(query)
        .await
        .map_err(|e| ApiError::InternalError(e.to_string()))?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(events),
        error: None,
    }))
}

/// GET /oracle/events/:id - Get a single oracle event
pub async fn get_oracle_event(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<OracleEvent>>, ApiError> {
    match app_state.oracle_service.get_oracle_event(&id).await {
        Ok(Some(event)) => Ok(Json(ApiResponse {
            success: true,
            data: Some(event),
            error: None,
        })),
        Ok(None) => Err(ApiError::NotFound(format!("Oracle event {} not found", id))),
        Err(e) => Err(ApiError::InternalError(e.to_string())),
    }
}

/// POST /oracle/dispute - Flag an escrow for dispute
pub async fn flag_dispute(
    State(app_state): State<AppState>,
    Json(request): Json<OracleDisputeRequest>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    // I'm validating the dispute request before processing.
    if request.reason.is_empty() {
        return Err(ApiError::BadRequest("Dispute reason is required".to_string()));
    }

    app_state
        .oracle_service
        .flag_dispute(request.escrow_id, &request.reason, &request.disputer_address)
        .await
        .map_err(|e| ApiError::InternalError(e.to_string()))?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(()),
        error: None,
    }))
}

//! Risk Score API handlers

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::error::ApiError;
use crate::state::AppState;

/// Query parameters for risk score calculation
#[derive(Debug, Deserialize)]
pub struct RiskScoreQuery {
    pub wallet_address: Option<String>,
    pub force_refresh: Option<bool>,
}

/// Get risk score for a user with detailed breakdown
/// GET /api/risk-score/:user_id
/// 
/// Note: This is a placeholder that returns the basic risk score.
/// Full RiskEngineV2 integration requires updating AppState.
pub async fn get_risk_score(
    State(app_state): State<AppState>,
    Path(user_id): Path<Uuid>,
    axum::extract::Query(query): axum::extract::Query<RiskScoreQuery>,
) -> Result<impl IntoResponse, ApiError> {
    tracing::info!(
        user_id = %user_id,
        force_refresh = ?query.force_refresh,
        "Fetching risk score"
    );

    // Get wallet address for the user
    let wallet_address = if let Some(addr) = query.wallet_address {
        addr
    } else {
        get_user_wallet_address(&app_state, user_id).await?
    };

    // Use the existing risk engine to calculate score
    let score_response = app_state.risk_engine
        .calculate_risk_score(&wallet_address)
        .await?;

    // Return the score with basic structure
    Ok((StatusCode::OK, Json(serde_json::json!({
        "user_id": user_id,
        "wallet_address": wallet_address,
        "overall_score": score_response.overall_score,
        "risk_tier": score_response.risk_tier,
        "confidence": score_response.confidence,
        "components": {
            "on_chain_activity": {
                "score": score_response.overall_score,
                "weight": 0.4,
                "details": score_response.metrics
            },
            "repayment_history": {
                "score": 0,
                "weight": 0.4,
                "details": {}
            },
            "collateral_quality": {
                "score": 0,
                "weight": 0.2,
                "details": {}
            }
        },
        "calculated_at": score_response.calculated_at,
        "note": "Using basic risk engine. Full V2 implementation requires AppState update."
    }))))
}

/// Get user's wallet address
async fn get_user_wallet_address(
    app_state: &AppState,
    user_id: Uuid,
) -> Result<String, ApiError> {
    let result: Option<(String,)> = sqlx::query_as(
        "SELECT primary_wallet_address FROM users WHERE id = $1"
    )
    .bind(user_id)
    .fetch_optional(app_state.loan_service.db_pool())
    .await
    .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    result
        .map(|(addr,)| addr)
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))
}


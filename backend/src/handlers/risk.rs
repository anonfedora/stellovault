//! Risk scoring API handlers

use axum::{
    extract::{Path, Query, State},
    Json,
};
use std::sync::Arc;

use crate::error::ApiError;
use crate::models::ApiResponse;
use crate::services::risk_engine::{
    HistoricalScore, HistoricalScoreQuery, RiskEngine, RiskScoreResponse, SimulationResult,
    SimulationScenario,
};

/// GET /risk/:wallet - Get risk score for a wallet
pub async fn get_risk_score(
    State(risk_engine): State<Arc<RiskEngine>>,
    Path(wallet): Path<String>,
) -> Result<Json<ApiResponse<RiskScoreResponse>>, ApiError> {
    let score = risk_engine.calculate_risk_score(&wallet).await?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(score),
        error: None,
    }))
}

/// GET /risk/:wallet/history - Get historical risk scores for backtesting
pub async fn get_risk_history(
    State(risk_engine): State<Arc<RiskEngine>>,
    Path(wallet): Path<String>,
    Query(query): Query<HistoricalScoreQuery>,
) -> Result<Json<ApiResponse<Vec<HistoricalScore>>>, ApiError> {
    let start_date = query
        .start_date
        .unwrap_or_else(|| chrono::Utc::now() - chrono::Duration::days(365));
    let end_date = query.end_date.unwrap_or_else(chrono::Utc::now);

    let history = risk_engine
        .get_historical_scores(&wallet, start_date, end_date)
        .await?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(history),
        error: None,
    }))
}

/// POST /risk/:wallet/simulate - Simulate score impact
pub async fn simulate_risk_score(
    State(risk_engine): State<Arc<RiskEngine>>,
    Path(wallet): Path<String>,
    Json(scenario): Json<SimulationScenario>,
) -> Result<Json<ApiResponse<SimulationResult>>, ApiError> {
    let result = risk_engine.simulate_score_impact(&wallet, scenario).await?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(result),
        error: None,
    }))
}

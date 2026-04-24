//! Governance API handlers

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::governance_service::GovernanceService;
use crate::middleware::AuthenticatedUser;
use crate::models::{
    ApiResponse, GovernanceMetrics, GovernanceProposal, GovernanceVote, ProposalCreationRequest,
    ProposalStatus, VoteSubmissionRequest,
};

#[derive(Deserialize)]
pub struct ListProposalsQuery {
    pub status: Option<String>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

/// POST /api/v1/governance/proposals — create a new proposal
pub async fn create_proposal(
    State(svc): State<Arc<GovernanceService>>,
    auth_user: AuthenticatedUser,
    Json(req): Json<ProposalCreationRequest>,
) -> Result<Json<ApiResponse<GovernanceProposal>>, (StatusCode, Json<ApiResponse<()>>)> {
    if req.title.trim().is_empty() || req.description.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                success: false,
                data: None,
                error: Some("title and description are required".to_string()),
            }),
        ));
    }

    match svc.create_proposal(req, &auth_user.wallet_address).await {
        Ok(proposal) => Ok(Json(ApiResponse {
            success: true,
            data: Some(proposal),
            error: None,
        })),
        Err(e) => {
            tracing::error!("create_proposal failed: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse {
                    success: false,
                    data: None,
                    error: Some(e.to_string()),
                }),
            ))
        }
    }
}

/// GET /api/v1/governance/proposals — list proposals
pub async fn list_proposals(
    State(svc): State<Arc<GovernanceService>>,
    Query(q): Query<ListProposalsQuery>,
) -> Result<Json<ApiResponse<Vec<GovernanceProposal>>>, (StatusCode, Json<ApiResponse<()>>)> {
    let status = q.status.as_deref().and_then(parse_proposal_status);

    match svc.get_proposals(status, q.limit, q.offset).await {
        Ok(proposals) => Ok(Json(ApiResponse {
            success: true,
            data: Some(proposals),
            error: None,
        })),
        Err(e) => {
            tracing::error!("list_proposals failed: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse {
                    success: false,
                    data: None,
                    error: Some(e.to_string()),
                }),
            ))
        }
    }
}

/// GET /api/v1/governance/proposals/:id — get proposal details
pub async fn get_proposal(
    State(svc): State<Arc<GovernanceService>>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<GovernanceProposal>>, (StatusCode, Json<ApiResponse<()>>)> {
    match svc.get_proposal(&id).await {
        Ok(Some(proposal)) => Ok(Json(ApiResponse {
            success: true,
            data: Some(proposal),
            error: None,
        })),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse {
                success: false,
                data: None,
                error: Some(format!("Proposal '{}' not found", id)),
            }),
        )),
        Err(e) => {
            tracing::error!("get_proposal failed: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse {
                    success: false,
                    data: None,
                    error: Some(e.to_string()),
                }),
            ))
        }
    }
}

/// POST /api/v1/governance/proposals/:id/vote — cast a vote
pub async fn cast_vote(
    State(svc): State<Arc<GovernanceService>>,
    Path(id): Path<String>,
    auth_user: AuthenticatedUser,
    Json(mut req): Json<VoteSubmissionRequest>,
) -> Result<Json<ApiResponse<GovernanceVote>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Ensure the proposal_id in the body matches the path param
    req.proposal_id = id;
    req.voter_address = auth_user.wallet_address;

    match svc.submit_vote(req).await {
        Ok(vote) => Ok(Json(ApiResponse {
            success: true,
            data: Some(vote),
            error: None,
        })),
        Err(sqlx::Error::RowNotFound) => Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse {
                success: false,
                data: None,
                error: Some("Proposal not found".to_string()),
            }),
        )),
        Err(sqlx::Error::Protocol(msg)) => Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(ApiResponse {
                success: false,
                data: None,
                error: Some(msg),
            }),
        )),
        Err(e) => {
            tracing::error!("cast_vote failed: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse {
                    success: false,
                    data: None,
                    error: Some(e.to_string()),
                }),
            ))
        }
    }
}

/// Voting power response
#[derive(serde::Serialize)]
pub struct VotingPowerResponse {
    pub address: String,
    pub voting_power: i64,
    pub method: &'static str,
}

/// GET /api/v1/governance/voting-power/:address — get quadratic voting power
pub async fn get_voting_power(
    State(svc): State<Arc<GovernanceService>>,
    Path(address): Path<String>,
) -> Result<Json<ApiResponse<VotingPowerResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    match svc.calculate_voting_power(&address, None).await {
        Ok(power) => Ok(Json(ApiResponse {
            success: true,
            data: Some(VotingPowerResponse {
                address,
                voting_power: power,
                method: "quadratic",
            }),
            error: None,
        })),
        Err(e) => {
            tracing::error!("get_voting_power failed: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse {
                    success: false,
                    data: None,
                    error: Some(e.to_string()),
                }),
            ))
        }
    }
}

/// POST /api/v1/governance/execute/:id — execute a passed proposal
pub async fn execute_proposal(
    State(svc): State<Arc<GovernanceService>>,
    Path(id): Path<String>,
    auth_user: AuthenticatedUser,
) -> Result<Json<ApiResponse<GovernanceProposal>>, (StatusCode, Json<ApiResponse<()>>)> {
    match svc.execute_proposal(&id, &auth_user.wallet_address).await {
        Ok(proposal) => Ok(Json(ApiResponse {
            success: true,
            data: Some(proposal),
            error: None,
        })),
        Err(sqlx::Error::RowNotFound) => Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse {
                success: false,
                data: None,
                error: Some(format!("Proposal '{}' not found", id)),
            }),
        )),
        Err(sqlx::Error::Protocol(msg)) => Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(ApiResponse {
                success: false,
                data: None,
                error: Some(msg),
            }),
        )),
        Err(e) => {
            tracing::error!("execute_proposal failed: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse {
                    success: false,
                    data: None,
                    error: Some(e.to_string()),
                }),
            ))
        }
    }
}

/// GET /api/v1/governance/metrics — governance analytics
pub async fn get_governance_metrics(
    State(svc): State<Arc<GovernanceService>>,
) -> Result<Json<ApiResponse<GovernanceMetrics>>, (StatusCode, Json<ApiResponse<()>>)> {
    match svc.get_governance_metrics().await {
        Ok(metrics) => Ok(Json(ApiResponse {
            success: true,
            data: Some(metrics),
            error: None,
        })),
        Err(e) => {
            tracing::error!("get_governance_metrics failed: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse {
                    success: false,
                    data: None,
                    error: Some(e.to_string()),
                }),
            ))
        }
    }
}

fn parse_proposal_status(s: &str) -> Option<ProposalStatus> {
    match s.to_lowercase().as_str() {
        "pending" => Some(ProposalStatus::Pending),
        "active" => Some(ProposalStatus::Active),
        "succeeded" => Some(ProposalStatus::Succeeded),
        "failed" => Some(ProposalStatus::Failed),
        "executed" => Some(ProposalStatus::Executed),
        "cancelled" => Some(ProposalStatus::Cancelled),
        _ => None,
    }
}

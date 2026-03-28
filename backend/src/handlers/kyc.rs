//! KYC/AML compliance API handlers

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::middleware::AuthenticatedUser;
use crate::models::ApiResponse;
use crate::services::kyc::{
    KycService, KycStatusResponse, KycVerificationRequest, KycVerificationResponse,
};

/// Get KYC status for the authenticated user
pub async fn get_my_kyc_status(
    State(kyc_service): State<KycService>,
    auth_user: AuthenticatedUser,
) -> Result<Json<ApiResponse<KycStatusResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    match kyc_service.get_kyc_status(&auth_user.user_id).await {
        Ok(status) => Ok(Json(ApiResponse {
            success: true,
            data: Some(status),
            error: None,
        })),
        Err(e) => {
            tracing::error!("Failed to get KYC status: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse {
                    success: false,
                    data: None,
                    error: Some("Failed to retrieve KYC status".to_string()),
                }),
            ))
        }
    }
}

/// Get KYC status for a specific user (admin only)
pub async fn get_user_kyc_status(
    State(kyc_service): State<KycService>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<ApiResponse<KycStatusResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    match kyc_service.get_kyc_status(&user_id).await {
        Ok(status) => Ok(Json(ApiResponse {
            success: true,
            data: Some(status),
            error: None,
        })),
        Err(e) => {
            tracing::error!("Failed to get KYC status for user {}: {}", user_id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse {
                    success: false,
                    data: None,
                    error: Some("Failed to retrieve KYC status".to_string()),
                }),
            ))
        }
    }
}

/// Initiate KYC verification for the authenticated user
pub async fn initiate_kyc_verification(
    State(kyc_service): State<KycService>,
    auth_user: AuthenticatedUser,
) -> Result<
    Json<ApiResponse<KycVerificationResponse>>,
    (StatusCode, Json<ApiResponse<()>>),
> {
    match kyc_service.initiate_verification(&auth_user.user_id).await {
        Ok(response) => Ok(Json(ApiResponse {
            success: true,
            data: Some(response),
            error: None,
        })),
        Err(e) => {
            tracing::error!("Failed to initiate KYC verification: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse {
                    success: false,
                    data: None,
                    error: Some("Failed to initiate verification".to_string()),
                }),
            ))
        }
    }
}

/// Mock approve KYC verification (for testing - admin only)
pub async fn mock_approve_kyc(
    State(kyc_service): State<KycService>,
    Path(user_id): Path<Uuid>,
    Json(request): Json<KycVerificationRequest>,
) -> Result<
    Json<ApiResponse<KycVerificationResponse>>,
    (StatusCode, Json<ApiResponse<()>>),
> {
    match kyc_service
        .mock_approve_verification(&user_id, request.provider, request.reference_id)
        .await
    {
        Ok(response) => Ok(Json(ApiResponse {
            success: true,
            data: Some(response),
            error: None,
        })),
        Err(e) => {
            tracing::error!("Failed to approve KYC verification: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse {
                    success: false,
                    data: None,
                    error: Some("Failed to approve verification".to_string()),
                }),
            ))
        }
    }
}

/// Mock reject KYC verification (for testing - admin only)
pub async fn mock_reject_kyc(
    State(kyc_service): State<KycService>,
    Path(user_id): Path<Uuid>,
    Json(reason): Json<Option<String>>,
) -> Result<
    Json<ApiResponse<KycVerificationResponse>>,
    (StatusCode, Json<ApiResponse<()>>),
> {
    match kyc_service.mock_reject_verification(&user_id, reason).await {
        Ok(response) => Ok(Json(ApiResponse {
            success: true,
            data: Some(response),
            error: None,
        })),
        Err(e) => {
            tracing::error!("Failed to reject KYC verification: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse {
                    success: false,
                    data: None,
                    error: Some("Failed to reject verification".to_string()),
                }),
            ))
        }
    }
}

//! Compliance middleware for KYC/AML checks

use axum::{
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::models::ApiResponse;
use crate::services::KycService;
use crate::middleware::AuthenticatedUser;

/// Middleware to enforce KYC compliance for high-value transactions
pub async fn kyc_compliance_middleware(
    State(kyc_service): State<KycService>,
    auth_user: AuthenticatedUser,
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    // Extract transaction amount from request body if present
    // This is a simplified check - in production, you'd parse the body more carefully
    let (parts, body) = request.into_parts();
    
    // Read body to check for amount field
    let bytes = match axum::body::to_bytes(body, usize::MAX).await {
        Ok(b) => b,
        Err(_) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<()> {
                    success: false,
                    data: None,
                    error: Some("Failed to read request body".to_string()),
                }),
            )
                .into_response());
        }
    };

    // Try to parse as JSON to extract amount
    let amount: i64 = if let Ok(json_value) = serde_json::from_slice::<serde_json::Value>(&bytes) {
        json_value
            .get("amount")
            .and_then(|v| v.as_i64())
            .unwrap_or(0)
    } else {
        0
    };

    // Check KYC compliance
    match kyc_service
        .check_kyc_compliance(&auth_user.user_id, amount)
        .await
    {
        Ok(true) => {
            // KYC check passed, reconstruct request and continue
            let request = Request::from_parts(parts, Body::from(bytes));
            Ok(next.run(request).await)
        }
        Ok(false) => {
            // KYC check failed
            Err((
                StatusCode::FORBIDDEN,
                Json(json!({
                    "success": false,
                    "error": "KYC verification required for transactions over $10,000. Please complete identity verification.",
                    "code": "KYC_REQUIRED",
                    "threshold": 1000000 // $10,000 in cents
                })),
            )
                .into_response())
        }
        Err(e) => {
            // Error checking KYC
            tracing::error!("KYC compliance check error: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()> {
                    success: false,
                    data: None,
                    error: Some("Failed to verify compliance status".to_string()),
                }),
            )
                .into_response())
        }
    }
}

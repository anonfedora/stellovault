//! Loan management HTTP handlers
//!
//! Endpoints for loan lifecycle management including creation, repayment, and status tracking.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::error::ApiError;
use crate::handlers::AuthenticatedUser;
use crate::loan::{CreateLoanRequest, ListLoansQuery, Loan, RepaymentRequest, Repayment};
use crate::state::AppState;

/// POST /api/v1/loans - Create a new loan with collateral
pub async fn create_loan(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(req): Json<CreateLoanRequest>,
) -> Result<(StatusCode, Json<Loan>), ApiError> {
    // Validate that the borrower is the authenticated user
    if req.borrower_id != user.user_id {
        return Err(ApiError::Forbidden(
            "Cannot create loan for another user".to_string(),
        ));
    }

    // Validate collateral exists and is owned by borrower
    let collateral_uuid = uuid::Uuid::parse_str(&req.collateral_id)
        .map_err(|_| ApiError::BadRequest("Invalid collateral ID format".to_string()))?;
    let collateral = state
        .collateral_service
        .get_collateral(collateral_uuid)
        .await
        .map_err(|e| ApiError::BadRequest(format!("Invalid collateral: {}", e)))?;

    if collateral.owner_id != user.user_id {
        return Err(ApiError::Forbidden(
            "Collateral not owned by borrower".to_string(),
        ));
    }

    // Validate LTV ratio (Loan-to-Value)
    let ltv = (req.principal_amount as f64 / collateral.face_value as f64) * 100.0;
    if ltv > 80.0 {
        return Err(ApiError::BadRequest(
            format!("LTV ratio {:.2}% exceeds maximum 80%", ltv),
        ));
    }

    // Create the loan
    let loan = state
        .loan_service
        .issue_loan(req)
        .await
        .map_err(|e| ApiError::InternalError(e.to_string()))?;

    Ok((StatusCode::CREATED, Json(loan)))
}

/// GET /api/v1/loans - List loans for the authenticated user
pub async fn list_loans(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Query(mut query): Query<ListLoansQuery>,
) -> Result<Json<Vec<Loan>>, ApiError> {
    // Default to listing loans for the current user
    if query.borrower_id.is_none() && query.lender_id.is_none() {
        query.borrower_id = Some(user.user_id);
    }

    // Validate user can only see their own loans
    if let Some(borrower_id) = query.borrower_id {
        if borrower_id != user.user_id {
            return Err(ApiError::Forbidden(
                "Cannot list loans for another user".to_string(),
            ));
        }
    }

    if let Some(lender_id) = query.lender_id {
        if lender_id != user.user_id {
            return Err(ApiError::Forbidden(
                "Cannot list loans for another user".to_string(),
            ));
        }
    }

    let loans = state
        .loan_service
        .list_loans(query)
        .await
        .map_err(|e| ApiError::InternalError(e.to_string()))?;

    Ok(Json(loans))
}

/// GET /api/v1/loans/:id - Get loan details
pub async fn get_loan(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(loan_id): Path<Uuid>,
) -> Result<Json<Loan>, ApiError> {
    let loan = state
        .loan_service
        .get_loan(loan_id)
        .await
        .map_err(|e| ApiError::NotFound(e.to_string()))?;

    // Verify user is borrower or lender
    if loan.borrower_id != user.user_id && loan.lender_id != user.user_id {
        return Err(ApiError::Forbidden(
            "Cannot access this loan".to_string(),
        ));
    }

    Ok(Json(loan))
}

/// POST /api/v1/loans/:id/repay - Make a loan repayment
pub async fn make_repayment(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(loan_id): Path<Uuid>,
    Json(mut req): Json<RepaymentRequest>,
) -> Result<(StatusCode, Json<Repayment>), ApiError> {
    // Get the loan to verify ownership
    let loan = state
        .loan_service
        .get_loan(loan_id)
        .await
        .map_err(|e| ApiError::NotFound(e.to_string()))?;

    // Only borrower can make repayments
    if loan.borrower_id != user.user_id {
        return Err(ApiError::Forbidden(
            "Only borrower can make repayments".to_string(),
        ));
    }

    // Validate repayment amount
    if req.amount <= 0 {
        return Err(ApiError::BadRequest(
            "Repayment amount must be positive".to_string(),
        ));
    }

    if req.amount > loan.outstanding_balance {
        return Err(ApiError::BadRequest(
            format!(
                "Repayment amount {} exceeds outstanding balance {}",
                req.amount, loan.outstanding_balance
            ),
        ));
    }

    req.loan_id = loan_id;

    // Record the repayment
    let repayment = state
        .loan_service
        .record_repayment(req)
        .await
        .map_err(|e| ApiError::InternalError(e.to_string()))?;

    Ok((StatusCode::CREATED, Json(repayment)))
}

/// GET /api/v1/loans/:id/schedule - Get repayment schedule
pub async fn get_repayment_schedule(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(loan_id): Path<Uuid>,
) -> Result<Json<RepaymentScheduleResponse>, ApiError> {
    let loan = state
        .loan_service
        .get_loan(loan_id)
        .await
        .map_err(|e| ApiError::NotFound(e.to_string()))?;

    // Verify user is borrower or lender
    if loan.borrower_id != user.user_id && loan.lender_id != user.user_id {
        return Err(ApiError::Forbidden(
            "Cannot access this loan".to_string(),
        ));
    }

    let schedule = state
        .loan_service
        .generate_repayment_schedule(loan_id)
        .await
        .map_err(|e| ApiError::InternalError(e.to_string()))?;

    Ok(Json(schedule))
}

/// POST /api/v1/loans/:id/extend - Request loan extension
pub async fn request_extension(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(loan_id): Path<Uuid>,
    Json(req): Json<ExtensionRequest>,
) -> Result<Json<Loan>, ApiError> {
    let loan = state
        .loan_service
        .get_loan(loan_id)
        .await
        .map_err(|e| ApiError::NotFound(e.to_string()))?;

    // Only borrower can request extension
    if loan.borrower_id != user.user_id {
        return Err(ApiError::Forbidden(
            "Only borrower can request extension".to_string(),
        ));
    }

    // Validate extension period
    if req.extension_days <= 0 || req.extension_days > 365 {
        return Err(ApiError::BadRequest(
            "Extension period must be between 1 and 365 days".to_string(),
        ));
    }

    let updated_loan = state
        .loan_service
        .extend_loan(loan_id, req.extension_days)
        .await
        .map_err(|e| ApiError::InternalError(e.to_string()))?;

    Ok(Json(updated_loan))
}

/// GET /api/v1/loans/:id/history - Get loan transaction history
pub async fn get_loan_history(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(loan_id): Path<Uuid>,
) -> Result<Json<Vec<LoanHistoryEntry>>, ApiError> {
    let loan = state
        .loan_service
        .get_loan(loan_id)
        .await
        .map_err(|e| ApiError::NotFound(e.to_string()))?;

    // Verify user is borrower or lender
    if loan.borrower_id != user.user_id && loan.lender_id != user.user_id {
        return Err(ApiError::Forbidden(
            "Cannot access this loan".to_string(),
        ));
    }

    let history = state
        .loan_service
        .get_loan_history(loan_id)
        .await
        .map_err(|e| ApiError::InternalError(e.to_string()))?;

    Ok(Json(history))
}

// ============================================================================
// Request/Response DTOs
// ============================================================================

/// Request to extend a loan
#[derive(Debug, serde::Deserialize)]
pub struct ExtensionRequest {
    pub extension_days: i32,
}

/// Repayment schedule response
#[derive(Debug, serde::Serialize)]
pub struct RepaymentScheduleResponse {
    pub loan_id: Uuid,
    pub total_amount: i64,
    pub outstanding_balance: i64,
    pub next_payment_due: chrono::DateTime<chrono::Utc>,
    pub payments: Vec<ScheduledPayment>,
}

/// Individual scheduled payment
#[derive(Debug, serde::Serialize)]
pub struct ScheduledPayment {
    pub payment_number: i32,
    pub due_date: chrono::DateTime<chrono::Utc>,
    pub principal: i64,
    pub interest: i64,
    pub total: i64,
}

/// Loan history entry
#[derive(Debug, serde::Serialize)]
pub struct LoanHistoryEntry {
    pub event_type: String,
    pub amount: Option<i64>,
    pub description: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

//! Loan models for StelloVault
use serde::{Deserialize, Serialize};
use sqlx::types::chrono::{DateTime, Utc};
use uuid::Uuid;

/// Loan status enum
#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, Copy, PartialEq, Eq)]
#[sqlx(type_name = "loan_status", rename_all = "lowercase")]
pub enum LoanStatus {
    Active,
    Repaid,
    Defaulted,
    Liquidated,
}

/// Loan model
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Loan {
    pub id: Uuid,
    pub loan_id: String, // Soroban contract loan ID
    pub borrower_id: Uuid,
    pub lender_id: Uuid,
    pub collateral_id: String,
    pub principal_amount: i64,
    pub outstanding_balance: i64,
    pub interest_rate: i32, // basis points
    pub status: LoanStatus,
    pub due_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Repayment model
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Repayment {
    pub id: Uuid,
    pub loan_id: Uuid,
    pub amount: i64,
    pub tx_hash: String,
    pub created_at: DateTime<Utc>,
}

/// Request to create a new loan
#[derive(Debug, Deserialize)]
pub struct CreateLoanRequest {
    pub loan_id: String,
    pub borrower_id: Uuid,
    pub lender_id: Uuid,
    pub collateral_id: String,
    pub principal_amount: i64,
    pub interest_rate: i32,
    pub timeout_hours: i64,
}

/// Request to record a repayment
#[derive(Debug, Deserialize)]
pub struct RepaymentRequest {
    pub loan_id: Uuid,
    pub amount: i64,
    pub tx_hash: String,
}

/// Query for listing loans
#[derive(Debug, Deserialize)]
pub struct ListLoansQuery {
    pub borrower_id: Option<Uuid>,
    pub lender_id: Option<Uuid>,
    pub status: Option<LoanStatus>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

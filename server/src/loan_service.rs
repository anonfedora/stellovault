//! Loan service layer - Business logic for loan management

use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::loan::{CreateLoanRequest, Loan, LoanStatus, Repayment, RepaymentRequest};

/// Loan service for managing loan lifecycle
#[derive(Clone)]
pub struct LoanService {
    db_pool: PgPool,
}

impl LoanService {
    /// Create a new loan service instance
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    /// Issue a new loan (simulated on-chain interaction)
    pub async fn issue_loan(&self, request: CreateLoanRequest) -> Result<Loan> {
        let timeout_at = Utc::now() + Duration::hours(request.timeout_hours);

        // In a real scenario, we would call Soroban here.
        // For now, we simulate success and store in DB.

        let loan = sqlx::query_as::<_, Loan>(
            r#"
            INSERT INTO loans (
                loan_id, borrower_id, lender_id, collateral_id, 
                principal_amount, outstanding_balance, interest_rate, 
                status, due_at, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING *
            "#,
        )
        .bind(request.loan_id)
        .bind(request.borrower_id)
        .bind(request.lender_id)
        .bind(request.collateral_id)
        .bind(request.principal_amount)
        .bind(request.principal_amount) // Initial balance is principal
        .bind(request.interest_rate)
        .bind(LoanStatus::Active)
        .bind(timeout_at)
        .bind(Utc::now())
        .bind(Utc::now())
        .fetch_one(&self.db_pool)
        .await
        .context("Failed to insert loan into database")?;

        Ok(loan)
    }

    /// Record a repayment and update loan balance
    pub async fn record_repayment(&self, request: RepaymentRequest) -> Result<Repayment> {
        let mut tx = self.db_pool.begin().await?;

        // 1. Create repayment record
        let repayment = sqlx::query_as::<_, Repayment>(
            r#"
            INSERT INTO repayments (loan_id, amount, tx_hash, created_at)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
        )
        .bind(request.loan_id)
        .bind(request.amount)
        .bind(request.tx_hash)
        .bind(Utc::now())
        .fetch_one(&mut *tx)
        .await
        .context("Failed to insert repayment record")?;

        // 2. Update loan balance and status
        let loan = sqlx::query_as::<_, Loan>("SELECT * FROM loans WHERE id = $1 FOR UPDATE")
            .bind(request.loan_id)
            .fetch_one(&mut *tx)
            .await?;

        let new_balance = (loan.outstanding_balance - request.amount).max(0);
        let new_status = if new_balance == 0 {
            LoanStatus::Repaid
        } else {
            loan.status
        };

        sqlx::query(
            "UPDATE loans SET outstanding_balance = $1, status = $2, updated_at = $3 WHERE id = $4",
        )
        .bind(new_balance)
        .bind(new_status)
        .bind(Utc::now())
        .bind(request.loan_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(repayment)
    }

    /// Calculate interest accrual for all active loans
    /// This would typically be called by a background worker
    pub async fn accrue_interest(&self) -> Result<()> {
        // Simple logic: add interest if time has passed.
        // For a more realistic implementation, we'd track last_accrued_at.
        // For now, let's just demonstrate the logic.

        let active_loans = sqlx::query_as::<_, Loan>("SELECT * FROM loans WHERE status = 'active'")
            .fetch_all(&self.db_pool)
            .await?;

        for loan in active_loans {
            // Logic: 1% increase for demonstration
            let interest = (loan.outstanding_balance * loan.interest_rate as i64) / 10000;
            if interest > 0 {
                sqlx::query(
                    "UPDATE loans SET outstanding_balance = outstanding_balance + $1, updated_at = $2 WHERE id = $3"
                )
                .bind(interest)
                .bind(Utc::now())
                .bind(loan.id)
                .execute(&self.db_pool)
                .await?;
            }
        }

        Ok(())
    }

    /// Detect defaulted loans (past due)
    pub async fn detect_defaults(&self) -> Result<Vec<Uuid>> {
        let defaulted = sqlx::query_as::<_, (Uuid,)>(
            r#"
            UPDATE loans 
            SET status = 'defaulted', updated_at = $1
            WHERE status = 'active' AND due_at < $1
            RETURNING id
            "#,
        )
        .bind(Utc::now())
        .fetch_all(&self.db_pool)
        .await?;

        Ok(defaulted.into_iter().map(|(id,)| id).collect())
    }

    /// Get loan by ID
    pub async fn get_loan(&self, id: &Uuid) -> Result<Option<Loan>> {
        let loan = sqlx::query_as::<_, Loan>("SELECT * FROM loans WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.db_pool)
            .await?;
        Ok(loan)
    }

    /// List loans with filters
    pub async fn list_loans(
        &self,
        borrower_id: Option<Uuid>,
        lender_id: Option<Uuid>,
        status: Option<LoanStatus>,
    ) -> Result<Vec<Loan>> {
        let mut query = String::from("SELECT * FROM loans WHERE 1=1");

        if let Some(b_id) = borrower_id {
            query.push_str(&format!(" AND borrower_id = '{}'", b_id));
        }
        if let Some(l_id) = lender_id {
            query.push_str(&format!(" AND lender_id = '{}'", l_id));
        }
        if let Some(s) = status {
            query.push_str(&format!(" AND status = '{:?}'", s).to_lowercase());
        }

        let loans = sqlx::query_as::<_, Loan>(&query)
            .fetch_all(&self.db_pool)
            .await?;

        Ok(loans)
    }
}

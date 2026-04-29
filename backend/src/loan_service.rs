//! Loan service layer - Business logic for loan management

use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::loan::{CreateLoanRequest, ListLoansQuery, Loan, LoanStatus, Repayment, RepaymentRequest};
use crate::handlers::loan::{LoanHistoryEntry, RepaymentScheduleResponse, ScheduledPayment};

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

    /// Get reference to the database pool
    pub fn db_pool(&self) -> &PgPool {
        &self.db_pool
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
    /// Also triggers risk score recalculation for the borrower
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

        // 3. Trigger risk score update (async, non-blocking)
        // Queue the update for background processing
        let borrower_id = loan.borrower_id;
        let db_pool = self.db_pool.clone();
        
        tokio::spawn(async move {
            if let Err(e) = Self::trigger_risk_score_update(db_pool, borrower_id).await {
                tracing::error!(
                    borrower_id = %borrower_id,
                    error = %e,
                    "Failed to trigger risk score update after repayment"
                );
            }
        });

        tracing::info!(
            loan_id = %request.loan_id,
            borrower_id = %borrower_id,
            amount = request.amount,
            new_status = ?new_status,
            "Repayment recorded and risk score update triggered"
        );

        Ok(repayment)
    }

    /// Trigger risk score update for a user
    /// This is called automatically after successful loan repayment
    async fn trigger_risk_score_update(db_pool: PgPool, user_id: Uuid) -> Result<()> {
        // Insert into a queue table for async processing
        sqlx::query(
            r#"
            INSERT INTO risk_score_update_queue (user_id, trigger_type, created_at)
            VALUES ($1, 'repayment', NOW())
            ON CONFLICT DO NOTHING
            "#
        )
        .bind(user_id)
        .execute(&db_pool)
        .await
        .context("Failed to queue risk score update")?;

        tracing::info!(
            user_id = %user_id,
            "Risk score update queued for processing"
        );

        Ok(())
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
    pub async fn get_loan(&self, id: Uuid) -> Result<Loan> {
        let loan = sqlx::query_as::<_, Loan>("SELECT * FROM loans WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.db_pool)
            .await?
            .context("Loan not found")?;
        Ok(loan)
    }

    /// List loans with filters
    pub async fn list_loans(&self, query: ListLoansQuery) -> Result<Vec<Loan>> {
        let mut sql = String::from("SELECT * FROM loans WHERE 1=1");

        if let Some(borrower_id) = query.borrower_id {
            sql.push_str(&format!(" AND borrower_id = '{}'", borrower_id));
        }
        if let Some(lender_id) = query.lender_id {
            sql.push_str(&format!(" AND lender_id = '{}'", lender_id));
        }
        if let Some(status) = query.status {
            let status_str = match status {
                LoanStatus::Active => "active",
                LoanStatus::Repaid => "repaid",
                LoanStatus::Defaulted => "defaulted",
                LoanStatus::Liquidated => "liquidated",
            };
            sql.push_str(&format!(" AND status = '{}'", status_str));
        }

        sql.push_str(" ORDER BY created_at DESC");

        let limit = query.limit.unwrap_or(50).min(100);
        let offset = (query.page.unwrap_or(0)) * limit;
        sql.push_str(&format!(" LIMIT {} OFFSET {}", limit, offset));

        let loans = sqlx::query_as::<_, Loan>(&sql)
            .fetch_all(&self.db_pool)
            .await?;

        Ok(loans)
    }

    /// Generate repayment schedule for a loan
    pub async fn generate_repayment_schedule(&self, loan_id: Uuid) -> Result<RepaymentScheduleResponse> {
        let loan = self.get_loan(loan_id).await?;

        // Calculate monthly payments (simplified: equal principal + interest)
        let num_months = 12i64; // Default to 12 months
        let monthly_principal = loan.principal_amount / num_months;
        let monthly_interest_rate = loan.interest_rate as f64 / 10000.0 / 12.0;

        let mut payments = Vec::new();
        let mut remaining_balance = loan.principal_amount;

        for i in 1..=num_months {
            let interest = (remaining_balance as f64 * monthly_interest_rate).ceil() as i64;
            let principal = if i == num_months {
                remaining_balance
            } else {
                monthly_principal
            };

            let due_date = loan.due_at - Duration::days((num_months - i) * 30);

            payments.push(ScheduledPayment {
                payment_number: i as i32,
                due_date,
                principal,
                interest,
                total: principal + interest,
            });

            remaining_balance -= principal;
        }

        Ok(RepaymentScheduleResponse {
            loan_id,
            total_amount: loan.principal_amount,
            outstanding_balance: loan.outstanding_balance,
            next_payment_due: loan.due_at,
            payments,
        })
    }

    /// Extend a loan's due date
    pub async fn extend_loan(&self, loan_id: Uuid, extension_days: i32) -> Result<Loan> {
        let loan = self.get_loan(loan_id).await?;

        let new_due_date = loan.due_at + Duration::days(extension_days as i64);

        let updated_loan = sqlx::query_as::<_, Loan>(
            "UPDATE loans SET due_at = $1, updated_at = $2 WHERE id = $3 RETURNING *"
        )
        .bind(new_due_date)
        .bind(Utc::now())
        .bind(loan_id)
        .fetch_one(&self.db_pool)
        .await?;

        tracing::info!(
            loan_id = %loan_id,
            extension_days = extension_days,
            new_due_date = %new_due_date,
            "Loan extended"
        );

        Ok(updated_loan)
    }

    /// Get loan transaction history
    pub async fn get_loan_history(&self, loan_id: Uuid) -> Result<Vec<LoanHistoryEntry>> {
        let _loan = self.get_loan(loan_id).await?;

        // Get all repayments for this loan
        let repayments = sqlx::query_as::<_, Repayment>(
            "SELECT * FROM repayments WHERE loan_id = $1 ORDER BY created_at DESC"
        )
        .bind(loan_id)
        .fetch_all(&self.db_pool)
        .await?;

        let mut history = Vec::new();

        for repayment in repayments {
            history.push(LoanHistoryEntry {
                event_type: "repayment".to_string(),
                amount: Some(repayment.amount),
                description: format!("Repayment of {} processed", repayment.amount),
                timestamp: repayment.created_at,
            });
        }

        // Get loan creation event
        let loan = self.get_loan(loan_id).await?;
        history.push(LoanHistoryEntry {
            event_type: "creation".to_string(),
            amount: Some(loan.principal_amount),
            description: format!("Loan created with principal {}", loan.principal_amount),
            timestamp: loan.created_at,
        });

        history.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        Ok(history)
    }
}

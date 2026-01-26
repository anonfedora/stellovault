//! Risk Scoring & Analytics Engine for StelloVault
//!
//! This module implements credit scoring and risk assessment based on
//! historical on-chain and off-chain data. The scores are advisory only -
//! smart contracts enforce final rules.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::ApiError;

// ============================================================================
// Configuration Constants
// ============================================================================

/// Weight for deal count in overall score (0-1)
const WEIGHT_DEAL_COUNT: f64 = 0.20;

/// Weight for repayment ratio in overall score (0-1)
const WEIGHT_REPAYMENT_RATIO: f64 = 0.35;

/// Weight for escrow completion rate in overall score (0-1)
const WEIGHT_ESCROW_COMPLETION: f64 = 0.25;

/// Weight for account age factor in overall score (0-1)
const WEIGHT_ACCOUNT_AGE: f64 = 0.10;

/// Weight for average deal size consistency in overall score (0-1)
const WEIGHT_DEAL_CONSISTENCY: f64 = 0.10;

/// Time decay half-life in days (older transactions count less)
const TIME_DECAY_HALF_LIFE_DAYS: f64 = 90.0;

/// Minimum deals required for a reliable score
const MIN_DEALS_FOR_RELIABLE_SCORE: i32 = 5;

/// Maximum risk score (scale 0-1000)
const MAX_RISK_SCORE: i32 = 1000;

/// Minimum risk score
const MIN_RISK_SCORE: i32 = 0;

/// Default score for new users with no history
const DEFAULT_NEW_USER_SCORE: i32 = 500;

// ============================================================================
// Data Models
// ============================================================================

/// Risk score response returned by the API
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RiskScoreResponse {
    /// Wallet address being scored
    pub wallet_address: String,

    /// Overall risk score (0-1000, higher is better)
    pub overall_score: i32,

    /// Risk tier classification
    pub risk_tier: RiskTier,

    /// Individual metric scores
    pub metrics: RiskMetrics,

    /// Fraud indicators detected
    pub fraud_indicators: Vec<FraudIndicator>,

    /// Confidence level of the score (0.0-1.0)
    pub confidence: f64,

    /// Whether the score is reliable (based on data availability)
    pub is_reliable: bool,

    /// When the score was calculated
    pub calculated_at: DateTime<Utc>,

    /// Summary of the scoring factors
    pub summary: ScoreSummary,
}

/// Risk tier classification
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RiskTier {
    /// Excellent credit (800-1000)
    Excellent,
    /// Good credit (650-799)
    Good,
    /// Fair credit (500-649)
    Fair,
    /// Poor credit (300-499)
    Poor,
    /// Very high risk (0-299)
    HighRisk,
    /// Insufficient data to score
    Unscored,
}

impl RiskTier {
    pub fn from_score(score: i32) -> Self {
        match score {
            800..=1000 => RiskTier::Excellent,
            650..=799 => RiskTier::Good,
            500..=649 => RiskTier::Fair,
            300..=499 => RiskTier::Poor,
            _ => RiskTier::HighRisk,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            RiskTier::Excellent => "Excellent credit history with consistent repayment",
            RiskTier::Good => "Good credit history with minor issues",
            RiskTier::Fair => "Fair credit history, moderate risk",
            RiskTier::Poor => "Poor credit history, elevated risk",
            RiskTier::HighRisk => "High risk based on historical behavior",
            RiskTier::Unscored => "Insufficient data to generate a reliable score",
        }
    }
}

/// Individual risk metrics that contribute to the overall score
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RiskMetrics {
    /// Total number of completed deals (loans + escrows)
    pub deal_count: DealCountMetric,

    /// Loan repayment performance
    pub repayment_ratio: RepaymentMetric,

    /// Escrow completion performance
    pub escrow_completion: EscrowMetric,

    /// Account age and history
    pub account_age: AccountAgeMetric,

    /// Deal size consistency (less variance = more reliable)
    pub deal_consistency: ConsistencyMetric,
}

/// Deal count metric details
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DealCountMetric {
    pub total_deals: i32,
    pub as_borrower: i32,
    pub as_lender: i32,
    pub as_buyer: i32,
    pub as_seller: i32,
    /// Normalized score (0-1000)
    pub score: i32,
    /// Weight applied in overall calculation
    pub weight: f64,
}

/// Repayment ratio metric details
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RepaymentMetric {
    pub total_loans: i32,
    pub repaid_on_time: i32,
    pub repaid_late: i32,
    pub defaulted: i32,
    pub active: i32,
    /// Ratio of successful repayments (0.0-1.0)
    pub ratio: f64,
    /// Time-decayed ratio giving more weight to recent transactions
    pub time_decayed_ratio: f64,
    /// Normalized score (0-1000)
    pub score: i32,
    pub weight: f64,
}

/// Escrow completion metric details
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EscrowMetric {
    pub total_escrows: i32,
    pub completed_successfully: i32,
    pub cancelled: i32,
    pub disputed: i32,
    pub timed_out: i32,
    /// Completion ratio (0.0-1.0)
    pub completion_ratio: f64,
    /// Dispute ratio - lower is better
    pub dispute_ratio: f64,
    /// Normalized score (0-1000)
    pub score: i32,
    pub weight: f64,
}

/// Account age metric details
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccountAgeMetric {
    pub account_created_at: Option<DateTime<Utc>>,
    pub first_transaction_at: Option<DateTime<Utc>>,
    pub account_age_days: i32,
    pub active_period_days: i32,
    /// Normalized score (0-1000)
    pub score: i32,
    pub weight: f64,
}

/// Deal consistency metric details
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConsistencyMetric {
    pub average_deal_size: i64,
    pub deal_size_std_dev: f64,
    /// Coefficient of variation (std_dev / mean) - lower is more consistent
    pub coefficient_of_variation: f64,
    /// Transaction frequency (deals per month)
    pub deals_per_month: f64,
    /// Normalized score (0-1000)
    pub score: i32,
    pub weight: f64,
}

/// Fraud indicators detected during scoring
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FraudIndicator {
    pub indicator_type: FraudIndicatorType,
    pub severity: FraudSeverity,
    pub description: String,
    pub detected_at: DateTime<Utc>,
    /// Score penalty applied (-1000 to 0)
    pub score_impact: i32,
}

/// Types of fraud indicators
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FraudIndicatorType {
    /// Rapid succession of small deals followed by large one
    SmurfingPattern,
    /// Circular transactions between related wallets
    CircularTransactions,
    /// Sudden change in transaction patterns
    AnomalousActivity,
    /// Multiple disputes with same counterparty
    RepeatedDisputes,
    /// Account age doesn't match activity level
    SuspiciousAccountAge,
    /// Self-dealing detected
    SelfDealing,
    /// Unusually high default rate
    HighDefaultRate,
    /// Flash loan-like patterns
    FlashLoanPattern,
}

/// Fraud indicator severity levels
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FraudSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Summary of scoring factors
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScoreSummary {
    pub positive_factors: Vec<String>,
    pub negative_factors: Vec<String>,
    pub recommendations: Vec<String>,
}

// ============================================================================
// Internal Data Structures for Queries
// ============================================================================

/// Raw loan statistics from database
#[derive(Debug, sqlx::FromRow)]
#[allow(dead_code)]
struct LoanStats {
    total_loans: Option<i64>,
    repaid_count: Option<i64>,
    defaulted_count: Option<i64>,
    active_count: Option<i64>,
    total_principal: Option<i64>,
    total_repaid_amount: Option<i64>,
}

/// Raw escrow statistics from database
#[derive(Debug, sqlx::FromRow)]
#[allow(dead_code)]
struct EscrowStats {
    total_escrows: Option<i64>,
    released_count: Option<i64>,
    cancelled_count: Option<i64>,
    disputed_count: Option<i64>,
    timed_out_count: Option<i64>,
    total_amount: Option<i64>,
}

/// Loan with timing information for time decay calculation
#[derive(Debug, sqlx::FromRow)]
#[allow(dead_code)]
struct LoanWithTiming {
    id: Uuid,
    status: String,
    principal_amount: i64,
    created_at: DateTime<Utc>,
    due_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

/// Escrow with timing information
#[derive(Debug, sqlx::FromRow)]
#[allow(dead_code)]
struct EscrowWithTiming {
    id: Uuid,
    status: String,
    amount: i64,
    disputed: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

/// User account info
#[derive(Debug, sqlx::FromRow)]
struct UserAccount {
    id: Uuid,
    created_at: DateTime<Utc>,
}

/// Deal amounts for consistency calculation
#[derive(Debug)]
struct DealAmounts {
    amounts: Vec<i64>,
    timestamps: Vec<DateTime<Utc>>,
}

// ============================================================================
// Risk Engine Service
// ============================================================================

/// Risk scoring engine service
#[derive(Clone)]
pub struct RiskEngine {
    db_pool: PgPool,
}

impl RiskEngine {
    /// Create a new risk engine instance
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    /// Calculate risk score for a wallet address
    pub async fn calculate_risk_score(
        &self,
        wallet_address: &str,
    ) -> Result<RiskScoreResponse, ApiError> {
        // 1. Find user by wallet address
        let user = self.get_user_by_wallet(wallet_address).await?;

        let user = match user {
            Some(u) => u,
            None => {
                // Return default score for unknown wallet
                return Ok(self.create_unscored_response(wallet_address));
            }
        };

        // 2. Gather all metrics
        let loan_stats = self.get_loan_statistics(user.id).await?;
        let escrow_stats = self.get_escrow_statistics(user.id).await?;
        let loans_with_timing = self.get_loans_with_timing(user.id).await?;
        let escrows_with_timing = self.get_escrows_with_timing(user.id).await?;
        let deal_amounts = self.get_deal_amounts(user.id).await?;

        // 3. Calculate individual metrics
        let deal_count_metric = self.calculate_deal_count_metric(&loan_stats, &escrow_stats);
        let repayment_metric =
            self.calculate_repayment_metric(&loan_stats, &loans_with_timing);
        let escrow_metric =
            self.calculate_escrow_metric(&escrow_stats, &escrows_with_timing);
        let account_age_metric =
            self.calculate_account_age_metric(&user, &loans_with_timing, &escrows_with_timing);
        let consistency_metric = self.calculate_consistency_metric(&deal_amounts);

        // 4. Detect fraud indicators
        let fraud_indicators = self
            .detect_fraud_indicators(
                &loan_stats,
                &escrow_stats,
                &loans_with_timing,
                &escrows_with_timing,
                &account_age_metric,
            )
            .await;

        // 5. Calculate overall score
        let (overall_score, confidence) = self.calculate_overall_score(
            &deal_count_metric,
            &repayment_metric,
            &escrow_metric,
            &account_age_metric,
            &consistency_metric,
            &fraud_indicators,
        );

        // 6. Determine if score is reliable
        let total_deals = deal_count_metric.total_deals;
        let is_reliable = total_deals >= MIN_DEALS_FOR_RELIABLE_SCORE;

        // 7. Generate summary
        let summary = self.generate_summary(
            &deal_count_metric,
            &repayment_metric,
            &escrow_metric,
            &fraud_indicators,
            is_reliable,
        );

        // 8. Build response
        let risk_tier = if is_reliable {
            RiskTier::from_score(overall_score)
        } else {
            RiskTier::Unscored
        };

        Ok(RiskScoreResponse {
            wallet_address: wallet_address.to_string(),
            overall_score,
            risk_tier,
            metrics: RiskMetrics {
                deal_count: deal_count_metric,
                repayment_ratio: repayment_metric,
                escrow_completion: escrow_metric,
                account_age: account_age_metric,
                deal_consistency: consistency_metric,
            },
            fraud_indicators,
            confidence,
            is_reliable,
            calculated_at: Utc::now(),
            summary,
        })
    }

    /// Get historical risk scores for backtesting
    pub async fn get_historical_scores(
        &self,
        wallet_address: &str,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<HistoricalScore>, ApiError> {
        // For backtesting, we simulate what the score would have been at each point
        let user = self.get_user_by_wallet(wallet_address).await?;
        
        let user = match user {
            Some(u) => u,
            None => return Ok(vec![]),
        };

        let mut historical_scores = Vec::new();
        let mut current_date = start_date;
        
        // Generate weekly snapshots
        while current_date <= end_date {
            let score = self
                .calculate_score_at_point_in_time(user.id, current_date)
                .await?;
            
            historical_scores.push(HistoricalScore {
                date: current_date,
                score,
                tier: RiskTier::from_score(score),
            });
            
            current_date = current_date + Duration::days(7);
        }

        Ok(historical_scores)
    }

    /// Run simulation with hypothetical scenarios
    pub async fn simulate_score_impact(
        &self,
        wallet_address: &str,
        scenario: SimulationScenario,
    ) -> Result<SimulationResult, ApiError> {
        let current_score = self.calculate_risk_score(wallet_address).await?;
        
        // Calculate projected score based on scenario
        let projected_score = self.apply_scenario_to_score(&current_score, &scenario);
        
        Ok(SimulationResult {
            current_score: current_score.overall_score,
            projected_score,
            score_change: projected_score - current_score.overall_score,
            scenario_description: scenario.description(),
            recommendations: self.generate_scenario_recommendations(&scenario, projected_score),
        })
    }

    // ========================================================================
    // Private Helper Methods
    // ========================================================================

    async fn get_user_by_wallet(&self, wallet_address: &str) -> Result<Option<UserAccount>, ApiError> {
        let user = sqlx::query_as::<_, UserAccount>(
            "SELECT id, created_at FROM users WHERE primary_wallet_address = $1",
        )
        .bind(wallet_address)
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(user)
    }

    async fn get_loan_statistics(&self, user_id: Uuid) -> Result<LoanStats, ApiError> {
        let stats = sqlx::query_as::<_, LoanStats>(
            r#"
            SELECT 
                COUNT(*) as total_loans,
                COUNT(*) FILTER (WHERE status = 'repaid') as repaid_count,
                COUNT(*) FILTER (WHERE status = 'defaulted' OR status = 'liquidated') as defaulted_count,
                COUNT(*) FILTER (WHERE status = 'active') as active_count,
                COALESCE(SUM(principal_amount), 0) as total_principal,
                COALESCE(SUM(principal_amount) FILTER (WHERE status = 'repaid'), 0) as total_repaid_amount
            FROM loans
            WHERE borrower_id = $1 OR lender_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(stats)
    }

    async fn get_escrow_statistics(&self, user_id: Uuid) -> Result<EscrowStats, ApiError> {
        let stats = sqlx::query_as::<_, EscrowStats>(
            r#"
            SELECT 
                COUNT(*) as total_escrows,
                COUNT(*) FILTER (WHERE status = 'released') as released_count,
                COUNT(*) FILTER (WHERE status = 'cancelled') as cancelled_count,
                COUNT(*) FILTER (WHERE disputed = true) as disputed_count,
                COUNT(*) FILTER (WHERE status = 'timedout') as timed_out_count,
                COALESCE(SUM(amount), 0) as total_amount
            FROM escrows
            WHERE buyer_id = $1 OR seller_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(stats)
    }

    async fn get_loans_with_timing(&self, user_id: Uuid) -> Result<Vec<LoanWithTiming>, ApiError> {
        let loans = sqlx::query_as::<_, LoanWithTiming>(
            r#"
            SELECT id, status::text as status, principal_amount, created_at, due_at, updated_at
            FROM loans
            WHERE borrower_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(loans)
    }

    async fn get_escrows_with_timing(&self, user_id: Uuid) -> Result<Vec<EscrowWithTiming>, ApiError> {
        let escrows = sqlx::query_as::<_, EscrowWithTiming>(
            r#"
            SELECT id, status::text as status, amount, disputed, created_at, updated_at
            FROM escrows
            WHERE buyer_id = $1 OR seller_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(escrows)
    }

    async fn get_deal_amounts(&self, user_id: Uuid) -> Result<DealAmounts, ApiError> {
        // Combine loan and escrow amounts
        let loan_amounts: Vec<(i64, DateTime<Utc>)> = sqlx::query_as(
            "SELECT principal_amount, created_at FROM loans WHERE borrower_id = $1 OR lender_id = $1",
        )
        .bind(user_id)
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        let escrow_amounts: Vec<(i64, DateTime<Utc>)> = sqlx::query_as(
            "SELECT amount, created_at FROM escrows WHERE buyer_id = $1 OR seller_id = $1",
        )
        .bind(user_id)
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        let mut amounts = Vec::new();
        let mut timestamps = Vec::new();

        for (amount, ts) in loan_amounts.into_iter().chain(escrow_amounts) {
            amounts.push(amount);
            timestamps.push(ts);
        }

        Ok(DealAmounts { amounts, timestamps })
    }

    fn calculate_deal_count_metric(
        &self,
        loan_stats: &LoanStats,
        escrow_stats: &EscrowStats,
    ) -> DealCountMetric {
        let total_loans = loan_stats.total_loans.unwrap_or(0) as i32;
        let total_escrows = escrow_stats.total_escrows.unwrap_or(0) as i32;
        let total_deals = total_loans + total_escrows;

        // Score based on deal count with diminishing returns
        // 0 deals = 0, 5 deals = 500, 20+ deals = 1000
        let score = if total_deals == 0 {
            0
        } else {
            let normalized = (total_deals as f64 / 20.0).min(1.0);
            (normalized.sqrt() * MAX_RISK_SCORE as f64) as i32
        };

        DealCountMetric {
            total_deals,
            as_borrower: total_loans / 2, // Approximation
            as_lender: total_loans / 2,
            as_buyer: total_escrows / 2,
            as_seller: total_escrows / 2,
            score,
            weight: WEIGHT_DEAL_COUNT,
        }
    }

    fn calculate_repayment_metric(
        &self,
        loan_stats: &LoanStats,
        loans_with_timing: &[LoanWithTiming],
    ) -> RepaymentMetric {
        let total_loans = loan_stats.total_loans.unwrap_or(0) as i32;
        let repaid = loan_stats.repaid_count.unwrap_or(0) as i32;
        let defaulted = loan_stats.defaulted_count.unwrap_or(0) as i32;
        let active = loan_stats.active_count.unwrap_or(0) as i32;

        // Calculate basic ratio
        let completed_loans = repaid + defaulted;
        let ratio = if completed_loans > 0 {
            repaid as f64 / completed_loans as f64
        } else {
            1.0 // No completed loans = perfect ratio by default
        };

        // Calculate time-decayed ratio
        let time_decayed_ratio = self.calculate_time_decayed_loan_ratio(loans_with_timing);

        // Score: heavily penalize defaults
        let score = if total_loans == 0 {
            DEFAULT_NEW_USER_SCORE // Neutral for new users
        } else {
            let base_score = time_decayed_ratio * MAX_RISK_SCORE as f64;
            // Apply penalty for defaults
            let penalty = (defaulted as f64 * 100.0).min(base_score);
            (base_score - penalty).max(MIN_RISK_SCORE as f64) as i32
        };

        RepaymentMetric {
            total_loans,
            repaid_on_time: repaid,
            repaid_late: 0, // Would need additional tracking
            defaulted,
            active,
            ratio,
            time_decayed_ratio,
            score,
            weight: WEIGHT_REPAYMENT_RATIO,
        }
    }

    fn calculate_time_decayed_loan_ratio(&self, loans: &[LoanWithTiming]) -> f64 {
        if loans.is_empty() {
            return 1.0;
        }

        let now = Utc::now();
        let mut weighted_sum = 0.0;
        let mut weight_total = 0.0;

        for loan in loans {
            let age_days = (now - loan.created_at).num_days() as f64;
            let decay_factor = 0.5_f64.powf(age_days / TIME_DECAY_HALF_LIFE_DAYS);

            let outcome_score = match loan.status.as_str() {
                "repaid" => 1.0,
                "active" => 0.5, // Neutral for active loans
                "defaulted" | "liquidated" => 0.0,
                _ => 0.5,
            };

            weighted_sum += outcome_score * decay_factor;
            weight_total += decay_factor;
        }

        if weight_total > 0.0 {
            weighted_sum / weight_total
        } else {
            1.0
        }
    }

    /// Calculate time-decayed escrow completion ratio
    /// Recent escrows are weighted more heavily than older ones
    fn calculate_time_decayed_escrow_ratio(&self, escrows: &[EscrowWithTiming]) -> f64 {
        if escrows.is_empty() {
            return 1.0;
        }

        let now = Utc::now();
        let mut weighted_sum = 0.0;
        let mut weight_total = 0.0;

        for escrow in escrows {
            let age_days = (now - escrow.created_at).num_days() as f64;
            let decay_factor = 0.5_f64.powf(age_days / TIME_DECAY_HALF_LIFE_DAYS);

            // Score based on escrow status
            let outcome_score = match escrow.status.as_str() {
                "released" => 1.0,     // Successful completion
                "active" => 0.5,       // Still in progress
                "cancelled" => 0.3,    // Cancelled (not ideal but not failure)
                "disputed" => 0.1,     // Disputed is negative
                "timedout" => 0.0,     // Timeout is worst outcome
                _ => 0.5,
            };

            // Additional penalty for disputed escrows
            let dispute_factor = if escrow.disputed { 0.5 } else { 1.0 };

            weighted_sum += outcome_score * decay_factor * dispute_factor;
            weight_total += decay_factor;
        }

        if weight_total > 0.0 {
            weighted_sum / weight_total
        } else {
            1.0
        }
    }

    fn calculate_escrow_metric(
        &self,
        escrow_stats: &EscrowStats,
        escrows_with_timing: &[EscrowWithTiming],
    ) -> EscrowMetric {
        let total = escrow_stats.total_escrows.unwrap_or(0) as i32;
        let released = escrow_stats.released_count.unwrap_or(0) as i32;
        let cancelled = escrow_stats.cancelled_count.unwrap_or(0) as i32;
        let disputed = escrow_stats.disputed_count.unwrap_or(0) as i32;
        let timed_out = escrow_stats.timed_out_count.unwrap_or(0) as i32;

        // Calculate time-weighted completion ratio for more accurate scoring
        let time_weighted_completion = self.calculate_time_decayed_escrow_ratio(escrows_with_timing);
        
        let completion_ratio = if total > 0 {
            released as f64 / total as f64
        } else {
            1.0
        };

        let dispute_ratio = if total > 0 {
            disputed as f64 / total as f64
        } else {
            0.0
        };

        // Score: reward completions, penalize disputes and timeouts
        // Use time-weighted ratio for more accurate recent performance
        let score = if total == 0 {
            DEFAULT_NEW_USER_SCORE
        } else {
            let base_score = time_weighted_completion * MAX_RISK_SCORE as f64;
            let dispute_penalty = dispute_ratio * 200.0;
            let timeout_penalty = (timed_out as f64 / total as f64) * 100.0;
            (base_score - dispute_penalty - timeout_penalty).max(MIN_RISK_SCORE as f64) as i32
        };

        EscrowMetric {
            total_escrows: total,
            completed_successfully: released,
            cancelled,
            disputed,
            timed_out,
            completion_ratio,
            dispute_ratio,
            score,
            weight: WEIGHT_ESCROW_COMPLETION,
        }
    }

    fn calculate_account_age_metric(
        &self,
        user: &UserAccount,
        loans: &[LoanWithTiming],
        escrows: &[EscrowWithTiming],
    ) -> AccountAgeMetric {
        let now = Utc::now();
        let account_age_days = (now - user.created_at).num_days() as i32;

        // Find first transaction
        let first_loan = loans.iter().map(|l| l.created_at).min();
        let first_escrow = escrows.iter().map(|e| e.created_at).min();
        let first_transaction = match (first_loan, first_escrow) {
            (Some(l), Some(e)) => Some(l.min(e)),
            (Some(l), None) => Some(l),
            (None, Some(e)) => Some(e),
            (None, None) => None,
        };

        let active_period_days = first_transaction
            .map(|ft| (now - ft).num_days() as i32)
            .unwrap_or(0);

        // Score: older accounts with consistent activity score higher
        // Max out at ~2 years (730 days)
        let age_score = ((account_age_days as f64 / 730.0).min(1.0) * 500.0) as i32;
        let activity_score = ((active_period_days as f64 / 365.0).min(1.0) * 500.0) as i32;
        let score = (age_score + activity_score).min(MAX_RISK_SCORE);

        AccountAgeMetric {
            account_created_at: Some(user.created_at),
            first_transaction_at: first_transaction,
            account_age_days,
            active_period_days,
            score,
            weight: WEIGHT_ACCOUNT_AGE,
        }
    }

    fn calculate_consistency_metric(&self, deal_amounts: &DealAmounts) -> ConsistencyMetric {
        if deal_amounts.amounts.is_empty() {
            return ConsistencyMetric {
                average_deal_size: 0,
                deal_size_std_dev: 0.0,
                coefficient_of_variation: 0.0,
                deals_per_month: 0.0,
                score: DEFAULT_NEW_USER_SCORE,
                weight: WEIGHT_DEAL_CONSISTENCY,
            };
        }

        let amounts = &deal_amounts.amounts;
        let n = amounts.len() as f64;

        // Calculate mean
        let sum: i64 = amounts.iter().sum();
        let mean = sum as f64 / n;

        // Calculate standard deviation
        let variance: f64 = amounts
            .iter()
            .map(|&x| {
                let diff = x as f64 - mean;
                diff * diff
            })
            .sum::<f64>()
            / n;
        let std_dev = variance.sqrt();

        // Coefficient of variation
        let cv = if mean > 0.0 { std_dev / mean } else { 0.0 };

        // Calculate deals per month
        let timestamps = &deal_amounts.timestamps;
        let deals_per_month = if timestamps.len() >= 2 {
            let earliest = timestamps.iter().min().unwrap();
            let latest = timestamps.iter().max().unwrap();
            let months = ((*latest - *earliest).num_days() as f64 / 30.0).max(1.0);
            n / months
        } else {
            0.0
        };

        // Score: lower CV = more consistent = higher score
        // CV of 0 = perfect consistency = 1000
        // CV of 2+ = very inconsistent = lower score
        let consistency_score = ((1.0 - (cv / 2.0).min(1.0)) * 700.0) as i32;
        let frequency_score = ((deals_per_month / 5.0).min(1.0) * 300.0) as i32;
        let score = (consistency_score + frequency_score).min(MAX_RISK_SCORE);

        ConsistencyMetric {
            average_deal_size: mean as i64,
            deal_size_std_dev: std_dev,
            coefficient_of_variation: cv,
            deals_per_month,
            score,
            weight: WEIGHT_DEAL_CONSISTENCY,
        }
    }

    async fn detect_fraud_indicators(
        &self,
        loan_stats: &LoanStats,
        escrow_stats: &EscrowStats,
        loans: &[LoanWithTiming],
        escrows: &[EscrowWithTiming],
        account_age: &AccountAgeMetric,
    ) -> Vec<FraudIndicator> {
        let mut indicators = Vec::new();
        let now = Utc::now();

        // 1. High default rate indicator
        let total_loans = loan_stats.total_loans.unwrap_or(0);
        let defaulted = loan_stats.defaulted_count.unwrap_or(0);
        if total_loans >= 3 && defaulted as f64 / total_loans as f64 > 0.3 {
            indicators.push(FraudIndicator {
                indicator_type: FraudIndicatorType::HighDefaultRate,
                severity: FraudSeverity::High,
                description: format!(
                    "Default rate of {:.1}% exceeds threshold",
                    (defaulted as f64 / total_loans as f64) * 100.0
                ),
                detected_at: now,
                score_impact: -150,
            });
        }

        // 2. Repeated disputes indicator
        let disputed = escrow_stats.disputed_count.unwrap_or(0);
        let total_escrows = escrow_stats.total_escrows.unwrap_or(0);
        if total_escrows >= 3 && disputed as f64 / total_escrows as f64 > 0.25 {
            indicators.push(FraudIndicator {
                indicator_type: FraudIndicatorType::RepeatedDisputes,
                severity: FraudSeverity::Medium,
                description: format!(
                    "Dispute rate of {:.1}% is unusually high",
                    (disputed as f64 / total_escrows as f64) * 100.0
                ),
                detected_at: now,
                score_impact: -100,
            });
        }

        // 3. Suspicious account age - new account with high activity
        let total_deals = (total_loans + total_escrows) as i32;
        if account_age.account_age_days < 30 && total_deals > 10 {
            indicators.push(FraudIndicator {
                indicator_type: FraudIndicatorType::SuspiciousAccountAge,
                severity: FraudSeverity::Medium,
                description: format!(
                    "Account is {} days old but has {} deals",
                    account_age.account_age_days, total_deals
                ),
                detected_at: now,
                score_impact: -75,
            });
        }

        // 4. Smurfing pattern detection - many small deals followed by large one
        if let Some(pattern) = self.detect_smurfing_pattern(loans, escrows) {
            indicators.push(FraudIndicator {
                indicator_type: FraudIndicatorType::SmurfingPattern,
                severity: FraudSeverity::High,
                description: pattern,
                detected_at: now,
                score_impact: -200,
            });
        }

        // 5. Anomalous activity - sudden spike in transaction volume
        if let Some(anomaly) = self.detect_anomalous_activity(loans, escrows) {
            indicators.push(FraudIndicator {
                indicator_type: FraudIndicatorType::AnomalousActivity,
                severity: FraudSeverity::Medium,
                description: anomaly,
                detected_at: now,
                score_impact: -50,
            });
        }

        indicators
    }

    fn detect_smurfing_pattern(
        &self,
        loans: &[LoanWithTiming],
        escrows: &[EscrowWithTiming],
    ) -> Option<String> {
        // Combine all deal amounts with timestamps
        let mut deals: Vec<(i64, DateTime<Utc>)> = loans
            .iter()
            .map(|l| (l.principal_amount, l.created_at))
            .chain(escrows.iter().map(|e| (e.amount, e.created_at)))
            .collect();

        if deals.len() < 5 {
            return None;
        }

        // Sort by timestamp
        deals.sort_by_key(|(_, ts)| *ts);

        // Look for pattern: 3+ small deals followed by a large deal
        let amounts: Vec<i64> = deals.iter().map(|(a, _)| *a).collect();
        let avg = amounts.iter().sum::<i64>() as f64 / amounts.len() as f64;

        for window in amounts.windows(4) {
            let small_count = window[..3]
                .iter()
                .filter(|&&a| (a as f64) < avg * 0.3)
                .count();
            let is_large_last = window[3] as f64 > avg * 3.0;

            if small_count >= 2 && is_large_last {
                return Some(
                    "Pattern detected: multiple small transactions followed by large transaction"
                        .to_string(),
                );
            }
        }

        None
    }

    fn detect_anomalous_activity(
        &self,
        loans: &[LoanWithTiming],
        escrows: &[EscrowWithTiming],
    ) -> Option<String> {
        let now = Utc::now();
        let week_ago = now - Duration::days(7);
        let month_ago = now - Duration::days(30);

        // Count recent vs older activity
        let recent_loans = loans
            .iter()
            .filter(|l| l.created_at > week_ago)
            .count();
        let recent_escrows = escrows
            .iter()
            .filter(|e| e.created_at > week_ago)
            .count();
        let recent_total = recent_loans + recent_escrows;

        let older_loans = loans
            .iter()
            .filter(|l| l.created_at <= week_ago && l.created_at > month_ago)
            .count();
        let older_escrows = escrows
            .iter()
            .filter(|e| e.created_at <= week_ago && e.created_at > month_ago)
            .count();
        let older_weekly_avg = (older_loans + older_escrows) as f64 / 3.0; // 3 weeks

        // Flag if recent activity is 5x the average
        if recent_total as f64 > older_weekly_avg * 5.0 && recent_total > 5 {
            return Some(format!(
                "Unusual activity spike: {} transactions in last week vs {:.1} weekly average",
                recent_total, older_weekly_avg
            ));
        }

        None
    }

    fn calculate_overall_score(
        &self,
        deal_count: &DealCountMetric,
        repayment: &RepaymentMetric,
        escrow: &EscrowMetric,
        account_age: &AccountAgeMetric,
        consistency: &ConsistencyMetric,
        fraud_indicators: &[FraudIndicator],
    ) -> (i32, f64) {
        // Weighted sum of all metrics
        let weighted_score = (deal_count.score as f64 * deal_count.weight)
            + (repayment.score as f64 * repayment.weight)
            + (escrow.score as f64 * escrow.weight)
            + (account_age.score as f64 * account_age.weight)
            + (consistency.score as f64 * consistency.weight);

        // Apply fraud penalties
        let fraud_penalty: i32 = fraud_indicators.iter().map(|f| f.score_impact).sum();

        let final_score = (weighted_score as i32 + fraud_penalty)
            .max(MIN_RISK_SCORE)
            .min(MAX_RISK_SCORE);

        // Calculate confidence based on data availability
        let confidence = self.calculate_confidence(deal_count.total_deals);

        (final_score, confidence)
    }

    fn calculate_confidence(&self, total_deals: i32) -> f64 {
        // Confidence increases with more data
        // 0 deals = 0.1 confidence
        // MIN_DEALS_FOR_RELIABLE_SCORE = 0.5 confidence
        // 20+ deals = ~0.95 confidence
        let base = 0.1;
        let growth = 0.85 * (1.0 - (-0.1 * total_deals as f64).exp());
        (base + growth).min(0.99)
    }

    fn generate_summary(
        &self,
        deal_count: &DealCountMetric,
        repayment: &RepaymentMetric,
        escrow: &EscrowMetric,
        fraud_indicators: &[FraudIndicator],
        is_reliable: bool,
    ) -> ScoreSummary {
        let mut positive = Vec::new();
        let mut negative = Vec::new();
        let mut recommendations = Vec::new();

        // Positive factors
        if deal_count.total_deals >= 10 {
            positive.push("Established transaction history".to_string());
        }
        if repayment.ratio > 0.9 && repayment.total_loans > 0 {
            positive.push(format!(
                "Excellent repayment record ({:.0}% on-time)",
                repayment.ratio * 100.0
            ));
        }
        if escrow.completion_ratio > 0.9 && escrow.total_escrows > 0 {
            positive.push(format!(
                "High escrow completion rate ({:.0}%)",
                escrow.completion_ratio * 100.0
            ));
        }
        if escrow.dispute_ratio < 0.05 && escrow.total_escrows > 0 {
            positive.push("Low dispute rate".to_string());
        }

        // Negative factors
        if repayment.defaulted > 0 {
            negative.push(format!("{} loan default(s) on record", repayment.defaulted));
        }
        if escrow.disputed > 0 {
            negative.push(format!("{} disputed escrow(s)", escrow.disputed));
        }
        if escrow.timed_out > 0 {
            negative.push(format!("{} timed-out escrow(s)", escrow.timed_out));
        }
        for indicator in fraud_indicators {
            negative.push(format!(
                "{:?}: {}",
                indicator.indicator_type, indicator.description
            ));
        }

        // Recommendations
        if !is_reliable {
            recommendations.push(
                "Complete more transactions to establish a reliable credit history".to_string(),
            );
        }
        if repayment.ratio < 0.8 && repayment.total_loans > 0 {
            recommendations.push("Focus on timely loan repayments to improve score".to_string());
        }
        if escrow.dispute_ratio > 0.1 {
            recommendations.push("Reduce disputes by ensuring clear terms in escrows".to_string());
        }
        if deal_count.total_deals < 5 {
            recommendations.push(
                "Build transaction history with smaller, successful deals first".to_string(),
            );
        }

        ScoreSummary {
            positive_factors: positive,
            negative_factors: negative,
            recommendations,
        }
    }

    fn create_unscored_response(&self, wallet_address: &str) -> RiskScoreResponse {
        RiskScoreResponse {
            wallet_address: wallet_address.to_string(),
            overall_score: DEFAULT_NEW_USER_SCORE,
            risk_tier: RiskTier::Unscored,
            metrics: RiskMetrics {
                deal_count: DealCountMetric {
                    total_deals: 0,
                    as_borrower: 0,
                    as_lender: 0,
                    as_buyer: 0,
                    as_seller: 0,
                    score: 0,
                    weight: WEIGHT_DEAL_COUNT,
                },
                repayment_ratio: RepaymentMetric {
                    total_loans: 0,
                    repaid_on_time: 0,
                    repaid_late: 0,
                    defaulted: 0,
                    active: 0,
                    ratio: 1.0,
                    time_decayed_ratio: 1.0,
                    score: DEFAULT_NEW_USER_SCORE,
                    weight: WEIGHT_REPAYMENT_RATIO,
                },
                escrow_completion: EscrowMetric {
                    total_escrows: 0,
                    completed_successfully: 0,
                    cancelled: 0,
                    disputed: 0,
                    timed_out: 0,
                    completion_ratio: 1.0,
                    dispute_ratio: 0.0,
                    score: DEFAULT_NEW_USER_SCORE,
                    weight: WEIGHT_ESCROW_COMPLETION,
                },
                account_age: AccountAgeMetric {
                    account_created_at: None,
                    first_transaction_at: None,
                    account_age_days: 0,
                    active_period_days: 0,
                    score: 0,
                    weight: WEIGHT_ACCOUNT_AGE,
                },
                deal_consistency: ConsistencyMetric {
                    average_deal_size: 0,
                    deal_size_std_dev: 0.0,
                    coefficient_of_variation: 0.0,
                    deals_per_month: 0.0,
                    score: DEFAULT_NEW_USER_SCORE,
                    weight: WEIGHT_DEAL_CONSISTENCY,
                },
            },
            fraud_indicators: vec![],
            confidence: 0.1,
            is_reliable: false,
            calculated_at: Utc::now(),
            summary: ScoreSummary {
                positive_factors: vec![],
                negative_factors: vec![],
                recommendations: vec![
                    "No transaction history found for this wallet".to_string(),
                    "Start building credit history with small, successful transactions".to_string(),
                ],
            },
        }
    }

    async fn calculate_score_at_point_in_time(
        &self,
        user_id: Uuid,
        point_in_time: DateTime<Utc>,
    ) -> Result<i32, ApiError> {
        // Get stats up to the point in time
        let loan_stats = sqlx::query_as::<_, LoanStats>(
            r#"
            SELECT 
                COUNT(*) as total_loans,
                COUNT(*) FILTER (WHERE status = 'repaid' AND updated_at <= $2) as repaid_count,
                COUNT(*) FILTER (WHERE (status = 'defaulted' OR status = 'liquidated') AND updated_at <= $2) as defaulted_count,
                COUNT(*) FILTER (WHERE status = 'active' AND created_at <= $2) as active_count,
                COALESCE(SUM(principal_amount), 0) as total_principal,
                COALESCE(SUM(principal_amount) FILTER (WHERE status = 'repaid' AND updated_at <= $2), 0) as total_repaid_amount
            FROM loans
            WHERE (borrower_id = $1 OR lender_id = $1) AND created_at <= $2
            "#,
        )
        .bind(user_id)
        .bind(point_in_time)
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        let total_loans = loan_stats.total_loans.unwrap_or(0);
        let repaid = loan_stats.repaid_count.unwrap_or(0);
        let defaulted = loan_stats.defaulted_count.unwrap_or(0);

        // Simplified score calculation for historical points
        if total_loans == 0 {
            return Ok(DEFAULT_NEW_USER_SCORE);
        }

        let completed = repaid + defaulted;
        let ratio = if completed > 0 {
            repaid as f64 / completed as f64
        } else {
            1.0
        };

        let score = (ratio * MAX_RISK_SCORE as f64) as i32;
        Ok(score.max(MIN_RISK_SCORE).min(MAX_RISK_SCORE))
    }

    fn apply_scenario_to_score(
        &self,
        current: &RiskScoreResponse,
        scenario: &SimulationScenario,
    ) -> i32 {
        let mut projected = current.overall_score;

        match scenario {
            SimulationScenario::SuccessfulLoanRepayment { amount } => {
                // Positive impact based on amount
                let impact = ((*amount as f64 / 1_000_000.0) * 10.0).min(50.0) as i32;
                projected = (projected + impact).min(MAX_RISK_SCORE);
            }
            SimulationScenario::LoanDefault { amount } => {
                // Significant negative impact
                let impact = ((*amount as f64 / 1_000_000.0) * 50.0).min(200.0) as i32;
                projected = (projected - impact).max(MIN_RISK_SCORE);
            }
            SimulationScenario::SuccessfulEscrow { amount } => {
                let impact = ((*amount as f64 / 1_000_000.0) * 5.0).min(25.0) as i32;
                projected = (projected + impact).min(MAX_RISK_SCORE);
            }
            SimulationScenario::DisputedEscrow => {
                projected = (projected - 50).max(MIN_RISK_SCORE);
            }
            SimulationScenario::MultipleSuccessfulDeals { count } => {
                let impact = (*count as i32 * 15).min(100);
                projected = (projected + impact).min(MAX_RISK_SCORE);
            }
        }

        projected
    }

    fn generate_scenario_recommendations(
        &self,
        scenario: &SimulationScenario,
        projected_score: i32,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();
        let tier = RiskTier::from_score(projected_score);

        match scenario {
            SimulationScenario::LoanDefault { .. } => {
                recommendations.push(
                    "Avoid defaults by only borrowing amounts you can confidently repay"
                        .to_string(),
                );
                recommendations
                    .push("Consider smaller loan amounts to reduce default risk".to_string());
            }
            SimulationScenario::DisputedEscrow => {
                recommendations
                    .push("Ensure clear terms and documentation in all escrow agreements".to_string());
                recommendations
                    .push("Communicate proactively with counterparties".to_string());
            }
            _ => {}
        }

        match tier {
            RiskTier::Excellent | RiskTier::Good => {
                recommendations.push("Maintain current behavior to preserve excellent standing".to_string());
            }
            RiskTier::Fair | RiskTier::Poor => {
                recommendations.push("Focus on completing transactions successfully to improve score".to_string());
            }
            RiskTier::HighRisk => {
                recommendations.push("Multiple successful small transactions are recommended to rebuild trust".to_string());
            }
            RiskTier::Unscored => {
                recommendations.push("Begin building transaction history with low-risk deals".to_string());
            }
        }

        recommendations
    }
}

// ============================================================================
// Simulation and Backtesting Types
// ============================================================================

/// Historical score data point
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HistoricalScore {
    pub date: DateTime<Utc>,
    pub score: i32,
    pub tier: RiskTier,
}

/// Simulation scenario types
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SimulationScenario {
    /// Simulate a successful loan repayment
    SuccessfulLoanRepayment { amount: i64 },
    /// Simulate a loan default
    LoanDefault { amount: i64 },
    /// Simulate a successful escrow completion
    SuccessfulEscrow { amount: i64 },
    /// Simulate a disputed escrow
    DisputedEscrow,
    /// Simulate multiple successful deals
    MultipleSuccessfulDeals { count: u32 },
}

impl SimulationScenario {
    pub fn description(&self) -> String {
        match self {
            SimulationScenario::SuccessfulLoanRepayment { amount } => {
                format!("Successful repayment of {} stroops", amount)
            }
            SimulationScenario::LoanDefault { amount } => {
                format!("Default on loan of {} stroops", amount)
            }
            SimulationScenario::SuccessfulEscrow { amount } => {
                format!("Successful escrow completion of {} stroops", amount)
            }
            SimulationScenario::DisputedEscrow => "Disputed escrow resolution".to_string(),
            SimulationScenario::MultipleSuccessfulDeals { count } => {
                format!("Complete {} successful deals", count)
            }
        }
    }
}

/// Result of a score simulation
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SimulationResult {
    pub current_score: i32,
    pub projected_score: i32,
    pub score_change: i32,
    pub scenario_description: String,
    pub recommendations: Vec<String>,
}

// ============================================================================
// Query Parameters
// ============================================================================

/// Query parameters for historical scores
#[derive(Debug, Deserialize)]
pub struct HistoricalScoreQuery {
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_tier_from_score() {
        assert_eq!(RiskTier::from_score(900), RiskTier::Excellent);
        assert_eq!(RiskTier::from_score(800), RiskTier::Excellent);
        assert_eq!(RiskTier::from_score(750), RiskTier::Good);
        assert_eq!(RiskTier::from_score(650), RiskTier::Good);
        assert_eq!(RiskTier::from_score(550), RiskTier::Fair);
        assert_eq!(RiskTier::from_score(400), RiskTier::Poor);
        assert_eq!(RiskTier::from_score(200), RiskTier::HighRisk);
        assert_eq!(RiskTier::from_score(0), RiskTier::HighRisk);
    }

    #[test]
    fn test_confidence_calculation() {
        let engine = RiskEngine {
            db_pool: sqlx::PgPool::connect_lazy("postgres://localhost/test").unwrap(),
        };

        let conf_0 = engine.calculate_confidence(0);
        let conf_5 = engine.calculate_confidence(5);
        let conf_20 = engine.calculate_confidence(20);

        assert!(conf_0 < conf_5);
        assert!(conf_5 < conf_20);
        assert!(conf_0 >= 0.1);
        assert!(conf_20 < 1.0);
    }

    #[test]
    fn test_simulation_scenario_description() {
        let scenario = SimulationScenario::SuccessfulLoanRepayment { amount: 1_000_000 };
        assert!(scenario.description().contains("1000000"));

        let scenario = SimulationScenario::DisputedEscrow;
        assert!(scenario.description().contains("Disputed"));
    }

    #[test]
    fn test_weights_sum_to_one() {
        let total = WEIGHT_DEAL_COUNT
            + WEIGHT_REPAYMENT_RATIO
            + WEIGHT_ESCROW_COMPLETION
            + WEIGHT_ACCOUNT_AGE
            + WEIGHT_DEAL_CONSISTENCY;
        assert!((total - 1.0).abs() < 0.001, "Weights should sum to 1.0");
    }
}

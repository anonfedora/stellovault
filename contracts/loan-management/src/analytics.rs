//! Loan portfolio analytics module

use soroban_sdk::contracttype;

/// Portfolio analytics
#[contracttype]
#[derive(Clone)]
pub struct PortfolioAnalytics {
    pub total_loans: u64,
    pub total_principal: i128,
    pub total_outstanding: i128,
    pub total_interest_paid: i128,
    pub active_loans: u64,
    pub defaulted_loans: u64,
    pub paid_loans: u64,
    pub average_interest_rate: u32,
    pub portfolio_ltv: u32,
}

/// Loan performance metrics
#[contracttype]
#[derive(Clone)]
pub struct PerformanceMetrics {
    pub loan_id: u64,
    pub on_time_payments: u32,
    pub late_payments: u32,
    pub missed_payments: u32,
    pub payment_history_score: u32,
    pub days_past_due: u64,
    pub delinquency_rate: u32,
}

/// Risk metrics
#[contracttype]
#[derive(Clone)]
pub struct RiskMetrics {
    pub portfolio_risk_score: u32,
    pub default_probability: u32,
    pub loss_given_default: u32,
    pub expected_loss: i128,
    pub value_at_risk: i128,
}

/// Cohort analysis
#[contracttype]
#[derive(Clone)]
pub struct CohortAnalysis {
    pub cohort_id: u64,
    pub origination_date: u64,
    pub loan_count: u64,
    pub total_principal: i128,
    pub cumulative_default_rate: u32,
    pub cumulative_loss_rate: u32,
}

/// Loan performance trend
#[contracttype]
#[derive(Clone)]
pub struct PerformanceTrend {
    pub period: u64,
    pub average_payment_rate: u32,
    pub average_delinquency_rate: u32,
    pub default_rate: u32,
    pub prepayment_rate: u32,
}

/// Pricing analysis
#[contracttype]
#[derive(Clone)]
pub struct PricingAnalysis {
    pub average_rate: u32,
    pub rate_distribution: String,
    pub risk_adjusted_return: u32,
    pub spread_analysis: String,
}

/// Collateral analysis
#[contracttype]
#[derive(Clone)]
pub struct CollateralAnalysis {
    pub total_collateral_value: i128,
    pub average_ltv: u32,
    pub collateral_concentration: u32,
    pub collateral_type_distribution: String,
}

/// Borrower analysis
#[contracttype]
#[derive(Clone)]
pub struct BorrowerAnalysis {
    pub total_borrowers: u64,
    pub average_loan_size: i128,
    pub borrower_concentration: u32,
    pub repeat_borrower_rate: u32,
}

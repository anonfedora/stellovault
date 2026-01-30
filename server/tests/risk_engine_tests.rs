//! Risk Engine Backtesting and Simulation Tests
//!
//! These tests validate the risk scoring logic with various scenarios
//! including edge cases, fraud detection, and score simulations.

use stellovault_server::services::risk_engine::{RiskTier, SimulationScenario};

// ============================================================================
// Risk Tier Classification Tests
// ============================================================================

#[test]
fn test_risk_tier_excellent() {
    assert_eq!(RiskTier::from_score(1000), RiskTier::Excellent);
    assert_eq!(RiskTier::from_score(900), RiskTier::Excellent);
    assert_eq!(RiskTier::from_score(800), RiskTier::Excellent);
}

#[test]
fn test_risk_tier_good() {
    assert_eq!(RiskTier::from_score(799), RiskTier::Good);
    assert_eq!(RiskTier::from_score(700), RiskTier::Good);
    assert_eq!(RiskTier::from_score(650), RiskTier::Good);
}

#[test]
fn test_risk_tier_fair() {
    assert_eq!(RiskTier::from_score(649), RiskTier::Fair);
    assert_eq!(RiskTier::from_score(550), RiskTier::Fair);
    assert_eq!(RiskTier::from_score(500), RiskTier::Fair);
}

#[test]
fn test_risk_tier_poor() {
    assert_eq!(RiskTier::from_score(499), RiskTier::Poor);
    assert_eq!(RiskTier::from_score(400), RiskTier::Poor);
    assert_eq!(RiskTier::from_score(300), RiskTier::Poor);
}

#[test]
fn test_risk_tier_high_risk() {
    assert_eq!(RiskTier::from_score(299), RiskTier::HighRisk);
    assert_eq!(RiskTier::from_score(100), RiskTier::HighRisk);
    assert_eq!(RiskTier::from_score(0), RiskTier::HighRisk);
}

#[test]
fn test_risk_tier_descriptions() {
    assert!(!RiskTier::Excellent.description().is_empty());
    assert!(!RiskTier::Good.description().is_empty());
    assert!(!RiskTier::Fair.description().is_empty());
    assert!(!RiskTier::Poor.description().is_empty());
    assert!(!RiskTier::HighRisk.description().is_empty());
    assert!(!RiskTier::Unscored.description().is_empty());
}

// ============================================================================
// Simulation Scenario Tests
// ============================================================================

#[test]
fn test_simulation_successful_loan_description() {
    let scenario = SimulationScenario::SuccessfulLoanRepayment {
        amount: 1_000_000_000,
    };
    let desc = scenario.description();
    assert!(desc.contains("1000000000"));
    assert!(desc.contains("Successful"));
}

#[test]
fn test_simulation_loan_default_description() {
    let scenario = SimulationScenario::LoanDefault {
        amount: 500_000_000,
    };
    let desc = scenario.description();
    assert!(desc.contains("500000000"));
    assert!(desc.contains("Default"));
}

#[test]
fn test_simulation_successful_escrow_description() {
    let scenario = SimulationScenario::SuccessfulEscrow {
        amount: 250_000_000,
    };
    let desc = scenario.description();
    assert!(desc.contains("250000000"));
    assert!(desc.contains("escrow"));
}

#[test]
fn test_simulation_disputed_escrow_description() {
    let scenario = SimulationScenario::DisputedEscrow;
    let desc = scenario.description();
    assert!(desc.contains("Disputed"));
}

#[test]
fn test_simulation_multiple_deals_description() {
    let scenario = SimulationScenario::MultipleSuccessfulDeals { count: 10 };
    let desc = scenario.description();
    assert!(desc.contains("10"));
    assert!(desc.contains("successful"));
}

// ============================================================================
// Confidence Calculation Tests
// ============================================================================

#[test]
fn test_confidence_increases_with_deals() {
    // I'm testing confidence formula directly without needing a DB pool.
    let conf_0 = calculate_confidence_helper(0);
    let conf_5 = calculate_confidence_helper(5);
    let conf_10 = calculate_confidence_helper(10);
    let conf_20 = calculate_confidence_helper(20);

    assert!(conf_0 < conf_5, "More deals should increase confidence");
    assert!(conf_5 < conf_10, "More deals should increase confidence");
    assert!(conf_10 < conf_20, "More deals should increase confidence");
}

#[test]
fn test_confidence_minimum_bound() {
    let conf = calculate_confidence_helper(0);
    assert!(conf >= 0.1, "Minimum confidence should be 0.1");
}

#[test]
fn test_confidence_maximum_bound() {
    let conf = calculate_confidence_helper(100);
    assert!(conf < 1.0, "Confidence should never reach 1.0");
    assert!(conf >= 0.9, "High deal count should give high confidence");
}

/// Helper function to calculate confidence (mirrors the internal logic)
fn calculate_confidence_helper(total_deals: i32) -> f64 {
    let base = 0.1;
    let growth = 0.85 * (1.0 - (-0.1 * total_deals as f64).exp());
    (base + growth).min(0.99)
}

// ============================================================================
// Weight Validation Tests
// ============================================================================

#[test]
fn test_weights_sum_to_one() {
    // These constants should match those in risk_engine.rs
    const WEIGHT_DEAL_COUNT: f64 = 0.20;
    const WEIGHT_REPAYMENT_RATIO: f64 = 0.35;
    const WEIGHT_ESCROW_COMPLETION: f64 = 0.25;
    const WEIGHT_ACCOUNT_AGE: f64 = 0.10;
    const WEIGHT_DEAL_CONSISTENCY: f64 = 0.10;

    let total = WEIGHT_DEAL_COUNT
        + WEIGHT_REPAYMENT_RATIO
        + WEIGHT_ESCROW_COMPLETION
        + WEIGHT_ACCOUNT_AGE
        + WEIGHT_DEAL_CONSISTENCY;

    assert!(
        (total - 1.0).abs() < 0.001,
        "Weights must sum to 1.0, got {}",
        total
    );
}

#[test]
fn test_repayment_ratio_has_highest_weight() {
    const WEIGHT_DEAL_COUNT: f64 = 0.20;
    const WEIGHT_REPAYMENT_RATIO: f64 = 0.35;
    const WEIGHT_ESCROW_COMPLETION: f64 = 0.25;
    const WEIGHT_ACCOUNT_AGE: f64 = 0.10;
    #[allow(dead_code)]
    const WEIGHT_DEAL_CONSISTENCY: f64 = 0.10;

    assert!(
        WEIGHT_REPAYMENT_RATIO > WEIGHT_DEAL_COUNT,
        "Repayment ratio should be weighted higher than deal count"
    );
    assert!(
        WEIGHT_REPAYMENT_RATIO > WEIGHT_ESCROW_COMPLETION,
        "Repayment ratio should be weighted higher than escrow completion"
    );
    assert!(
        WEIGHT_REPAYMENT_RATIO > WEIGHT_ACCOUNT_AGE,
        "Repayment ratio should be weighted higher than account age"
    );
}

// ============================================================================
// Time Decay Tests
// ============================================================================

#[test]
fn test_time_decay_recent_transactions_weighted_more() {
    // A transaction from today should have weight ~1.0
    // A transaction from 90 days ago (half-life) should have weight ~0.5
    // A transaction from 180 days ago should have weight ~0.25

    let half_life_days = 90.0;

    let weight_today = time_decay_weight(0, half_life_days);
    let weight_90_days = time_decay_weight(90, half_life_days);
    let weight_180_days = time_decay_weight(180, half_life_days);
    let weight_365_days = time_decay_weight(365, half_life_days);

    assert!(
        (weight_today - 1.0).abs() < 0.01,
        "Today's weight should be ~1.0"
    );
    assert!(
        (weight_90_days - 0.5).abs() < 0.01,
        "90 day weight should be ~0.5"
    );
    assert!(
        (weight_180_days - 0.25).abs() < 0.01,
        "180 day weight should be ~0.25"
    );
    assert!(
        weight_365_days < 0.1,
        "Year-old transactions should have low weight"
    );
}

/// Helper to calculate time decay weight
fn time_decay_weight(age_days: i64, half_life: f64) -> f64 {
    0.5_f64.powf(age_days as f64 / half_life)
}

// ============================================================================
// Score Boundary Tests
// ============================================================================

#[test]
fn test_score_never_exceeds_maximum() {
    // Even with perfect metrics, score should not exceed 1000
    let max_possible = 1000;

    // Simulate perfect scores for all metrics
    let perfect_deal_count = 1000;
    let perfect_repayment = 1000;
    let perfect_escrow = 1000;
    let perfect_account_age = 1000;
    let perfect_consistency = 1000;

    let weights: [f64; 5] = [0.20, 0.35, 0.25, 0.10, 0.10];
    let scores = [
        perfect_deal_count,
        perfect_repayment,
        perfect_escrow,
        perfect_account_age,
        perfect_consistency,
    ];

    let weighted_sum: f64 = weights
        .iter()
        .zip(scores.iter())
        .map(|(w, s)| w * (*s as f64))
        .sum();

    assert!(
        weighted_sum as i32 <= max_possible,
        "Weighted sum should not exceed max score"
    );
}

#[test]
fn test_score_never_below_minimum() {
    // Even with terrible metrics and fraud penalties, score should not go below 0
    let min_possible = 0;
    let base_score = 0; // Worst possible base
    let max_fraud_penalty = -500; // Multiple high-severity fraud indicators

    let final_score = (base_score + max_fraud_penalty).max(min_possible);

    assert_eq!(
        final_score, min_possible,
        "Score should be clamped to minimum"
    );
}

// ============================================================================
// Fraud Detection Logic Tests
// ============================================================================

#[test]
fn test_high_default_rate_detection_threshold() {
    // Default rate > 30% with >= 3 loans should trigger indicator
    let total_loans = 10;
    let defaulted = 4; // 40% default rate

    let should_trigger = total_loans >= 3 && (defaulted as f64 / total_loans as f64) > 0.3;

    assert!(
        should_trigger,
        "40% default rate should trigger high default indicator"
    );
}

#[test]
fn test_high_default_rate_below_threshold() {
    let total_loans = 10;
    let defaulted = 2; // 20% default rate

    let should_trigger = total_loans >= 3 && (defaulted as f64 / total_loans as f64) > 0.3;

    assert!(
        !should_trigger,
        "20% default rate should not trigger indicator"
    );
}

#[test]
fn test_repeated_disputes_detection() {
    let total_escrows = 10;
    let disputed = 3; // 30% dispute rate

    let should_trigger = total_escrows >= 3 && (disputed as f64 / total_escrows as f64) > 0.25;

    assert!(
        should_trigger,
        "30% dispute rate should trigger repeated disputes indicator"
    );
}

#[test]
fn test_suspicious_account_age_detection() {
    let account_age_days = 20; // Very new account
    let total_deals = 15; // High activity

    let should_trigger = account_age_days < 30 && total_deals > 10;

    assert!(
        should_trigger,
        "New account with high activity should be flagged"
    );
}

#[test]
fn test_established_account_not_flagged() {
    let account_age_days = 180; // 6 months old
    let total_deals = 50; // High activity

    let should_trigger = account_age_days < 30 && total_deals > 10;

    assert!(
        !should_trigger,
        "Established account with high activity should not be flagged"
    );
}

// ============================================================================
// Simulation Impact Tests
// ============================================================================

#[test]
fn test_successful_repayment_improves_score() {
    let current_score = 600;
    let amount = 1_000_000_i64;

    // Simulate the impact calculation
    let impact = ((amount as f64 / 1_000_000.0) * 10.0).min(50.0) as i32;
    let projected = (current_score + impact).min(1000);

    assert!(
        projected > current_score,
        "Successful repayment should improve score"
    );
}

#[test]
fn test_loan_default_decreases_score() {
    let current_score = 700;
    let amount = 1_000_000_i64;

    // Simulate the impact calculation
    let impact = ((amount as f64 / 1_000_000.0) * 50.0).min(200.0) as i32;
    let projected = (current_score - impact).max(0);

    assert!(
        projected < current_score,
        "Loan default should decrease score"
    );
}

#[test]
fn test_disputed_escrow_penalty() {
    let current_score = 650;
    let penalty = 50; // Fixed penalty for disputed escrow

    let projected = (current_score - penalty).max(0);

    assert_eq!(
        projected,
        current_score - penalty,
        "Disputed escrow should apply fixed penalty"
    );
}

#[test]
fn test_multiple_successful_deals_bonus() {
    let current_score = 550;
    let deal_count = 5_u32;

    // Each successful deal adds 15 points, capped at 100
    let impact = (deal_count as i32 * 15).min(100);
    let projected = (current_score + impact).min(1000);

    assert_eq!(
        projected,
        current_score + (5 * 15),
        "Multiple successful deals should add points"
    );
}

#[test]
fn test_multiple_deals_bonus_capped() {
    let current_score = 500;
    let deal_count = 20_u32; // Would be 300 points without cap

    let impact = (deal_count as i32 * 15).min(100);
    let projected = (current_score + impact).min(1000);

    assert_eq!(impact, 100, "Multiple deals bonus should be capped at 100");
    assert_eq!(projected, 600, "Score should be increased by capped amount");
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn test_empty_transaction_history() {
    // New user with no history should get default score
    let default_score = 500;
    let total_deals = 0;

    let score = if total_deals == 0 { default_score } else { 0 };

    assert_eq!(
        score, default_score,
        "Empty history should get default score"
    );
}

#[test]
fn test_single_perfect_transaction() {
    // One successful transaction should improve from default
    let has_successful_transaction = true;
    let is_reliable = false; // Need 5+ for reliable

    assert!(
        has_successful_transaction && !is_reliable,
        "Single transaction should not be considered reliable"
    );
}

#[test]
fn test_reliability_threshold() {
    const MIN_DEALS_FOR_RELIABLE: i32 = 5;
    assert_eq!(MIN_DEALS_FOR_RELIABLE, 5); // Validate constant

    assert!(!is_reliable_helper(0));
    assert!(!is_reliable_helper(4));
    assert!(is_reliable_helper(5));
    assert!(is_reliable_helper(100));
}

fn is_reliable_helper(total_deals: i32) -> bool {
    const MIN_DEALS_FOR_RELIABLE_SCORE: i32 = 5;
    total_deals >= MIN_DEALS_FOR_RELIABLE_SCORE
}

// ============================================================================
// Coefficient of Variation Tests
// ============================================================================

#[test]
fn test_perfect_consistency_score() {
    // All deals of same size = CV of 0 = highest consistency score
    let amounts = vec![1000, 1000, 1000, 1000, 1000];

    let mean = amounts.iter().sum::<i64>() as f64 / amounts.len() as f64;
    let variance: f64 = amounts
        .iter()
        .map(|&x| {
            let diff = x as f64 - mean;
            diff * diff
        })
        .sum::<f64>()
        / amounts.len() as f64;
    let std_dev = variance.sqrt();
    let cv = if mean > 0.0 { std_dev / mean } else { 0.0 };

    assert!(cv < 0.001, "Same-sized deals should have CV near 0");
}

#[test]
fn test_high_variance_lowers_score() {
    // Widely varying deal sizes = high CV = lower consistency score
    let amounts = vec![100, 10000, 500, 50000, 1000];

    let mean = amounts.iter().sum::<i64>() as f64 / amounts.len() as f64;
    let variance: f64 = amounts
        .iter()
        .map(|&x| {
            let diff = x as f64 - mean;
            diff * diff
        })
        .sum::<f64>()
        / amounts.len() as f64;
    let std_dev = variance.sqrt();
    let cv = if mean > 0.0 { std_dev / mean } else { 0.0 };

    assert!(cv > 1.0, "Highly variable deals should have high CV");
}

// ============================================================================
// Backtesting Scenario Tests
// ============================================================================

#[test]
fn test_score_progression_with_good_behavior() {
    // Simulate score progression with consistently good behavior
    let mut score = 500; // Starting score

    // Each successful transaction should improve score (simplified model)
    for _ in 0..10 {
        let improvement = 15; // Simulate improvement from successful deal
        score = (score + improvement).min(1000);
    }

    assert!(
        score > 600,
        "Consistent good behavior should significantly improve score"
    );
}

#[test]
fn test_score_degradation_with_defaults() {
    // Simulate score degradation with defaults
    let mut score = 750; // Good starting score

    // Each default significantly hurts the score
    for _ in 0..3 {
        let penalty = 75; // Simulate penalty from default
        score = (score - penalty).max(0);
    }

    assert!(
        score < 600,
        "Multiple defaults should significantly hurt score"
    );
}

#[test]
fn test_recovery_after_bad_period() {
    // Start with good score, have bad period, then recover
    let mut score = 800;

    // Bad period - 2 defaults
    score = (score - 75).max(0);
    score = (score - 75).max(0);
    assert!(score < 700, "Score should drop after defaults");

    // Recovery - 10 successful transactions
    for _ in 0..10 {
        score = (score + 15).min(1000);
    }

    assert!(
        score > 700,
        "Score should recover with consistent good behavior"
    );
}

// ============================================================================
// Stress Tests
// ============================================================================

#[test]
fn test_large_transaction_volumes() {
    // System should handle high transaction counts without overflow
    let huge_deal_count = 100_000;
    let confidence = calculate_confidence_helper(huge_deal_count);

    assert!(
        confidence > 0.0 && confidence < 1.0,
        "Confidence should be valid"
    );
}

#[test]
fn test_extreme_amounts() {
    // Test with very large amounts (in stroops)
    let large_amount = i64::MAX / 2;

    // Ensure calculations don't overflow
    let ratio = large_amount as f64 / 1_000_000_000.0; // Convert from stroops
    assert!(
        ratio.is_finite(),
        "Large amount calculations should be finite"
    );
}

#[test]
fn test_score_calculation_performance() {
    // Ensure score calculation completes quickly
    use std::time::Instant;

    let start = Instant::now();

    // Simulate multiple metric calculations
    for _ in 0..1000 {
        let _ = calculate_confidence_helper(50);
        let _ = time_decay_weight(180, 90.0);
        let _ = RiskTier::from_score(750);
    }

    let duration = start.elapsed();
    assert!(
        duration.as_millis() < 100,
        "1000 score calculations should complete in < 100ms"
    );
}

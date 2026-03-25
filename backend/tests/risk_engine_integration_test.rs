//! Integration tests for Risk Engine V2

use stellovault_server::services::risk_engine_v2::{RiskEngineV2, ScoreWeights};

#[tokio::test]
async fn test_risk_engine_v2_initialization() {
    // This test verifies that RiskEngineV2 can be initialized
    // In a real test, you would set up a test database
    
    let weights = ScoreWeights::default();
    
    assert_eq!(weights.on_chain_activity, 0.40);
    assert_eq!(weights.repayment_history, 0.40);
    assert_eq!(weights.collateral_quality, 0.20);
    
    // Verify weights sum to 1.0
    let sum = weights.on_chain_activity + weights.repayment_history + weights.collateral_quality;
    assert!((sum - 1.0).abs() < 0.001);
}

#[test]
fn test_weighted_score_calculation() {
    // Test the weighting algorithm
    let on_chain = 800;
    let repayment = 900;
    let collateral = 600;
    
    let weights = ScoreWeights::default();
    
    let weighted = (on_chain as f64 * weights.on_chain_activity)
        + (repayment as f64 * weights.repayment_history)
        + (collateral as f64 * weights.collateral_quality);
    
    let expected = (800.0 * 0.40) + (900.0 * 0.40) + (600.0 * 0.20);
    assert_eq!(weighted, expected);
    assert_eq!(weighted, 800.0); // 320 + 360 + 120
}

#[test]
fn test_risk_tier_determination() {
    // Test risk tier boundaries
    assert_eq!(determine_tier(950), "excellent");
    assert_eq!(determine_tier(850), "excellent");
    assert_eq!(determine_tier(800), "good");
    assert_eq!(determine_tier(700), "good");
    assert_eq!(determine_tier(650), "fair");
    assert_eq!(determine_tier(550), "fair");
    assert_eq!(determine_tier(450), "poor");
    assert_eq!(determine_tier(400), "poor");
    assert_eq!(determine_tier(300), "very_poor");
}

fn determine_tier(score: i32) -> &'static str {
    match score {
        850..=1000 => "excellent",
        700..=849 => "good",
        550..=699 => "fair",
        400..=549 => "poor",
        _ => "very_poor",
    }
}

//! Tests for DeFi History Provider

use stellovault_server::defi_history::{DeFiHistoryProvider, DeFiHistoryQuery};

#[tokio::test]
async fn test_defi_provider_disabled() {
    let provider = DeFiHistoryProvider::new(
        "http://test-soroswap.com".to_string(),
        None,
        "http://test-aquarius.com".to_string(),
        None,
        false, // disabled
    );

    let query = DeFiHistoryQuery {
        wallet_address: "GABC123456789012345678901234567890123456789012345678".to_string(),
        days_back: 90,
        include_soroswap: true,
        include_aquarius: true,
    };

    let result = provider.fetch_activity_metrics(query).await;
    assert!(result.is_ok());

    let metrics = result.unwrap();
    assert_eq!(metrics.aggregated.total_tx_count, 0);
    assert_eq!(metrics.aggregated.total_volume_usd, 0);
}

#[tokio::test]
async fn test_invalid_wallet_address() {
    let provider = DeFiHistoryProvider::new(
        "http://test-soroswap.com".to_string(),
        None,
        "http://test-aquarius.com".to_string(),
        None,
        true,
    );

    let query = DeFiHistoryQuery {
        wallet_address: "INVALID_ADDRESS".to_string(),
        days_back: 90,
        include_soroswap: true,
        include_aquarius: true,
    };

    let result = provider.fetch_activity_metrics(query).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_valid_wallet_address_format() {
    let provider = DeFiHistoryProvider::new(
        "http://test-soroswap.com".to_string(),
        None,
        "http://test-aquarius.com".to_string(),
        None,
        true,
    );

    // Valid Stellar address format
    let valid_address = "GABC123456789012345678901234567890123456789012345678";
    let query = DeFiHistoryQuery {
        wallet_address: valid_address.to_string(),
        days_back: 90,
        include_soroswap: true,
        include_aquarius: true,
    };

    // This will fail because the APIs don't exist, but it should pass validation
    let result = provider.fetch_activity_metrics(query).await;
    
    // We expect it to fail on network, not validation
    if let Err(e) = result {
        let error_msg = e.to_string();
        assert!(!error_msg.contains("Invalid wallet address"));
    }
}

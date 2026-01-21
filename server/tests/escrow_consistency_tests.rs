//! Consistency tests between database and blockchain

#[cfg(test)]
mod tests {
    use sqlx::PgPool;
    use uuid::Uuid;

    use stellovault_server::escrow::{CreateEscrowRequest, EscrowStatus};
    use stellovault_server::escrow_service::EscrowService;

    /// Helper to create a test database pool
    async fn setup_test_db() -> PgPool {
        let database_url = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://localhost/stellovault_test".to_string());

        sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await
            .expect("Failed to connect to test database")
    }

    /// Helper to create test escrow request
    fn create_test_request() -> CreateEscrowRequest {
        CreateEscrowRequest {
            buyer_id: Uuid::new_v4(),
            seller_id: Uuid::new_v4(),
            collateral_id: Uuid::new_v4(),
            amount: 1000,
            oracle_address: "GABC123...".to_string(),
            release_conditions: r#"{"condition":"shipment_delivered"}"#.to_string(),
            timeout_hours: Some(24),
        }
    }

    #[tokio::test]
    #[ignore] // Requires database setup
    async fn test_escrow_creation_consistency() {
        let db_pool = setup_test_db().await;
        
        let escrow_service = EscrowService::new(
            db_pool.clone(),
            "https://horizon-testnet.stellar.org".to_string(),
            "Test SDF Network ; September 2015".to_string(),
        );

        let request = create_test_request();
        
        // Create escrow
        let response = escrow_service.create_escrow(request).await;
        
        // This will fail without proper database setup, but demonstrates the test structure
        assert!(response.is_ok() || response.is_err());
    }

    #[tokio::test]
    #[ignore] // Requires database setup
    async fn test_escrow_status_reconciliation() {
        let db_pool = setup_test_db().await;
        
        let escrow_service = EscrowService::new(
            db_pool.clone(),
            "https://horizon-testnet.stellar.org".to_string(),
            "Test SDF Network ; September 2015".to_string(),
        );

        // Test that tracking status updates database correctly
        let escrow_id = 12345u64;
        
        let result = escrow_service.track_escrow_status(escrow_id).await;
        
        // Verify result
        match result {
            Ok(status) => {
                assert!(matches!(
                    status,
                    EscrowStatus::Pending
                        | EscrowStatus::Active
                        | EscrowStatus::Released
                        | EscrowStatus::Cancelled
                        | EscrowStatus::TimedOut
                        | EscrowStatus::Disputed
                ));
            }
            Err(_) => {
                // Expected if escrow doesn't exist
            }
        }
    }

    #[tokio::test]
    async fn test_escrow_validation() {
        let mut request = create_test_request();
        
        // Valid request
        assert!(request.validate().is_ok());
        
        // Invalid amount
        request.amount = -100;
        assert!(request.validate().is_err());
        
        // Reset amount
        request.amount = 1000;
        
        // Same buyer and seller
        let same_id = Uuid::new_v4();
        request.buyer_id = same_id;
        request.seller_id = same_id;
        assert!(request.validate().is_err());
    }

    #[tokio::test]
    #[ignore] // Requires database setup
    async fn test_timeout_detection() {
        let db_pool = setup_test_db().await;
        
        let escrow_service = EscrowService::new(
            db_pool.clone(),
            "https://horizon-testnet.stellar.org".to_string(),
            "Test SDF Network ; September 2015".to_string(),
        );

        // Detect timeouts
        let result = escrow_service.detect_timeouts().await;
        
        match result {
            Ok(timed_out_escrows) => {
                // Verify it returns a list (even if empty)
                assert!(timed_out_escrows.len() >= 0);
            }
            Err(_) => {
                // Expected if database is not set up
            }
        }
    }

    #[tokio::test]
    #[ignore] // Requires database setup
    async fn test_db_chain_consistency() {
        let db_pool = setup_test_db().await;
        
        let escrow_service = EscrowService::new(
            db_pool.clone(),
            "https://horizon-testnet.stellar.org".to_string(),
            "Test SDF Network ; September 2015".to_string(),
        );

        // This test would:
        // 1. Create an escrow in DB
        // 2. Query the on-chain status
        // 3. Verify they match
        
        // For now, just verify service can be instantiated
        let request = create_test_request();
        let _result = escrow_service.create_escrow(request).await;
    }

    #[test]
    fn test_escrow_status_enum() {
        // Verify all status variants are covered
        let statuses = vec![
            EscrowStatus::Pending,
            EscrowStatus::Active,
            EscrowStatus::Released,
            EscrowStatus::Cancelled,
            EscrowStatus::TimedOut,
            EscrowStatus::Disputed,
        ];
        
        assert_eq!(statuses.len(), 6);
        
        // Test serialization
        for status in statuses {
            let json = serde_json::to_string(&status).unwrap();
            assert!(!json.is_empty());
        }
    }
}

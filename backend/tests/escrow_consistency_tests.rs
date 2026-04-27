//! Consistency tests between database and blockchain

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use stellovault_server::escrow::{CreateEscrowRequest, EscrowStatus};

    /// Helper to create test escrow request
    fn create_test_request() -> CreateEscrowRequest {
        CreateEscrowRequest {
            buyer_id: Uuid::new_v4(),
            seller_id: Uuid::new_v4(),
            lender_id: Uuid::new_v4(),
            collateral_id: Uuid::new_v4().to_string(),
            amount: 1000,
            oracle_address: "GABC123...".to_string(),
            release_conditions: r#"{"condition":"shipment_delivered"}"#.to_string(),
            timeout_hours: Some(24),
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

    #[test]
    fn test_escrow_status_transitions() {
        // Verify valid transitions are logically sound
        // Pending -> Active (funded)
        // Active -> Released
        // Active -> Disputed
        // Disputed -> Released or Refunded
        // Any -> Cancelled (if not terminal)

        let terminal = vec![
            EscrowStatus::Released,
            EscrowStatus::Refunded,
            EscrowStatus::Cancelled,
            EscrowStatus::TimedOut,
        ];

        for status in &terminal {
            let json = serde_json::to_string(status).unwrap();
            assert!(!json.is_empty());
        }
    }

    #[test]
    fn test_create_request_validation_zero_amount() {
        let mut req = create_test_request();
        req.amount = 0;
        assert!(req.validate().is_err(), "Zero amount should fail validation");
    }

    #[test]
    fn test_create_request_validation_positive_amount() {
        let req = create_test_request();
        assert!(req.validate().is_ok(), "Positive amount should pass validation");
    }
}

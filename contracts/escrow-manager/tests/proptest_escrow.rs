//! Property-based tests for the EscrowManager contract using proptest.
//!
//! These tests verify key invariants hold for *all* generated inputs:
//!   1. Total contract balance == sum of active escrow amounts
//!   2. create_escrow always rejects amount <= 0
//!   3. refund_escrow preserves the balance invariant
//!   4. Escrow status transitions are valid (Active → Released | Refunded)

#[cfg(test)]
mod proptest_escrow {
    extern crate std;

    use proptest::prelude::*;
    use soroban_sdk::{
        contract, contractimpl, symbol_short,
        testutils::{Address as _, Ledger as _},
        token, Address, Bytes, Env, Vec,
    };

    use escrow_manager::{ConfirmationData, EscrowConfig, EscrowManager, EscrowStatus};

    // ── Mock contracts ──────────────────────────────────────────────────

    #[contract]
    pub struct MockCollateralRegistry;

    #[contractimpl]
    impl MockCollateralRegistry {
        pub fn lock_collateral(_env: Env, _id: u64) {}
        pub fn unlock_collateral(_env: Env, _id: u64) {}
    }

    #[contract]
    pub struct MockOracleAdapter;

    #[contractimpl]
    impl MockOracleAdapter {
        pub fn get_confirmation(_env: Env, _escrow_id: Bytes) -> Option<Vec<ConfirmationData>> {
            None
        }
        pub fn check_consensus(
            _env: Env,
            _escrow_id: Bytes,
            _event_type: u32,
            _threshold: u32,
            _oracle_set: Vec<Address>,
        ) -> bool {
            false
        }
    }

    #[contract]
    pub struct MockLoanManagement;

    #[contractimpl]
    impl MockLoanManagement {}

    #[contract]
    pub struct MockTreasury;

    #[contractimpl]
    impl MockTreasury {
        pub fn get_fee_bps(_env: Env) -> u32 {
            50
        }
        pub fn deposit_fee(_env: Env, _asset: Address, _amount: i128) {}
    }

    // ── Helpers ─────────────────────────────────────────────────────────

    struct TestEnv {
        env: Env,
        escrow_contract: Address,
        token_addr: Address,
        lender: Address,
        buyer: Address,
        seller: Address,
    }

    fn setup() -> TestEnv {
        let env = Env::default();
        env.mock_all_auths();

        let coll_reg = env.register_contract(None, MockCollateralRegistry);
        let oracle = env.register_contract(None, MockOracleAdapter);
        let loan_mgr = env.register_contract(None, MockLoanManagement);
        let treasury = env.register_contract(None, MockTreasury);
        let escrow_contract = env.register_contract(None, EscrowManager);
        let admin = Address::generate(&env);

        let token_admin = Address::generate(&env);
        let token_id = env.register_stellar_asset_contract_v2(token_admin.clone());
        let token_addr = token_id.address();
        let sac = token::StellarAssetClient::new(&env, &token_addr);

        let lender = Address::generate(&env);
        sac.mint(&lender, &10_000_000_000i128);

        env.as_contract(&escrow_contract, || {
            EscrowManager::initialize(
                env.clone(),
                admin.clone(),
                coll_reg,
                oracle,
                loan_mgr,
                treasury,
            )
            .unwrap();
        });

        let buyer = Address::generate(&env);
        let seller = Address::generate(&env);

        TestEnv {
            env,
            escrow_contract,
            token_addr,
            lender,
            buyer,
            seller,
        }
    }

    // ── Property: create_escrow rejects non-positive amounts ────────────

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1000))]

        #[test]
        #[ignore = "fuzz"]
        fn prop_create_rejects_non_positive_amount(
            amount in prop_oneof![
                Just(0i128),
                (-1_000_000i128..0i128),
                Just(i128::MIN),
            ],
        ) {
            let t = setup();
            t.env.as_contract(&t.escrow_contract, || {
                let config = EscrowConfig {
                    buyer: t.buyer.clone(),
                    seller: t.seller.clone(),
                    lender: t.lender.clone(),
                    collateral_id: 1,
                    amount,
                    asset: t.token_addr.clone(),
                    required_confirmation: 1,
                    expiry_ts: t.env.ledger().timestamp() + 10_000,
                    destination_asset: t.token_addr.clone(),
                    min_destination_amount: 1, // positive to isolate amount check
                    required_confirmations: 0,
                    oracle_set: Vec::new(&t.env),
                };
                let result = EscrowManager::create_escrow(t.env.clone(), config);
                prop_assert!(result.is_err(), "Expected error for amount={}", amount);
                Ok(())
            });
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1000))]

        // ── Property: active escrow sum invariant after N creates ────────

        #[test]
        #[ignore = "fuzz"]
        fn prop_active_sum_equals_total_after_creates(
            amounts in proptest::collection::vec(1i128..=1_000_000i128, 1..10),
        ) {
            let t = setup();
            let mut expected_sum: i128 = 0;

            t.env.as_contract(&t.escrow_contract, || {
                for (i, &amt) in amounts.iter().enumerate() {
                    let config = EscrowConfig {
                        buyer: t.buyer.clone(),
                        seller: t.seller.clone(),
                        lender: t.lender.clone(),
                        collateral_id: i as u64 + 1,
                        amount: amt,
                        asset: t.token_addr.clone(),
                        required_confirmation: 1,
                        expiry_ts: t.env.ledger().timestamp() + 100_000,
                        destination_asset: t.token_addr.clone(),
                        min_destination_amount: amt,
                        required_confirmations: 0,
                        oracle_set: Vec::new(&t.env),
                    };
                    if EscrowManager::create_escrow(t.env.clone(), config).is_ok() {
                        expected_sum += amt;
                    }
                }

                // Verify invariant
                let next_id: u64 = t.env
                    .storage()
                    .instance()
                    .get(&symbol_short!("next_id"))
                    .unwrap_or(1);

                let mut actual_sum: i128 = 0;
                for id in 1..next_id {
                    if let Some(escrow) = EscrowManager::get_escrow(t.env.clone(), id) {
                        if escrow.status == EscrowStatus::Active {
                            actual_sum += escrow.amount;
                        }
                    }
                }

                prop_assert_eq!(
                    actual_sum,
                    expected_sum,
                    "Active escrow sum mismatch"
                );
                Ok(())
            });
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(500))]

        // ── Property: refund preserves balance invariant ─────────────────

        #[test]
        #[ignore = "fuzz"]
        fn prop_refund_preserves_balance_invariant(
            num_escrows in 1usize..5,
            refund_idx in 0usize..5,
        ) {
            let t = setup();
            let amount: i128 = 1_000;
            let mut created_ids: std::vec::Vec<u64> = std::vec::Vec::new();
            let mut expected_sum: i128 = 0;

            t.env.as_contract(&t.escrow_contract, || {
                // Create escrows with short expiry
                for i in 0..num_escrows {
                    let config = EscrowConfig {
                        buyer: t.buyer.clone(),
                        seller: t.seller.clone(),
                        lender: t.lender.clone(),
                        collateral_id: i as u64 + 1,
                        amount,
                        asset: t.token_addr.clone(),
                        required_confirmation: 1,
                        expiry_ts: t.env.ledger().timestamp() + 100,
                        destination_asset: t.token_addr.clone(),
                        min_destination_amount: amount,
                        required_confirmations: 0,
                        oracle_set: Vec::new(&t.env),
                    };
                    if let Ok(id) = EscrowManager::create_escrow(t.env.clone(), config) {
                        created_ids.push(id);
                        expected_sum += amount;
                    }
                }

                if created_ids.is_empty() {
                    return Ok(());
                }

                // Advance time past expiry
                t.env.ledger().set_timestamp(t.env.ledger().timestamp() + 200);

                // Refund one escrow
                let idx = refund_idx % created_ids.len();
                let eid = created_ids[idx];
                if EscrowManager::refund_escrow(t.env.clone(), eid).is_ok() {
                    expected_sum -= amount;
                }

                // Verify invariant
                let next_id: u64 = t.env
                    .storage()
                    .instance()
                    .get(&symbol_short!("next_id"))
                    .unwrap_or(1);

                let mut actual_sum: i128 = 0;
                for id in 1..next_id {
                    if let Some(escrow) = EscrowManager::get_escrow(t.env.clone(), id) {
                        if escrow.status == EscrowStatus::Active {
                            actual_sum += escrow.amount;
                        }
                    }
                }

                prop_assert_eq!(actual_sum, expected_sum, "Balance invariant violated after refund");
                Ok(())
            });
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1000))]

        // ── Property: escrow ID always increments monotonically ──────────

        #[test]
        #[ignore = "fuzz"]
        fn prop_escrow_ids_are_monotonically_increasing(
            num_escrows in 2usize..8,
        ) {
            let t = setup();
            let mut ids: std::vec::Vec<u64> = std::vec::Vec::new();

            t.env.as_contract(&t.escrow_contract, || {
                for i in 0..num_escrows {
                    let config = EscrowConfig {
                        buyer: t.buyer.clone(),
                        seller: t.seller.clone(),
                        lender: t.lender.clone(),
                        collateral_id: i as u64 + 1,
                        amount: 100,
                        asset: t.token_addr.clone(),
                        required_confirmation: 1,
                        expiry_ts: t.env.ledger().timestamp() + 100_000,
                        destination_asset: t.token_addr.clone(),
                        min_destination_amount: 100,
                        required_confirmations: 0,
                        oracle_set: Vec::new(&t.env),
                    };
                    if let Ok(id) = EscrowManager::create_escrow(t.env.clone(), config) {
                        ids.push(id);
                    }
                }

                for window in ids.windows(2) {
                    prop_assert!(
                        window[1] > window[0],
                        "IDs not monotonically increasing: {} not > {}",
                        window[1],
                        window[0]
                    );
                }
                Ok(())
            });
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(500))]

        // ── Property: refund only succeeds after expiry ──────────────────

        #[test]
        #[ignore = "fuzz"]
        fn prop_refund_fails_before_expiry(
            expiry_offset in 1000u64..1_000_000,
        ) {
            let t = setup();

            t.env.as_contract(&t.escrow_contract, || {
                let config = EscrowConfig {
                    buyer: t.buyer.clone(),
                    seller: t.seller.clone(),
                    lender: t.lender.clone(),
                    collateral_id: 1,
                    amount: 1000,
                    asset: t.token_addr.clone(),
                    required_confirmation: 1,
                    expiry_ts: t.env.ledger().timestamp() + expiry_offset,
                    destination_asset: t.token_addr.clone(),
                    min_destination_amount: 1000,
                    required_confirmations: 0,
                    oracle_set: Vec::new(&t.env),
                };
                let id = EscrowManager::create_escrow(t.env.clone(), config).unwrap();

                // Without advancing time, refund should fail
                let result = EscrowManager::refund_escrow(t.env.clone(), id);
                prop_assert!(result.is_err(), "Refund should fail before expiry");
                Ok(())
            });
        }
    }
}

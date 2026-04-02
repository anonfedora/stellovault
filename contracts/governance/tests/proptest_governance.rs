//! Property-based tests for the Governance contract using proptest.
//!
//! Invariants verified:
//!   1. Vote tallies always equal the sum of individual voting powers
//!   2. Parameter validation rejects out-of-range values for every parameter
//!   3. Quorum and majority checks are consistent with vote counts
//!   4. Proposal ID counter increments correctly
//!   5. Double-voting is always rejected

#![cfg(feature = "testutils")]

#[cfg(test)]
mod proptest_governance {
    extern crate std;

    use proptest::prelude::*;
    use soroban_sdk::{
        contract, contractimpl, contracttype, symbol_short,
        testutils::{Address as _, Ledger as _},
        Address, Env,
    };

    use governance::{ContractError, Governance, GovernanceConfig};

    // ── Mock RiskAssessment ─────────────────────────────────────────────

    #[contracttype]
    #[derive(Clone, Debug)]
    pub struct RiskParameters {
        pub liquidation_threshold: u32,
        pub liquidation_penalty: u32,
        pub min_health_factor: u32,
        pub max_liquidation_ratio: u32,
        pub grace_period: u64,
        pub liquidator_bonus: u32,
    }

    #[contract]
    pub struct MockRiskAssessment;

    #[contractimpl]
    impl MockRiskAssessment {
        pub fn get_risk_parameters(_env: Env) -> RiskParameters {
            RiskParameters {
                liquidation_threshold: 8000,
                liquidation_penalty: 500,
                min_health_factor: 10000,
                max_liquidation_ratio: 5000,
                grace_period: 3600,
                liquidator_bonus: 500,
            }
        }
        pub fn update_risk_parameters(_env: Env, _new_params: RiskParameters) {}
    }

    // ── Helpers ─────────────────────────────────────────────────────────

    struct TestEnv {
        env: Env,
        gov_contract: Address,
        risk_contract: Address,
    }

    fn setup() -> TestEnv {
        let env = Env::default();
        env.mock_all_auths();

        let mock_risk = env.register_contract(None, MockRiskAssessment);
        let gov_contract = env.register_contract(None, Governance);
        let admin = Address::generate(&env);
        let token = Address::generate(&env);

        env.as_contract(&gov_contract, || {
            Governance::initialize(env.clone(), admin, token, mock_risk.clone()).unwrap();
        });

        TestEnv {
            env,
            gov_contract,
            risk_contract: mock_risk,
        }
    }

    // ── Property: parameter validation rejects out-of-range values ──────

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(2000))]

        #[test]
        #[ignore = "fuzz"]
        fn prop_liq_threshold_validation(value in prop_oneof![
            // Valid range: 5000..=9500
            (5000i128..=9500i128),
            // Below valid
            (i128::MIN..5000i128),
            // Above valid
            (9501i128..100_000i128),
        ]) {
            let t = setup();
            t.env.as_contract(&t.gov_contract, || {
                let proposer = Address::generate(&t.env);
                Governance::set_voting_power(t.env.clone(), proposer.clone(), 2000 * 2000);

                let result = Governance::create_proposal(
                    t.env.clone(),
                    proposer,
                    t.risk_contract.clone(),
                    symbol_short!("liq_thr"),
                    value,
                );

                if (5000..=9500).contains(&value) {
                    prop_assert!(result.is_ok(), "Valid value {} should be accepted", value);
                } else {
                    prop_assert!(result.is_err(), "Invalid value {} should be rejected", value);
                }
                Ok(())
            });
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(2000))]

        #[test]
        #[ignore = "fuzz"]
        fn prop_liq_penalty_validation(value in prop_oneof![
            (100i128..=1000i128),
            (i128::MIN..100i128),
            (1001i128..100_000i128),
        ]) {
            let t = setup();
            t.env.as_contract(&t.gov_contract, || {
                let proposer = Address::generate(&t.env);
                Governance::set_voting_power(t.env.clone(), proposer.clone(), 2000 * 2000);

                let result = Governance::create_proposal(
                    t.env.clone(),
                    proposer,
                    t.risk_contract.clone(),
                    symbol_short!("liq_pen"),
                    value,
                );

                if (100..=1000).contains(&value) {
                    prop_assert!(result.is_ok(), "Valid value {} should be accepted", value);
                } else {
                    prop_assert!(result.is_err(), "Invalid value {} should be rejected", value);
                }
                Ok(())
            });
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(2000))]

        #[test]
        #[ignore = "fuzz"]
        fn prop_grace_period_validation(value in prop_oneof![
            (300i128..=86400i128),
            (0i128..300i128),
            (86401i128..1_000_000i128),
        ]) {
            let t = setup();
            t.env.as_contract(&t.gov_contract, || {
                let proposer = Address::generate(&t.env);
                Governance::set_voting_power(t.env.clone(), proposer.clone(), 2000 * 2000);

                let result = Governance::create_proposal(
                    t.env.clone(),
                    proposer,
                    t.risk_contract.clone(),
                    symbol_short!("grace_pd"),
                    value,
                );

                if (300..=86400).contains(&value) {
                    prop_assert!(result.is_ok(), "Valid value {} should be accepted", value);
                } else {
                    prop_assert!(result.is_err(), "Invalid value {} should be rejected", value);
                }
                Ok(())
            });
        }
    }

    // ── Property: vote tallies match individual powers ───────────────────

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(500))]

        #[test]
        #[ignore = "fuzz"]
        fn prop_vote_tallies_consistent(
            powers in proptest::collection::vec(1i128..100_000i128, 1..8),
            supports in proptest::collection::vec(proptest::bool::ANY, 1..8),
        ) {
            let t = setup();
            t.env.as_contract(&t.gov_contract, || {
                let proposer = Address::generate(&t.env);
                Governance::set_voting_power(t.env.clone(), proposer.clone(), 2000 * 2000);

                let proposal_id = Governance::create_proposal(
                    t.env.clone(),
                    proposer,
                    t.risk_contract.clone(),
                    symbol_short!("liq_thr"),
                    7500,
                ).unwrap();

                let mut expected_for: i128 = 0;
                let mut expected_against: i128 = 0;
                let count = powers.len().min(supports.len());

                for i in 0..count {
                    let voter = Address::generate(&t.env);
                    let power = powers[i];
                    let support = supports[i];

                    Governance::set_voting_power(t.env.clone(), voter.clone(), power * power);
                    if Governance::cast_vote(t.env.clone(), proposal_id, voter, support, power).is_ok() {
                        if support {
                            expected_for += power;
                        } else {
                            expected_against += power;
                        }
                    }
                }

                let proposal = Governance::get_proposal(t.env.clone(), proposal_id).unwrap();
                prop_assert_eq!(proposal.votes_for, expected_for);
                prop_assert_eq!(proposal.votes_against, expected_against);
                Ok(())
            });
        }
    }

    // ── Property: double voting is always rejected ──────────────────────

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(500))]

        #[test]
        #[ignore = "fuzz"]
        fn prop_double_vote_always_rejected(
            power in 1i128..100_000,
            first_support in proptest::bool::ANY,
            second_support in proptest::bool::ANY,
        ) {
            let t = setup();
            t.env.as_contract(&t.gov_contract, || {
                let proposer = Address::generate(&t.env);
                Governance::set_voting_power(t.env.clone(), proposer.clone(), 2000 * 2000);

                let proposal_id = Governance::create_proposal(
                    t.env.clone(),
                    proposer,
                    t.risk_contract.clone(),
                    symbol_short!("liq_thr"),
                    7500,
                ).unwrap();

                let voter = Address::generate(&t.env);
                Governance::set_voting_power(t.env.clone(), voter.clone(), power * power);

                let first = Governance::cast_vote(
                    t.env.clone(), proposal_id, voter.clone(), first_support, power,
                );
                prop_assert!(first.is_ok());

                let second = Governance::cast_vote(
                    t.env.clone(), proposal_id, voter, second_support, power,
                );
                prop_assert_eq!(second, Err(ContractError::AlreadyVoted));
                Ok(())
            });
        }
    }

    // ── Property: proposal IDs increment monotonically ──────────────────

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(500))]

        #[test]
        #[ignore = "fuzz"]
        fn prop_proposal_ids_monotonic(num_proposals in 2usize..6) {
            let t = setup();
            t.env.as_contract(&t.gov_contract, || {
                let mut ids: std::vec::Vec<u64> = std::vec::Vec::new();

                for _ in 0..num_proposals {
                    let proposer = Address::generate(&t.env);
                    Governance::set_voting_power(t.env.clone(), proposer.clone(), 2000 * 2000);

                    if let Ok(id) = Governance::create_proposal(
                        t.env.clone(),
                        proposer,
                        t.risk_contract.clone(),
                        symbol_short!("liq_thr"),
                        7500,
                    ) {
                        ids.push(id);
                    }
                }

                for window in ids.windows(2) {
                    prop_assert!(
                        window[1] > window[0],
                        "Proposal IDs not monotonic: {} not > {}",
                        window[1], window[0]
                    );
                }
                Ok(())
            });
        }
    }

    // ── Property: quorum/majority check consistent with execution ────────

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(300))]

        #[test]
        #[ignore = "fuzz"]
        fn prop_execution_requires_quorum_and_majority(
            for_power in 0i128..200_000,
            against_power in 0i128..200_000,
            total_power in 100_000i128..1_000_000,
            quorum_bps in 100u32..5000,
            majority_bps in 5000u32..9000,
        ) {
            let t = setup();
            t.env.as_contract(&t.gov_contract, || {
                let config = GovernanceConfig {
                    voting_period: 604800,
                    timelock_period: 86400,
                    tally_period: 3600,
                    quorum_bps,
                    majority_bps,
                    min_voting_power: 100,
                };
                Governance::update_config(t.env.clone(), config).unwrap();
                Governance::set_total_voting_power(t.env.clone(), total_power).unwrap();

                let proposer = Address::generate(&t.env);
                Governance::set_voting_power(t.env.clone(), proposer.clone(), 2000 * 2000);

                let proposal_id = Governance::create_proposal(
                    t.env.clone(),
                    proposer,
                    t.risk_contract.clone(),
                    symbol_short!("liq_thr"),
                    7500,
                ).unwrap();

                // Vote for
                if for_power > 0 {
                    let voter_for = Address::generate(&t.env);
                    Governance::set_voting_power(t.env.clone(), voter_for.clone(), for_power * for_power);
                    let _ = Governance::cast_vote(t.env.clone(), proposal_id, voter_for, true, for_power);
                }

                // Vote against
                if against_power > 0 {
                    let voter_against = Address::generate(&t.env);
                    Governance::set_voting_power(t.env.clone(), voter_against.clone(), against_power * against_power);
                    let _ = Governance::cast_vote(t.env.clone(), proposal_id, voter_against, false, against_power);
                }

                // Advance past voting + timelock
                t.env.ledger().set_timestamp(
                    t.env.ledger().timestamp() + 604800 + 86400 + 1,
                );

                let exec_result = Governance::execute_proposal(t.env.clone(), proposal_id);

                let total_votes = for_power + against_power;
                let quorum_needed = total_power * quorum_bps as i128 / 10000;
                let majority_needed = total_votes * majority_bps as i128 / 10000;

                if total_votes < quorum_needed || for_power < majority_needed {
                    prop_assert!(
                        exec_result.is_err(),
                        "Should fail: votes={}, quorum_needed={}, for={}, majority_needed={}",
                        total_votes, quorum_needed, for_power, majority_needed
                    );
                }
                // If quorum and majority are met, success is valid
                Ok(())
            });
        }
    }
}

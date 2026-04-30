//! Fuzz target for Governance voting invariants.
//!
//! Invariants tested:
//!   1. votes_for + votes_against == sum of individual vote powers
//!   2. A proposal cannot be executed unless quorum AND majority are met
//!   3. No voter can vote twice on the same proposal

#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short,
    testutils::{Address as _, Ledger as _},
    Address, Env,
};

// ── Mock RiskAssessment (required by execute_proposal) ──────────────────────

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

// ── Fuzz input ──────────────────────────────────────────────────────────────

#[derive(Arbitrary, Debug)]
struct FuzzInput {
    /// Total voting power (clamped to [10_000, 10_000_000])
    total_power: u32,
    /// Quorum bps (clamped to [100, 5000])
    quorum_bps: u16,
    /// Majority bps (clamped to [5000, 9000])
    majority_bps: u16,
    /// Voter operations
    voters: std::vec::Vec<VoterAction>,
    /// Whether to attempt execution
    try_execute: bool,
}

#[derive(Arbitrary, Debug)]
struct VoterAction {
    /// Voter index (we generate a fixed set of addresses)
    voter_idx: u8,
    /// Voting power for this voter
    power: u32,
    /// true = vote for, false = vote against
    support: bool,
}

fuzz_target!(|input: FuzzInput| {
    let env = Env::default();
    env.mock_all_auths();

    if input.voters.is_empty() || input.voters.len() > 32 {
        return;
    }

    let mock_risk = env.register(MockRiskAssessment, ());
    let gov_contract = env.register(governance::Governance, ());
    let gov = governance::GovernanceClient::new(&env, &gov_contract);
    let admin = Address::generate(&env);
    let token = Address::generate(&env);

    let total_power = (input.total_power as i128).max(10_000).min(10_000_000);
    let quorum_bps = (input.quorum_bps as u32).max(100).min(5000);
    let majority_bps = (input.majority_bps as u32).max(5000).min(9000);

    // Phase 1: Initialize and write config/power directly to storage.
    gov.initialize(&admin, &token, &mock_risk);
    env.as_contract(&gov_contract, || {
        let config = governance::GovernanceConfig {
            voting_period: 604800,
            timelock_period: 86400,
            tally_period: 3600,
            quorum_bps,
            majority_bps,
            min_voting_power: 100,
        };
        env.storage().instance().set(&symbol_short!("config"), &config);
        env.storage()
            .instance()
            .set(&symbol_short!("total_pwr"), &total_power);
    });

    // Phase 2: Create a proposal.
    let proposer = Address::generate(&env);
    gov.set_voting_power(&proposer, &1000);
    let proposal_id = gov.create_proposal(&proposer, &mock_risk, &symbol_short!("liq_thr"), &7500);

    // Phase 3: Cast votes.
    let mut voter_addrs: std::vec::Vec<Address> = std::vec::Vec::new();
    for _ in 0..16 {
        voter_addrs.push(Address::generate(&env));
    }

    let mut total_for: i128 = 0;
    let mut total_against: i128 = 0;
    let mut voted: std::collections::HashSet<u8> = std::collections::HashSet::new();

    for action in &input.voters {
        let idx = (action.voter_idx % 16) as usize;
        let effective_idx = idx as u8;

        if voted.contains(&effective_idx) {
            // Try to cast a duplicate vote (should fail)
            let result = gov.try_cast_vote(&proposal_id, &voter_addrs[idx], &action.support, &1);
            assert!(
                result.is_err(),
                "INVARIANT VIOLATED: duplicate vote should be rejected"
            );
            continue;
        }

        let power = (action.power as i128).max(1).min(1_000_000);
        gov.set_voting_power(&voter_addrs[idx], &(power * power));

        match gov.try_cast_vote(&proposal_id, &voter_addrs[idx], &action.support, &power) {
            Ok(Ok(())) => {
                voted.insert(effective_idx);
                if action.support {
                    total_for += power;
                } else {
                    total_against += power;
                }
            }
            Ok(Err(_)) | Err(_) => { /* validation errors are fine */ }
        }
    }

    // ── INVARIANT 1: vote tally matches individual votes ───────────────────
    let proposal = gov.get_proposal(&proposal_id);
    assert_eq!(
        proposal.votes_for, total_for,
        "INVARIANT VIOLATED: votes_for ({}) != tracked for ({})",
        proposal.votes_for, total_for
    );
    assert_eq!(
        proposal.votes_against, total_against,
        "INVARIANT VIOLATED: votes_against ({}) != tracked against ({})",
        proposal.votes_against, total_against
    );

    // ── INVARIANT 2: execution respects quorum + majority ──────────────────
    if input.try_execute {
        env.ledger()
            .set_timestamp(env.ledger().timestamp() + 604800 + 86400 + 1);

        let exec_result = gov.try_execute_proposal(&proposal_id);

        let total_votes = total_for + total_against;
        let quorum_needed = total_power * quorum_bps as i128 / 10000;
        let majority_needed = total_votes * majority_bps as i128 / 10000;

        if total_votes < quorum_needed || total_for < majority_needed {
            assert!(
                exec_result.is_err(),
                "INVARIANT VIOLATED: proposal executed without quorum/majority"
            );
        }
    }
});

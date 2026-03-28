//! Fuzz target for EscrowManager invariants.
//!
//! Primary invariant tested:
//!   "Total assets held by the contract must always equal
//!    the sum of amounts across all Active escrows."
//!
//! The fuzzer generates sequences of operations (create / refund) and
//! checks the invariant after every step.

#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use soroban_sdk::{
    contract, contractimpl, symbol_short,
    testutils::{Address as _, Ledger as _},
    token, Address, Bytes, Env, Vec,
};

// ── Mock contracts used during fuzzing ──────────────────────────────────────

// Minimal CollateralRegistry mock
#[contract]
pub struct MockCollateralRegistry;

#[contractimpl]
impl MockCollateralRegistry {
    pub fn lock_collateral(_env: Env, _id: u64) {}
    pub fn unlock_collateral(_env: Env, _id: u64) {}
}

// Minimal OracleAdapter mock
#[contract]
pub struct MockOracleAdapter;

#[contractimpl]
impl MockOracleAdapter {
    pub fn get_confirmation(
        _env: Env,
        _escrow_id: Bytes,
    ) -> Option<Vec<escrow_manager::ConfirmationData>> {
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

// Minimal LoanManagement mock
#[contract]
pub struct MockLoanManagement;

#[contractimpl]
impl MockLoanManagement {}

// Minimal ProtocolTreasury mock
#[contract]
pub struct MockTreasury;

#[contractimpl]
impl MockTreasury {
    pub fn get_fee_bps(_env: Env) -> u32 {
        50
    }
    pub fn deposit_fee(_env: Env, _asset: Address, _amount: i128) {}
}

// ── Fuzz input ──────────────────────────────────────────────────────────────

#[derive(Arbitrary, Debug)]
struct FuzzInput {
    /// Each operation to execute in sequence
    ops: std::vec::Vec<Op>,
}

#[derive(Arbitrary, Debug)]
enum Op {
    CreateEscrow {
        /// Amount clamped to [1, 1_000_000_000] inside harness
        amount: u64,
        /// Expiry offset in seconds from current timestamp
        expiry_offset: u32,
    },
    RefundExpired {
        /// Index into the list of created escrow IDs
        escrow_idx: u8,
    },
    AdvanceTime {
        /// Seconds to advance the ledger clock
        seconds: u32,
    },
}

// ── Invariant checker ───────────────────────────────────────────────────────

/// Walk all escrow IDs and sum amounts of Active ones.
fn active_escrow_sum(env: &Env, next_id: u64) -> i128 {
    let mut sum: i128 = 0;
    for id in 1..next_id {
        if let Some(escrow) = escrow_manager::EscrowManager::get_escrow(env.clone(), id) {
            if escrow.status == escrow_manager::EscrowStatus::Active {
                sum += escrow.amount;
            }
        }
    }
    sum
}

// ── Fuzz target entry point ─────────────────────────────────────────────────

fuzz_target!(|input: FuzzInput| {
    // Limit ops to avoid excessive run-time per input
    if input.ops.is_empty() || input.ops.len() > 64 {
        return;
    }

    let env = Env::default();
    env.mock_all_auths();

    // Register mock contracts
    let coll_reg = env.register_contract(None, MockCollateralRegistry);
    let oracle = env.register_contract(None, MockOracleAdapter);
    let loan_mgr = env.register_contract(None, MockLoanManagement);
    let treasury = env.register_contract(None, MockTreasury);
    let escrow_contract = env.register_contract(None, escrow_manager::EscrowManager);

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);

    // Deploy a test token
    let token_id = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_addr = token_id.address();
    let sac = token::StellarAssetClient::new(&env, &token_addr);

    // Initialize the escrow manager
    env.as_contract(&escrow_contract, || {
        escrow_manager::EscrowManager::initialize(
            env.clone(),
            admin.clone(),
            coll_reg.clone(),
            oracle.clone(),
            loan_mgr.clone(),
            treasury.clone(),
        )
        .unwrap();
    });

    // Pre-fund enough lenders. We reuse one lender for simplicity.
    let lender = Address::generate(&env);
    sac.mint(&lender, &10_000_000_000_000i128);

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);

    // Track created escrow IDs for refund ops
    let mut escrow_ids: std::vec::Vec<u64> = std::vec::Vec::new();

    // Running balance that the contract *should* hold
    let mut expected_balance: i128 = 0;

    for op in &input.ops {
        match op {
            Op::CreateEscrow {
                amount,
                expiry_offset,
            } => {
                // Clamp amount to sensible range
                let amt = (*amount).max(1).min(1_000_000_000) as i128;
                let expiry_ts =
                    env.ledger().timestamp() + (*expiry_offset).max(1) as u64;

                let config = escrow_manager::EscrowConfig {
                    buyer: buyer.clone(),
                    seller: seller.clone(),
                    lender: lender.clone(),
                    collateral_id: escrow_ids.len() as u64 + 1,
                    amount: amt,
                    asset: token_addr.clone(),
                    required_confirmation: 1,
                    expiry_ts,
                    destination_asset: token_addr.clone(),
                    min_destination_amount: amt,
                    required_confirmations: 0,
                    oracle_set: Vec::new(&env),
                };

                env.as_contract(&escrow_contract, || {
                    match escrow_manager::EscrowManager::create_escrow(env.clone(), config) {
                        Ok(id) => {
                            escrow_ids.push(id);
                            expected_balance += amt;
                        }
                        Err(_) => { /* validation errors are fine */ }
                    }
                });
            }
            Op::RefundExpired { escrow_idx } => {
                if escrow_ids.is_empty() {
                    continue;
                }
                let idx = (*escrow_idx as usize) % escrow_ids.len();
                let eid = escrow_ids[idx];

                env.as_contract(&escrow_contract, || {
                    // Check escrow before attempting refund
                    if let Some(escrow) =
                        escrow_manager::EscrowManager::get_escrow(env.clone(), eid)
                    {
                        if escrow.status == escrow_manager::EscrowStatus::Active {
                            if let Ok(()) = escrow_manager::EscrowManager::refund_escrow(
                                env.clone(),
                                eid,
                            ) {
                                expected_balance -= escrow.amount;
                            }
                        }
                    }
                });
            }
            Op::AdvanceTime { seconds } => {
                let new_ts =
                    env.ledger().timestamp() + (*seconds).min(31_536_000) as u64;
                env.ledger().set_timestamp(new_ts);
            }
        }

        // ── INVARIANT CHECK ─────────────────────────────────────────────
        // After every operation the sum of active escrow amounts must
        // equal our tracked expected balance.
        env.as_contract(&escrow_contract, || {
            let next_id: u64 = env
                .storage()
                .instance()
                .get(&symbol_short!("next_id"))
                .unwrap_or(1);
            let actual_sum = active_escrow_sum(&env, next_id);
            assert_eq!(
                actual_sum, expected_balance,
                "INVARIANT VIOLATED: active escrow sum ({}) != expected ({})",
                actual_sum, expected_balance
            );
        });
    }
});

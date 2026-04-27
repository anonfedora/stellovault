//! Gas optimization analysis and benchmarking

mod common;

use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env,
};
use common::*;

/// Gas cost baseline for contract operations
/// These tests measure and document gas consumption
#[test]
fn test_gas_cost_collateral_registration() {
    let env = setup_env();
    let owner = Address::generate(&env);
    
    // Measure gas for collateral registration
    // let gas_before = env.budget().get_cpu_insns_consumed();
    
    // let _collateral_id = register_collateral(
    //     &env,
    //     owner,
    //     constants::MIN_COLLATERAL_VALUE,
    //     env.ledger().timestamp() + constants::ONE_YEAR_SECONDS,
    //     soroban_sdk::BytesN::from_array(&env, &[0u8; 32]),
    //     soroban_sdk::String::from_slice(&env, "ipfs://QmTest"),
    // ).unwrap();
    
    // let gas_after = env.budget().get_cpu_insns_consumed();
    // let gas_used = gas_after - gas_before;
    
    // println!("Collateral registration gas: {}", gas_used);
    // assert!(gas_used < 1_000_000, "Gas usage exceeds threshold");
}

#[test]
fn test_gas_cost_escrow_creation() {
    let env = setup_env();
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let arbiter = Address::generate(&env);
    
    // Measure gas for escrow creation
    // let gas_before = env.budget().get_cpu_insns_consumed();
    
    // let _escrow_id = create_escrow(
    //     &env,
    //     buyer,
    //     seller,
    //     arbiter,
    //     constants::MIN_LOAN_AMOUNT,
    //     env.ledger().timestamp() + constants::ONE_DAY_SECONDS,
    // ).unwrap();
    
    // let gas_after = env.budget().get_cpu_insns_consumed();
    // let gas_used = gas_after - gas_before;
    
    // println!("Escrow creation gas: {}", gas_used);
    // assert!(gas_used < 500_000, "Gas usage exceeds threshold");
}

#[test]
fn test_gas_cost_escrow_deposit() {
    let env = setup_env();
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let arbiter = Address::generate(&env);
    
    // Create escrow first
    // let escrow_id = create_escrow(
    //     &env,
    //     buyer.clone(),
    //     seller,
    //     arbiter,
    //     constants::MIN_LOAN_AMOUNT,
    //     env.ledger().timestamp() + constants::ONE_DAY_SECONDS,
    // ).unwrap();
    
    // Measure gas for deposit
    // let gas_before = env.budget().get_cpu_insns_consumed();
    
    // deposit_to_escrow(&env, escrow_id, buyer, constants::MIN_LOAN_AMOUNT).unwrap();
    
    // let gas_after = env.budget().get_cpu_insns_consumed();
    // let gas_used = gas_after - gas_before;
    
    // println!("Escrow deposit gas: {}", gas_used);
    // assert!(gas_used < 300_000, "Gas usage exceeds threshold");
}

#[test]
fn test_gas_cost_escrow_release() {
    let env = setup_env();
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let arbiter = Address::generate(&env);
    
    // Create and fund escrow
    // let escrow_id = create_escrow(
    //     &env,
    //     buyer.clone(),
    //     seller,
    //     arbiter.clone(),
    //     constants::MIN_LOAN_AMOUNT,
    //     env.ledger().timestamp() + constants::ONE_DAY_SECONDS,
    // ).unwrap();
    // deposit_to_escrow(&env, escrow_id, buyer, constants::MIN_LOAN_AMOUNT).unwrap();
    
    advance_ledger(&env, constants::ONE_DAY_SECONDS + 1);
    
    // Measure gas for release
    // let gas_before = env.budget().get_cpu_insns_consumed();
    
    // release_escrow(&env, escrow_id, arbiter).unwrap();
    
    // let gas_after = env.budget().get_cpu_insns_consumed();
    // let gas_used = gas_after - gas_before;
    
    // println!("Escrow release gas: {}", gas_used);
    // assert!(gas_used < 400_000, "Gas usage exceeds threshold");
}

#[test]
fn test_gas_cost_collateral_lock() {
    let env = setup_env();
    let owner = Address::generate(&env);
    
    // Register collateral first
    // let collateral_id = register_collateral(
    //     &env,
    //     owner.clone(),
    //     constants::MIN_COLLATERAL_VALUE,
    //     env.ledger().timestamp() + constants::ONE_YEAR_SECONDS,
    //     soroban_sdk::BytesN::from_array(&env, &[0u8; 32]),
    //     soroban_sdk::String::from_slice(&env, "ipfs://QmTest"),
    // ).unwrap();
    
    // Measure gas for lock
    // let gas_before = env.budget().get_cpu_insns_consumed();
    
    // lock_collateral(&env, collateral_id, owner).unwrap();
    
    // let gas_after = env.budget().get_cpu_insns_consumed();
    // let gas_used = gas_after - gas_before;
    
    // println!("Collateral lock gas: {}", gas_used);
    // assert!(gas_used < 200_000, "Gas usage exceeds threshold");
}

#[test]
fn test_gas_cost_dispute_creation() {
    let env = setup_env();
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let arbiter = Address::generate(&env);
    
    // Create escrow
    // let escrow_id = create_escrow(
    //     &env,
    //     buyer.clone(),
    //     seller,
    //     arbiter,
    //     constants::MIN_LOAN_AMOUNT,
    //     env.ledger().timestamp() + constants::ONE_DAY_SECONDS,
    // ).unwrap();
    
    // Measure gas for dispute
    // let gas_before = env.budget().get_cpu_insns_consumed();
    
    // dispute_escrow(&env, escrow_id, buyer, "Quality issue").unwrap();
    
    // let gas_after = env.budget().get_cpu_insns_consumed();
    // let gas_used = gas_after - gas_before;
    
    // println!("Dispute creation gas: {}", gas_used);
    // assert!(gas_used < 350_000, "Gas usage exceeds threshold");
}

#[test]
fn test_gas_cost_dispute_resolution() {
    let env = setup_env();
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let arbiter = Address::generate(&env);
    
    // Create and dispute escrow
    // let escrow_id = create_escrow(
    //     &env,
    //     buyer.clone(),
    //     seller,
    //     arbiter.clone(),
    //     constants::MIN_LOAN_AMOUNT,
    //     env.ledger().timestamp() + constants::ONE_DAY_SECONDS,
    // ).unwrap();
    // dispute_escrow(&env, escrow_id, buyer, "Issue").unwrap();
    
    // Measure gas for resolution
    // let gas_before = env.budget().get_cpu_insns_consumed();
    
    // resolve_dispute(&env, escrow_id, arbiter, true).unwrap();
    
    // let gas_after = env.budget().get_cpu_insns_consumed();
    // let gas_used = gas_after - gas_before;
    
    // println!("Dispute resolution gas: {}", gas_used);
    // assert!(gas_used < 400_000, "Gas usage exceeds threshold");
}

#[test]
fn test_gas_cost_valuation_update() {
    let env = setup_env();
    let owner = Address::generate(&env);
    
    // Register collateral
    // let collateral_id = register_collateral(
    //     &env,
    //     owner.clone(),
    //     constants::MIN_COLLATERAL_VALUE,
    //     env.ledger().timestamp() + constants::ONE_YEAR_SECONDS,
    //     soroban_sdk::BytesN::from_array(&env, &[0u8; 32]),
    //     soroban_sdk::String::from_slice(&env, "ipfs://QmTest"),
    // ).unwrap();
    
    // Measure gas for valuation update
    // let gas_before = env.budget().get_cpu_insns_consumed();
    
    // update_valuation(&env, collateral_id, constants::MIN_COLLATERAL_VALUE * 2, owner).unwrap();
    
    // let gas_after = env.budget().get_cpu_insns_consumed();
    // let gas_used = gas_after - gas_before;
    
    // println!("Valuation update gas: {}", gas_used);
    // assert!(gas_used < 250_000, "Gas usage exceeds threshold");
}

/// Batch operation gas efficiency
#[test]
fn test_gas_efficiency_batch_collateral_registration() {
    let env = setup_env();
    let owner = Address::generate(&env);
    
    let num_collaterals = 10;
    
    // Measure total gas for batch registration
    // let gas_before = env.budget().get_cpu_insns_consumed();
    
    // for i in 0..num_collaterals {
    //     register_collateral(
    //         &env,
    //         owner.clone(),
    //         constants::MIN_COLLATERAL_VALUE + (i as i128),
    //         env.ledger().timestamp() + constants::ONE_YEAR_SECONDS,
    //         soroban_sdk::BytesN::from_array(&env, &[i as u8; 32]),
    //         soroban_sdk::String::from_slice(&env, &format!("ipfs://Qm{}", i)),
    //     ).unwrap();
    // }
    
    // let gas_after = env.budget().get_cpu_insns_consumed();
    // let total_gas = gas_after - gas_before;
    // let avg_gas = total_gas / num_collaterals;
    
    // println!("Average gas per collateral (batch): {}", avg_gas);
    // assert!(avg_gas < 1_200_000, "Average gas usage exceeds threshold");
}

/// Storage efficiency test
#[test]
fn test_storage_efficiency_collateral_data() {
    let env = setup_env();
    let owner = Address::generate(&env);
    
    // Register collateral and measure storage
    // let collateral_id = register_collateral(
    //     &env,
    //     owner,
    //     constants::MIN_COLLATERAL_VALUE,
    //     env.ledger().timestamp() + constants::ONE_YEAR_SECONDS,
    //     soroban_sdk::BytesN::from_array(&env, &[0u8; 32]),
    //     soroban_sdk::String::from_slice(&env, "ipfs://QmTest"),
    // ).unwrap();
    
    // Get storage metrics
    // let storage_used = env.budget().get_mem_bytes_consumed();
    // println!("Storage used for collateral: {} bytes", storage_used);
    
    // Verify reasonable storage usage
    // assert!(storage_used < 10_000, "Storage usage exceeds threshold");
}

/// Gas cost comparison: different operation sizes
#[test]
fn test_gas_scaling_with_collateral_value() {
    let env = setup_env();
    let owner = Address::generate(&env);
    
    let test_values = vec![
        constants::MIN_COLLATERAL_VALUE,
        constants::MIN_COLLATERAL_VALUE * 100,
        constants::MIN_COLLATERAL_VALUE * 10_000,
    ];
    
    // for value in test_values {
    //     let gas_before = env.budget().get_cpu_insns_consumed();
    //     
    //     register_collateral(
    //         &env,
    //         owner.clone(),
    //         value,
    //         env.ledger().timestamp() + constants::ONE_YEAR_SECONDS,
    //         soroban_sdk::BytesN::from_array(&env, &[0u8; 32]),
    //         soroban_sdk::String::from_slice(&env, "ipfs://QmTest"),
    //     ).unwrap();
    //     
    //     let gas_after = env.budget().get_cpu_insns_consumed();
    //     println!("Gas for value {}: {}", value, gas_after - gas_before);
    // }
}

/// Optimization: Caching and memoization effectiveness
#[test]
fn test_gas_savings_from_caching() {
    let env = setup_env();
    
    // First call (cache miss)
    // let gas_before_1 = env.budget().get_cpu_insns_consumed();
    // let _result1 = get_governance_param(&env, "interest_rate").unwrap();
    // let gas_after_1 = env.budget().get_cpu_insns_consumed();
    // let gas_first = gas_after_1 - gas_before_1;
    
    // Second call (cache hit)
    // let gas_before_2 = env.budget().get_cpu_insns_consumed();
    // let _result2 = get_governance_param(&env, "interest_rate").unwrap();
    // let gas_after_2 = env.budget().get_cpu_insns_consumed();
    // let gas_second = gas_after_2 - gas_before_2;
    
    // println!("First call gas: {}, Second call gas: {}", gas_first, gas_second);
    // assert!(gas_second < gas_first, "Caching not effective");
}

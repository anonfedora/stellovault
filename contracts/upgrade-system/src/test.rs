#![cfg(test)]

use crate::contract::{UpgradeSystem, UpgradeSystemClient};
use crate::types::{ImplementationMetadata, UpgradeAction, Version};
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, Ledger as _},
    Address, Env,
};

fn version(major: u32, minor: u32, patch: u32) -> Version {
    Version {
        major,
        minor,
        patch,
    }
}

fn metadata(
    implementation: &Address,
    version: Version,
    interface_hash: u64,
    storage_hash: u64,
    has_migration: bool,
    breaking_change: bool,
) -> ImplementationMetadata {
    ImplementationMetadata {
        implementation: implementation.clone(),
        contract_kind: symbol_short!("loan"),
        version,
        min_supported_version: Version {
            major: 1,
            minor: 0,
            patch: 0,
        },
        interface_hash,
        storage_hash,
        has_migration,
        breaking_change,
    }
}

fn setup() -> (Env, Address, Address, Address, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|li| {
        li.timestamp = 1_000;
    });

    let contract_id = env.register(UpgradeSystem, ());
    let client = UpgradeSystemClient::new(&env, &contract_id);
    let governance = Address::generate(&env);
    let validator = Address::generate(&env);
    let emergency_admin = Address::generate(&env);
    let proxy = Address::generate(&env);
    let impl_v1 = Address::generate(&env);
    let impl_v2 = Address::generate(&env);

    client.initialize(
        &governance,
        &validator,
        &emergency_admin,
        &60,
        &86_400,
        &3_600,
    );

    client.register_implementation(&metadata(&impl_v1, version(1, 0, 0), 77, 88, false, false));
    client.register_implementation(&metadata(&impl_v2, version(1, 1, 0), 77, 88, false, false));
    client.register_contract(&proxy, &impl_v1);

    (
        env,
        contract_id,
        governance,
        validator,
        emergency_admin,
        proxy,
        impl_v2,
    )
}

#[test]
fn upgrade_requires_validation_and_governance_approval() {
    let (env, contract_id, _governance, _validator, _admin, proxy, impl_v2) = setup();
    let client = UpgradeSystemClient::new(&env, &contract_id);

    let proposal_id = client.propose_upgrade(&proxy, &impl_v2);

    assert!(client.try_execute_upgrade(&proposal_id).is_err());

    client.validate_upgrade(&proposal_id);
    assert!(client.try_execute_upgrade(&proposal_id).is_err());

    client.approve_upgrade(&proposal_id);
    assert!(client.try_execute_upgrade(&proposal_id).is_err());
}

#[test]
fn validation_blocks_breaking_change() {
    let (env, contract_id, _governance, _validator, _admin, proxy, _impl_v2) = setup();
    let bad_impl = Address::generate(&env);
    let client = UpgradeSystemClient::new(&env, &contract_id);

    client.register_implementation(&metadata(&bad_impl, version(2, 0, 0), 55, 99, false, true));

    let proposal_id = client.propose_upgrade(&proxy, &bad_impl);
    let report = client.simulate_upgrade(&proposal_id);

    assert!(!report.contract_kind_match || !report.interface_match || report.breaking_change);
    assert!(client.try_validate_upgrade(&proposal_id).is_err());
}

#[test]
fn execute_upgrade_updates_managed_state_and_audit_trail() {
    let (env, contract_id, _governance, _validator, _admin, proxy, impl_v2) = setup();
    let client = UpgradeSystemClient::new(&env, &contract_id);

    let proposal_id = client.propose_upgrade(&proxy, &impl_v2);
    let report = client.validate_upgrade(&proposal_id);
    assert!(report.simulation_passed);
    client.approve_upgrade(&proposal_id);

    env.ledger().with_mut(|li| {
        li.timestamp += 61;
    });
    client.execute_upgrade(&proposal_id);

    let state = client.get_managed_state(&proxy);
    assert_eq!(state.current_implementation, impl_v2);
    assert_eq!(state.current_version, version(1, 1, 0));

    let history = client.get_upgrade_history(&proxy);
    assert!(history.len() >= 4);
    let notes = client.get_notifications(&proxy);
    assert!(notes.len() >= 4);
}

#[test]
fn rollback_restores_previous_implementation_within_window() {
    let (env, contract_id, _governance, _validator, _admin, proxy, impl_v2) = setup();
    let client = UpgradeSystemClient::new(&env, &contract_id);

    let proposal_id = client.propose_upgrade(&proxy, &impl_v2);
    client.validate_upgrade(&proposal_id);
    client.approve_upgrade(&proposal_id);
    env.ledger().with_mut(|li| {
        li.timestamp += 61;
    });
    client.execute_upgrade(&proposal_id);

    client.rollback_upgrade(&proxy);

    let state = client.get_managed_state(&proxy);
    assert_eq!(state.current_version, version(1, 0, 0));

    let history = client.get_upgrade_history(&proxy);
    let last = history.get(history.len() - 1).unwrap();
    assert_eq!(last.action, UpgradeAction::RolledBack);
}

#[test]
fn rollback_window_is_enforced() {
    let (env, contract_id, _governance, _validator, _admin, proxy, impl_v2) = setup();
    let client = UpgradeSystemClient::new(&env, &contract_id);

    let proposal_id = client.propose_upgrade(&proxy, &impl_v2);
    client.validate_upgrade(&proposal_id);
    client.approve_upgrade(&proposal_id);
    env.ledger().with_mut(|li| {
        li.timestamp += 61;
    });
    client.execute_upgrade(&proposal_id);
    env.ledger().with_mut(|li| {
        li.timestamp += 3_601;
    });

    assert!(client.try_rollback_upgrade(&proxy).is_err());
}

#[test]
fn emergency_pause_and_emergency_execution_work() {
    let (env, contract_id, _governance, _validator, _admin, proxy, impl_v2) = setup();
    let client = UpgradeSystemClient::new(&env, &contract_id);

    client.emergency_pause(&proxy);
    let paused = client.get_managed_state(&proxy);
    assert!(paused.paused);

    assert!(client.try_propose_upgrade(&proxy, &impl_v2).is_err());

    let impl_v3 = Address::generate(&env);
    client.register_implementation(&metadata(&impl_v3, version(1, 2, 0), 77, 88, false, false));

    let proxy2 = Address::generate(&env);
    let impl_v1b = Address::generate(&env);
    client.register_implementation(&metadata(&impl_v1b, version(1, 0, 0), 77, 88, false, false));
    client.register_contract(&proxy2, &impl_v1b);
    let proposal_id = client.propose_upgrade(&proxy2, &impl_v3);
    client.validate_upgrade(&proposal_id);
    client.approve_upgrade(&proposal_id);
    client.mark_emergency_upgrade(&proposal_id);
    client.execute_upgrade(&proposal_id);

    let state = client.get_managed_state(&proxy2);
    assert_eq!(state.current_version, version(1, 2, 0));
}

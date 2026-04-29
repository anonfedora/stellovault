# Contract Upgrade System

This contract provides a governance-controlled upgrade manager for Soroban contracts that use a proxy or upgrade-admin pattern.

## Model

- `register_contract`: registers a managed proxy/contract and its active implementation metadata.
- `register_implementation`: registers candidate implementation metadata for compatibility checks.
- `propose_upgrade`: creates a scheduled upgrade proposal.
- `simulate_upgrade`: previews compatibility outcome without mutating upgrade state.
- `validate_upgrade`: stores a validation report from registered metadata.
- `approve_upgrade`: governance approval gate before execution.
- `execute_upgrade`: switches the active implementation after validation, approval, and timelock.
- `rollback_upgrade`: reverts to the previous implementation within the rollback window.
- `emergency_pause`: pauses a managed contract immediately.

## Compatibility Rules

Validation succeeds only when:

- contract kind matches
- interface hash matches
- storage hash matches, or the new implementation explicitly declares a migration path
- semantic version does not move backward
- semantic major version is unchanged
- current version is within the new implementation's supported range
- implementation is not flagged as breaking

## Auditability

Every proposal, validation, approval, execution, rollback, and emergency pause emits an event and appends to per-contract history.

## Emergency Flow

1. `emergency_pause(contract_address)`
2. inspect latest history / validation report
3. `rollback_upgrade(contract_address)` or execute a pre-approved emergency upgrade

## Notes

This contract stores the source of truth for upgrade governance and proxy routing metadata. The actual proxy contract is expected to read its active implementation from this manager or be controlled by an off-chain orchestrator that consumes emitted events.

use soroban_sdk::{contracttype, Address, Symbol};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImplementationMetadata {
    pub implementation: Address,
    pub contract_kind: Symbol,
    pub version: Version,
    pub min_supported_version: Version,
    pub interface_hash: u64,
    pub storage_hash: u64,
    pub has_migration: bool,
    pub breaking_change: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManagedContract {
    pub contract_address: Address,
    pub current_implementation: Address,
    pub current_version: Version,
    pub contract_kind: Symbol,
    pub interface_hash: u64,
    pub storage_hash: u64,
    pub paused: bool,
    pub last_upgrade_ts: u64,
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProposalStatus {
    Proposed = 0,
    Validated = 1,
    Approved = 2,
    Executed = 3,
    RolledBack = 4,
    Cancelled = 5,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpgradeProposal {
    pub id: u64,
    pub contract_address: Address,
    pub old_implementation: Address,
    pub new_implementation: Address,
    pub proposer: Address,
    pub proposed_at: u64,
    pub execute_after: u64,
    pub expires_at: u64,
    pub emergency: bool,
    pub approved: bool,
    pub executed: bool,
    pub status: ProposalStatus,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidationReport {
    pub proposal_id: u64,
    pub contract_kind_match: bool,
    pub interface_match: bool,
    pub storage_compatible: bool,
    pub version_forward: bool,
    pub version_supported: bool,
    pub breaking_change: bool,
    pub rollback_safe: bool,
    pub simulation_passed: bool,
    pub checked_at: u64,
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UpgradeAction {
    Proposed = 0,
    Validated = 1,
    Approved = 2,
    Executed = 3,
    RolledBack = 4,
    EmergencyPaused = 5,
    EmergencyExecuted = 6,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpgradeRecord {
    pub proposal_id: u64,
    pub action: UpgradeAction,
    pub actor: Address,
    pub implementation: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NotificationKind {
    UpgradeProposed = 0,
    UpgradeValidated = 1,
    UpgradeApproved = 2,
    UpgradeScheduled = 3,
    UpgradeExecuted = 4,
    UpgradeRolledBack = 5,
    EmergencyPaused = 6,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpgradeNotification {
    pub proposal_id: u64,
    pub contract_address: Address,
    pub kind: NotificationKind,
    pub scheduled_for: u64,
    pub created_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpgradeConfig {
    pub governance: Address,
    pub validator: Address,
    pub emergency_admin: Address,
    pub timelock_seconds: u64,
    pub proposal_ttl_seconds: u64,
    pub rollback_window_seconds: u64,
}

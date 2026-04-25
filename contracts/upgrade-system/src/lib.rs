#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol, Vec};

const EVT_INIT: Symbol = symbol_short!("up_init");
const EVT_NOTIFY: Symbol = symbol_short!("up_note");
const EVT_ACTION: Symbol = symbol_short!("up_act");

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContractError {
    Unauthorized = 1,
    AlreadyInitialized = 2,
    ContractNotRegistered = 3,
    ImplementationNotRegistered = 4,
    ProposalNotFound = 5,
    ProposalAlreadyProcessed = 6,
    ProposalNotValidated = 7,
    ProposalNotApproved = 8,
    UpgradeNotReady = 9,
    ValidationFailed = 10,
    ContractPaused = 11,
    NoRollbackAvailable = 12,
    RollbackWindowExpired = 13,
    InvalidVersion = 14,
    AlreadyRegistered = 15,
    ProposalExpired = 16,
}

impl From<ContractError> for soroban_sdk::Error {
    fn from(err: ContractError) -> Self {
        soroban_sdk::Error::from_contract_error(err as u32)
    }
}

impl From<soroban_sdk::Error> for ContractError {
    fn from(_: soroban_sdk::Error) -> Self {
        ContractError::Unauthorized
    }
}

impl From<&ContractError> for soroban_sdk::Error {
    fn from(err: &ContractError) -> Self {
        soroban_sdk::Error::from_contract_error(*err as u32)
    }
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
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

impl Default for Version {
    fn default() -> Self {
        Self {
            major: 0,
            minor: 0,
            patch: 0,
        }
    }
}

#[contracttype]
enum DataKey {
    Config,
    ProposalCount,
    Managed(Address),
    ImplMeta(Address),
    Proposal(u64),
    Validation(u64),
    History(Address),
    Notifications(Address),
}

#[contract]
pub struct UpgradeSystem;

#[contractimpl]
impl UpgradeSystem {
    pub fn initialize(
        env: Env,
        governance: Address,
        validator: Address,
        emergency_admin: Address,
        timelock_seconds: u64,
        proposal_ttl_seconds: u64,
        rollback_window_seconds: u64,
    ) -> Result<(), ContractError> {
        if env.storage().instance().has(&DataKey::Config) {
            return Err(ContractError::AlreadyInitialized);
        }

        let config = UpgradeConfig {
            governance: governance.clone(),
            validator,
            emergency_admin,
            timelock_seconds,
            proposal_ttl_seconds,
            rollback_window_seconds,
        };

        env.storage().instance().set(&DataKey::Config, &config);
        env.storage().instance().set(&DataKey::ProposalCount, &0u64);
        env.events().publish((EVT_INIT,), governance);
        Ok(())
    }

    pub fn register_implementation(
        env: Env,
        metadata: ImplementationMetadata,
    ) -> Result<(), ContractError> {
        let config = Self::get_config(env.clone())?;
        config.governance.require_auth();
        Self::validate_version_shape(&metadata.version)?;
        Self::validate_version_shape(&metadata.min_supported_version)?;

        let key = DataKey::ImplMeta(metadata.implementation.clone());
        if env.storage().persistent().has(&key) {
            return Err(ContractError::AlreadyRegistered);
        }

        env.storage().persistent().set(&key, &metadata);
        Ok(())
    }

    pub fn register_contract(
        env: Env,
        contract_address: Address,
        initial_implementation: Address,
    ) -> Result<(), ContractError> {
        let config = Self::get_config(env.clone())?;
        config.governance.require_auth();

        let contract_key = DataKey::Managed(contract_address.clone());
        if env.storage().persistent().has(&contract_key) {
            return Err(ContractError::AlreadyRegistered);
        }

        let metadata = Self::get_impl_metadata(&env, &initial_implementation)?;
        let managed = ManagedContract {
            contract_address: contract_address.clone(),
            current_implementation: initial_implementation.clone(),
            current_version: metadata.version.clone(),
            contract_kind: metadata.contract_kind,
            interface_hash: metadata.interface_hash,
            storage_hash: metadata.storage_hash,
            paused: false,
            last_upgrade_ts: Self::now(&env),
        };

        env.storage().persistent().set(&contract_key, &managed);
        Self::append_history(
            &env,
            &contract_address,
            UpgradeRecord {
                proposal_id: 0,
                action: UpgradeAction::Executed,
                actor: config.governance.clone(),
                implementation: initial_implementation,
                timestamp: Self::now(&env),
            },
        );
        Ok(())
    }

    pub fn propose_upgrade(
        env: Env,
        contract_address: Address,
        new_implementation: Address,
    ) -> Result<u64, ContractError> {
        let config = Self::get_config(env.clone())?;
        config.governance.require_auth();

        let managed = Self::get_managed_contract(&env, &contract_address)?;
        if managed.paused {
            return Err(ContractError::ContractPaused);
        }
        let _candidate = Self::get_impl_metadata(&env, &new_implementation)?;

        let proposal_id = Self::next_proposal_id(&env);
        let now = Self::now(&env);
        let proposal = UpgradeProposal {
            id: proposal_id,
            contract_address: contract_address.clone(),
            old_implementation: managed.current_implementation.clone(),
            new_implementation: new_implementation.clone(),
            proposer: config.governance.clone(),
            proposed_at: now,
            execute_after: now + config.timelock_seconds,
            expires_at: now + config.proposal_ttl_seconds,
            emergency: false,
            approved: false,
            executed: false,
            status: ProposalStatus::Proposed,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_id), &proposal);

        Self::append_history(
            &env,
            &contract_address,
            UpgradeRecord {
                proposal_id,
                action: UpgradeAction::Proposed,
                actor: config.governance.clone(),
                implementation: new_implementation.clone(),
                timestamp: now,
            },
        );
        Self::append_notification(
            &env,
            &contract_address,
            UpgradeNotification {
                proposal_id,
                contract_address: contract_address.clone(),
                kind: NotificationKind::UpgradeProposed,
                scheduled_for: proposal.execute_after,
                created_at: now,
            },
        );
        Self::append_notification(
            &env,
            &contract_address,
            UpgradeNotification {
                proposal_id,
                contract_address: contract_address.clone(),
                kind: NotificationKind::UpgradeScheduled,
                scheduled_for: proposal.execute_after,
                created_at: now,
            },
        );
        Ok(proposal_id)
    }

    pub fn simulate_upgrade(env: Env, proposal_id: u64) -> Result<ValidationReport, ContractError> {
        let proposal = Self::get_proposal(&env, proposal_id)?;
        let managed = Self::get_managed_contract(&env, &proposal.contract_address)?;
        let candidate = Self::get_impl_metadata(&env, &proposal.new_implementation)?;
        Ok(Self::build_validation_report(
            &env,
            proposal_id,
            &managed,
            &candidate,
        ))
    }

    pub fn validate_upgrade(env: Env, proposal_id: u64) -> Result<ValidationReport, ContractError> {
        let config = Self::get_config(env.clone())?;
        config.validator.require_auth();

        let mut proposal = Self::get_proposal(&env, proposal_id)?;
        Self::ensure_active_proposal(&env, &proposal)?;

        let managed = Self::get_managed_contract(&env, &proposal.contract_address)?;
        let candidate = Self::get_impl_metadata(&env, &proposal.new_implementation)?;
        let report = Self::build_validation_report(&env, proposal_id, &managed, &candidate);

        if !Self::report_passes(&report) {
            return Err(ContractError::ValidationFailed);
        }

        proposal.status = ProposalStatus::Validated;
        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_id), &proposal);
        env.storage()
            .persistent()
            .set(&DataKey::Validation(proposal_id), &report);

        Self::append_history(
            &env,
            &proposal.contract_address,
            UpgradeRecord {
                proposal_id,
                action: UpgradeAction::Validated,
                actor: config.validator.clone(),
                implementation: proposal.new_implementation.clone(),
                timestamp: Self::now(&env),
            },
        );
        Self::append_notification(
            &env,
            &proposal.contract_address,
            UpgradeNotification {
                proposal_id,
                contract_address: proposal.contract_address.clone(),
                kind: NotificationKind::UpgradeValidated,
                scheduled_for: proposal.execute_after,
                created_at: Self::now(&env),
            },
        );

        Ok(report)
    }

    pub fn approve_upgrade(env: Env, proposal_id: u64) -> Result<(), ContractError> {
        let config = Self::get_config(env.clone())?;
        config.governance.require_auth();

        let mut proposal = Self::get_proposal(&env, proposal_id)?;
        Self::ensure_active_proposal(&env, &proposal)?;
        if proposal.status != ProposalStatus::Validated {
            return Err(ContractError::ProposalNotValidated);
        }
        if !env
            .storage()
            .persistent()
            .has(&DataKey::Validation(proposal_id))
        {
            return Err(ContractError::ProposalNotValidated);
        }

        proposal.approved = true;
        proposal.status = ProposalStatus::Approved;
        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_id), &proposal);

        Self::append_history(
            &env,
            &proposal.contract_address,
            UpgradeRecord {
                proposal_id,
                action: UpgradeAction::Approved,
                actor: config.governance.clone(),
                implementation: proposal.new_implementation.clone(),
                timestamp: Self::now(&env),
            },
        );
        Self::append_notification(
            &env,
            &proposal.contract_address,
            UpgradeNotification {
                proposal_id,
                contract_address: proposal.contract_address.clone(),
                kind: NotificationKind::UpgradeApproved,
                scheduled_for: proposal.execute_after,
                created_at: Self::now(&env),
            },
        );

        Ok(())
    }

    pub fn execute_upgrade(env: Env, proposal_id: u64) -> Result<(), ContractError> {
        let config = Self::get_config(env.clone())?;
        config.governance.require_auth();

        let mut proposal = Self::get_proposal(&env, proposal_id)?;
        Self::ensure_active_proposal(&env, &proposal)?;

        if proposal.executed {
            return Err(ContractError::ProposalAlreadyProcessed);
        }
        if !proposal.approved {
            return Err(ContractError::ProposalNotApproved);
        }

        let report: ValidationReport = env
            .storage()
            .persistent()
            .get(&DataKey::Validation(proposal_id))
            .ok_or(ContractError::ProposalNotValidated)?;
        if !Self::report_passes(&report) {
            return Err(ContractError::ValidationFailed);
        }

        let now = Self::now(&env);
        let managed = Self::get_managed_contract(&env, &proposal.contract_address)?;
        if !proposal.emergency && now < proposal.execute_after {
            return Err(ContractError::UpgradeNotReady);
        }

        let candidate = Self::get_impl_metadata(&env, &proposal.new_implementation)?;
        let updated = ManagedContract {
            contract_address: proposal.contract_address.clone(),
            current_implementation: proposal.new_implementation.clone(),
            current_version: candidate.version,
            contract_kind: managed.contract_kind,
            interface_hash: candidate.interface_hash,
            storage_hash: candidate.storage_hash,
            paused: false,
            last_upgrade_ts: now,
        };

        env.storage().persistent().set(
            &DataKey::Managed(proposal.contract_address.clone()),
            &updated,
        );

        proposal.executed = true;
        proposal.status = ProposalStatus::Executed;
        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_id), &proposal);

        Self::append_history(
            &env,
            &proposal.contract_address,
            UpgradeRecord {
                proposal_id,
                action: if proposal.emergency {
                    UpgradeAction::EmergencyExecuted
                } else {
                    UpgradeAction::Executed
                },
                actor: config.governance.clone(),
                implementation: proposal.new_implementation.clone(),
                timestamp: now,
            },
        );
        Self::append_notification(
            &env,
            &proposal.contract_address,
            UpgradeNotification {
                proposal_id,
                contract_address: proposal.contract_address.clone(),
                kind: NotificationKind::UpgradeExecuted,
                scheduled_for: now,
                created_at: now,
            },
        );

        Ok(())
    }

    pub fn rollback_upgrade(env: Env, contract_address: Address) -> Result<(), ContractError> {
        let config = Self::get_config(env.clone())?;
        config.emergency_admin.require_auth();

        let mut managed = Self::get_managed_contract(&env, &contract_address)?;
        let history = Self::get_upgrade_history(env.clone(), contract_address.clone());
        let last_exec =
            Self::find_last_execution(&history).ok_or(ContractError::NoRollbackAvailable)?;
        let previous_exec =
            Self::find_previous_execution(&history).ok_or(ContractError::NoRollbackAvailable)?;

        let now = Self::now(&env);
        if now > managed.last_upgrade_ts + config.rollback_window_seconds {
            return Err(ContractError::RollbackWindowExpired);
        }

        let previous_metadata = Self::get_impl_metadata(&env, &previous_exec.implementation)?;
        managed.current_implementation = previous_exec.implementation.clone();
        managed.current_version = previous_metadata.version;
        managed.interface_hash = previous_metadata.interface_hash;
        managed.storage_hash = previous_metadata.storage_hash;
        managed.paused = false;
        managed.last_upgrade_ts = now;
        env.storage()
            .persistent()
            .set(&DataKey::Managed(contract_address.clone()), &managed);

        let rollback_record = UpgradeRecord {
            proposal_id: last_exec.proposal_id,
            action: UpgradeAction::RolledBack,
            actor: config.emergency_admin.clone(),
            implementation: previous_exec.implementation.clone(),
            timestamp: now,
        };
        Self::append_history(&env, &contract_address, rollback_record);
        Self::append_notification(
            &env,
            &contract_address,
            UpgradeNotification {
                proposal_id: last_exec.proposal_id,
                contract_address: contract_address.clone(),
                kind: NotificationKind::UpgradeRolledBack,
                scheduled_for: now,
                created_at: now,
            },
        );
        Ok(())
    }

    pub fn emergency_pause(env: Env, contract_address: Address) -> Result<(), ContractError> {
        let config = Self::get_config(env.clone())?;
        config.emergency_admin.require_auth();

        let mut managed = Self::get_managed_contract(&env, &contract_address)?;
        managed.paused = true;
        env.storage()
            .persistent()
            .set(&DataKey::Managed(contract_address.clone()), &managed);

        Self::append_history(
            &env,
            &contract_address,
            UpgradeRecord {
                proposal_id: 0,
                action: UpgradeAction::EmergencyPaused,
                actor: config.emergency_admin.clone(),
                implementation: managed.current_implementation,
                timestamp: Self::now(&env),
            },
        );
        Self::append_notification(
            &env,
            &contract_address,
            UpgradeNotification {
                proposal_id: 0,
                contract_address: contract_address.clone(),
                kind: NotificationKind::EmergencyPaused,
                scheduled_for: Self::now(&env),
                created_at: Self::now(&env),
            },
        );
        Ok(())
    }

    pub fn mark_emergency_upgrade(env: Env, proposal_id: u64) -> Result<(), ContractError> {
        let config = Self::get_config(env.clone())?;
        config.emergency_admin.require_auth();

        let mut proposal = Self::get_proposal(&env, proposal_id)?;
        proposal.emergency = true;
        proposal.execute_after = Self::now(&env);
        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_id), &proposal);
        Ok(())
    }

    pub fn get_upgrade_history(env: Env, contract_address: Address) -> Vec<UpgradeRecord> {
        env.storage()
            .persistent()
            .get(&DataKey::History(contract_address))
            .unwrap_or(Vec::new(&env))
    }

    pub fn get_notifications(env: Env, contract_address: Address) -> Vec<UpgradeNotification> {
        env.storage()
            .persistent()
            .get(&DataKey::Notifications(contract_address))
            .unwrap_or(Vec::new(&env))
    }

    pub fn get_managed_state(
        env: Env,
        contract_address: Address,
    ) -> Result<ManagedContract, ContractError> {
        Self::get_managed_contract(&env, &contract_address)
    }

    pub fn get_proposal_details(
        env: Env,
        proposal_id: u64,
    ) -> Result<UpgradeProposal, ContractError> {
        Self::get_proposal(&env, proposal_id)
    }

    pub fn get_validation_report(
        env: Env,
        proposal_id: u64,
    ) -> Result<ValidationReport, ContractError> {
        env.storage()
            .persistent()
            .get(&DataKey::Validation(proposal_id))
            .ok_or(ContractError::ProposalNotValidated)
    }

    pub fn get_config_view(env: Env) -> Result<UpgradeConfig, ContractError> {
        Self::get_config(env)
    }

    fn get_config(env: Env) -> Result<UpgradeConfig, ContractError> {
        env.storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(ContractError::Unauthorized)
    }

    fn get_managed_contract(
        env: &Env,
        contract_address: &Address,
    ) -> Result<ManagedContract, ContractError> {
        env.storage()
            .persistent()
            .get(&DataKey::Managed(contract_address.clone()))
            .ok_or(ContractError::ContractNotRegistered)
    }

    fn get_impl_metadata(
        env: &Env,
        implementation: &Address,
    ) -> Result<ImplementationMetadata, ContractError> {
        env.storage()
            .persistent()
            .get(&DataKey::ImplMeta(implementation.clone()))
            .ok_or(ContractError::ImplementationNotRegistered)
    }

    fn get_proposal(env: &Env, proposal_id: u64) -> Result<UpgradeProposal, ContractError> {
        env.storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .ok_or(ContractError::ProposalNotFound)
    }

    fn build_validation_report(
        env: &Env,
        proposal_id: u64,
        managed: &ManagedContract,
        candidate: &ImplementationMetadata,
    ) -> ValidationReport {
        let contract_kind_match = managed.contract_kind == candidate.contract_kind;
        let interface_match = managed.interface_hash == candidate.interface_hash;
        let storage_compatible =
            managed.storage_hash == candidate.storage_hash || candidate.has_migration;
        let version_forward =
            Self::version_key(&candidate.version) >= Self::version_key(&managed.current_version);
        let version_supported = Self::version_key(&managed.current_version)
            >= Self::version_key(&candidate.min_supported_version)
            && candidate.version.major == managed.current_version.major;
        let breaking_change = candidate.breaking_change;
        let rollback_safe = contract_kind_match && interface_match;
        let simulation_passed = contract_kind_match
            && interface_match
            && storage_compatible
            && version_forward
            && version_supported
            && !breaking_change;

        ValidationReport {
            proposal_id,
            contract_kind_match,
            interface_match,
            storage_compatible,
            version_forward,
            version_supported,
            breaking_change,
            rollback_safe,
            simulation_passed,
            checked_at: Self::now(env),
        }
    }

    fn report_passes(report: &ValidationReport) -> bool {
        report.contract_kind_match
            && report.interface_match
            && report.storage_compatible
            && report.version_forward
            && report.version_supported
            && !report.breaking_change
            && report.rollback_safe
            && report.simulation_passed
    }

    fn ensure_active_proposal(env: &Env, proposal: &UpgradeProposal) -> Result<(), ContractError> {
        if proposal.executed
            || proposal.status == ProposalStatus::Executed
            || proposal.status == ProposalStatus::RolledBack
            || proposal.status == ProposalStatus::Cancelled
        {
            return Err(ContractError::ProposalAlreadyProcessed);
        }
        if Self::now(env) > proposal.expires_at {
            return Err(ContractError::ProposalExpired);
        }
        Ok(())
    }

    fn append_history(env: &Env, contract_address: &Address, record: UpgradeRecord) {
        let mut history: Vec<UpgradeRecord> = env
            .storage()
            .persistent()
            .get(&DataKey::History(contract_address.clone()))
            .unwrap_or(Vec::new(env));
        history.push_back(record.clone());
        env.storage()
            .persistent()
            .set(&DataKey::History(contract_address.clone()), &history);
        env.events()
            .publish((EVT_ACTION, contract_address.clone()), record);
    }

    fn append_notification(env: &Env, contract_address: &Address, note: UpgradeNotification) {
        let mut notes: Vec<UpgradeNotification> = env
            .storage()
            .persistent()
            .get(&DataKey::Notifications(contract_address.clone()))
            .unwrap_or(Vec::new(env));
        notes.push_back(note.clone());
        env.storage()
            .persistent()
            .set(&DataKey::Notifications(contract_address.clone()), &notes);
        env.events()
            .publish((EVT_NOTIFY, contract_address.clone()), note);
    }

    fn next_proposal_id(env: &Env) -> u64 {
        let id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::ProposalCount)
            .unwrap_or(0u64);
        env.storage()
            .instance()
            .set(&DataKey::ProposalCount, &(id + 1));
        id
    }

    fn find_last_execution(history: &Vec<UpgradeRecord>) -> Option<UpgradeRecord> {
        let mut idx = history.len();
        while idx > 0 {
            idx -= 1;
            let record = history.get(idx).unwrap();
            if record.action == UpgradeAction::Executed
                || record.action == UpgradeAction::EmergencyExecuted
            {
                return Some(record);
            }
        }
        None
    }

    fn find_previous_execution(history: &Vec<UpgradeRecord>) -> Option<UpgradeRecord> {
        let mut found_first = false;
        let mut idx = history.len();
        while idx > 0 {
            idx -= 1;
            let record = history.get(idx).unwrap();
            if record.action == UpgradeAction::Executed
                || record.action == UpgradeAction::EmergencyExecuted
            {
                if found_first {
                    return Some(record);
                }
                found_first = true;
            }
        }
        None
    }

    fn validate_version_shape(version: &Version) -> Result<(), ContractError> {
        if version.major == u32::MAX || version.minor == u32::MAX || version.patch == u32::MAX {
            return Err(ContractError::InvalidVersion);
        }
        Ok(())
    }

    fn version_key(version: &Version) -> u128 {
        ((version.major as u128) << 64) | ((version.minor as u128) << 32) | version.patch as u128
    }

    fn now(env: &Env) -> u64 {
        env.ledger().timestamp()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{
        testutils::{Address as _, Ledger as _},
        Env,
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
        client.register_implementation(&metadata(
            &impl_v1b,
            version(1, 0, 0),
            77,
            88,
            false,
            false,
        ));
        client.register_contract(&proxy2, &impl_v1b);
        let proposal_id = client.propose_upgrade(&proxy2, &impl_v3);
        client.validate_upgrade(&proposal_id);
        client.approve_upgrade(&proposal_id);
        client.mark_emergency_upgrade(&proposal_id);
        client.execute_upgrade(&proposal_id);

        let state = client.get_managed_state(&proxy2);
        assert_eq!(state.current_version, version(1, 2, 0));
    }
}

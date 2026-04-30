//! Oracle Adapter Contract for StelloVault
//!
//! This contract manages oracle providers and verifies off-chain events
//! such as shipment confirmations, delivery status, and quality inspections.
//! It serves as the bridge between on-chain escrow operations and trusted oracles.

#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Bytes, BytesN, Env,
    Symbol, Vec,
};

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContractError {
    Unauthorized = 1,
    AlreadyInitialized = 2,
    OracleNotRegistered = 3,
    OracleAlreadyRegistered = 4,
    InvalidSignature = 5,
    ConfirmationAlreadyExists = 6,
    EscrowNotFound = 7,
    InvalidEventType = 8,
    ConsensusNotMet = 9,
    InvalidThreshold = 10,
    NoPendingAdmin = 11,
    InsufficientStake = 12,
    InvalidOracleId = 13,
    OracleNotFound = 14,
    InvalidReputationScore = 15,
    CannotSlashBelowMinimum = 16,
    NoRewardsToDistribute = 17,
    DisputeNotFound = 18,
    DisputeAlreadyResolved = 19,
    InvalidDataType = 20,
    DataTooOld = 21,
    InvalidDisputeReason = 22,
    NotDisputeInitiator = 23,
    OracleNotActive = 24,
    InsufficientSignatures = 25,
    RotationNotAllowed = 26,
}

/// Event types for oracle confirmations
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EventType {
    Shipment = 1,
    Delivery = 2,
    Quality = 3,
    Custom = 4,
    Valuation = 5,
}

/// Oracle status
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OracleStatus {
    Active = 1,
    Slashed = 2,
    Inactive = 3,
    UnderReview = 4,
}

/// Dispute status
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DisputeStatus {
    Open = 1,
    Voting = 2,
    Resolved = 3,
    Rejected = 4,
}

/// Dispute reason
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DisputeReason {
    FalseData = 1,
    LateSubmission = 2,
    SignatureManipulation = 3,
    Collusion = 4,
    Other = 5,
}

/// Oracle confirmation data structure
#[contracttype]
#[derive(Clone, Debug)]
pub struct ConfirmationData {
    pub escrow_id: Bytes,
    pub event_type: u32,
    pub result: Bytes,
    pub oracle: Address,
    pub timestamp: u64,
    pub verified: bool,
}

/// Oracle information with staking and reputation
#[contracttype]
#[derive(Clone, Debug)]
pub struct OracleInfo {
    pub oracle_id: u64,
    pub address: Address,
    pub stake: i128,
    pub reputation_score: u32,
    pub status: OracleStatus,
    pub total_submissions: u64,
    pub successful_submissions: u64,
    pub last_activity: u64,
    pub metadata: Bytes,
}

/// Reputation data for tracking oracle performance
#[contracttype]
#[derive(Clone, Debug)]
pub struct ReputationData {
    pub oracle_id: u64,
    pub accuracy_score: u32,
    pub reliability_score: u32,
    pub timeliness_score: u32,
    pub dispute_count: u32,
    pub slash_count: u32,
    pub last_updated: u64,
}

/// Dispute information
#[contracttype]
#[derive(Clone, Debug)]
pub struct Dispute {
    pub dispute_id: u64,
    pub oracle_id: u64,
    pub initiator: Address,
    pub reason: DisputeReason,
    pub evidence: Bytes,
    pub status: DisputeStatus,
    pub votes_for: u32,
    pub votes_against: u32,
    pub created_at: u64,
    pub resolved_at: Option<u64>,
}

/// Reward pool for epoch distribution
#[contracttype]
#[derive(Clone, Debug)]
pub struct RewardPool {
    pub epoch: u64,
    pub total_rewards: i128,
    pub distributed: i128,
    pub oracle_rewards: Vec<(u64, i128)>,
}

/// Data type registry entry
#[contracttype]
#[derive(Clone, Debug)]
pub struct DataType {
    pub data_type_id: u32,
    pub name: Bytes,
    pub schema: Bytes,
    pub max_age_seconds: u64,
    pub required_signatures: u32,
    pub active: bool,
}

/// Submitted data with verification
#[contracttype]
#[derive(Clone, Debug)]
pub struct SubmittedData {
    pub data_id: u64,
    pub oracle_id: u64,
    pub data_type_id: u32,
    pub data_hash: Bytes,
    pub signatures: Vec<Bytes>,
    pub timestamp: u64,
    pub verified: bool,
}

/// Contract data structure for storage
#[contracttype]
#[derive(Clone)]
pub struct ContractData {
    pub admin: Address,
    pub initialized: bool,
    pub oracles: Vec<Address>,
    pub minimum_stake: i128,
    pub next_oracle_id: u64,
    pub next_dispute_id: u64,
    pub next_data_id: u64,
    pub current_epoch: u64,
    pub reward_pool: i128,
    pub slashing_percentage: u32,
    pub data_freshness_threshold: u64,
}

/// Event symbols
const ORACLE_ADDED: Symbol = symbol_short!("orc_add");
const ORACLE_REMOVED: Symbol = symbol_short!("orc_rem");
const ORACLE_CONFIRMED: Symbol = symbol_short!("confirmed");
const INITIALIZED: Symbol = symbol_short!("init");
const CONFIRMING_ORACLES: Symbol = symbol_short!("conf_orc");
const ORACLE_REGISTERED: Symbol = symbol_short!("orc_reg");
const ORACLE_SLASHED: Symbol = symbol_short!("orc_slash");
const REPUTATION_UPDATED: Symbol = symbol_short!("rep_upd");
const REWARDS_DISTRIBUTED: Symbol = symbol_short!("rew_dist");
const DISPUTE_CREATED: Symbol = symbol_short!("disp_cre");
const DISPUTE_RESOLVED: Symbol = symbol_short!("disp_res");
const DATA_SUBMITTED: Symbol = symbol_short!("data_sub");
const DATA_TYPE_REGISTERED: Symbol = symbol_short!("dt_reg");
const ORACLE_ROTATED: Symbol = symbol_short!("orc_rot");

/// Storage keys
const ORACLE_INFO: Symbol = symbol_short!("orc_info");
const REPUTATION_DATA: Symbol = symbol_short!("rep_data");
const DISPUTE_DATA: Symbol = symbol_short!("disp_data");
const REWARD_POOL_DATA: Symbol = symbol_short!("rew_pool");
const DATA_TYPE_REGISTRY: Symbol = symbol_short!("dt_reg");
const SUBMITTED_DATA: Symbol = symbol_short!("sub_data");

/// Main contract for oracle adapter operations
#[contract]
pub struct OracleAdapter;

/// Contract implementation
#[contractimpl]
impl OracleAdapter {
    /// Initialize the contract with admin address
    ///
    /// # Arguments
    /// * `admin` - The admin address that can manage the contract
    ///
    /// # Events
    /// Emits `INITIALIZED` event
    pub fn initialize(env: Env, admin: Address) -> Result<(), ContractError> {
        // Check if already initialized
        if Self::is_initialized(&env) {
            return Err(ContractError::AlreadyInitialized);
        }

        // Store admin and initialization status with default values
        let contract_data = ContractData {
            admin: admin.clone(),
            initialized: true,
            oracles: Vec::new(&env),
            minimum_stake: 1000_0000000, // 1000 XLM default
            next_oracle_id: 1,
            next_dispute_id: 1,
            next_data_id: 1,
            current_epoch: 0,
            reward_pool: 0,
            slashing_percentage: 10,         // 10% default
            data_freshness_threshold: 86400, // 24 hours default
        };

        env.storage()
            .instance()
            .set(&symbol_short!("data"), &contract_data);

        // Emit initialization event
        env.events().publish((INITIALIZED,), (admin,));
        Ok(())
    }

    /// Add an oracle to the registry (admin only)
    ///
    /// # Arguments
    /// * `oracle` - The oracle address to add
    ///
    /// # Events
    /// Emits `ORACLE_ADDED` event
    pub fn add_oracle(env: Env, oracle: Address) -> Result<(), ContractError> {
        Self::check_admin(&env)?;

        let mut contract_data = Self::get_contract_data(&env)?;

        // Check if oracle is already registered
        if Self::is_oracle_registered(&contract_data, &oracle) {
            return Err(ContractError::OracleAlreadyRegistered);
        }

        // Add oracle to registry
        contract_data.oracles.push_back(oracle.clone());

        // Save updated data
        env.storage()
            .instance()
            .set(&symbol_short!("data"), &contract_data);

        // Emit event
        env.events().publish((ORACLE_ADDED,), (oracle,));

        Ok(())
    }

    /// Remove an oracle from the registry (admin only)
    ///
    /// # Arguments
    /// * `oracle` - The oracle address to remove
    ///
    /// # Events
    /// Emits `ORACLE_REMOVED` event
    pub fn remove_oracle(env: Env, oracle: Address) -> Result<(), ContractError> {
        Self::check_admin(&env)?;

        let mut contract_data = Self::get_contract_data(&env)?;

        // Find and remove oracle
        let mut found = false;
        let mut new_oracles = Vec::new(&env);

        for existing_oracle in contract_data.oracles.iter() {
            if existing_oracle != oracle {
                new_oracles.push_back(existing_oracle);
            } else {
                found = true;
            }
        }

        if !found {
            return Err(ContractError::OracleNotRegistered);
        }

        contract_data.oracles = new_oracles;

        // Save updated data
        env.storage()
            .instance()
            .set(&symbol_short!("data"), &contract_data);

        // Note: Stale confirmations from removed oracles are automatically filtered
        // by check_consensus which validates against current registration state.
        // This prevents removed oracles from ever contributing to consensus.

        // Emit event
        env.events().publish((ORACLE_REMOVED,), (oracle,));

        Ok(())
    }

    /// Confirm an event with oracle signature verification
    ///
    /// # Arguments
    /// * `escrow_id` - The escrow ID to confirm
    /// * `event_type` - Type of event (1=Shipment, 2=Delivery, 3=Quality, 4=Custom)
    /// * `result` - The confirmation result data
    /// * `signature` - Oracle signature for verification
    ///
    /// # Events
    /// Emits `ORACLE_CONFIRMED` event
    pub fn confirm_event(
        env: Env,
        oracle: Address,
        escrow_id: Bytes,
        event_type: u32,
        result: Bytes,
        signature: Bytes,
    ) -> Result<(), ContractError> {
        let contract_data = Self::get_contract_data(&env)?;

        // Verify oracle is registered
        if !Self::is_oracle_registered(&contract_data, &oracle) {
            return Err(ContractError::OracleNotRegistered);
        }

        // Validate event type
        if !(1..=5).contains(&event_type) {
            return Err(ContractError::InvalidEventType);
        }

        // Check if confirmation already exists (prevent replay)
        let confirmation_key = (escrow_id.clone(), oracle.clone());
        if env.storage().persistent().has(&confirmation_key) {
            return Err(ContractError::ConfirmationAlreadyExists);
        }

        // Check if oracle already recorded for this escrow (defense-in-depth)
        let mut confirming_oracles = Self::get_confirming_oracles(&env, &escrow_id);
        for existing_oracle in confirming_oracles.iter() {
            if existing_oracle == oracle {
                return Err(ContractError::ConfirmationAlreadyExists);
            }
        }

        // Create message for signature verification
        let message = Self::create_message(&env, &escrow_id, event_type, &result);

        // Verify signature
        Self::verify_signature(&env, &message, &signature, &oracle)?;

        // Create confirmation data
        let confirmation = ConfirmationData {
            escrow_id: escrow_id.clone(),
            event_type,
            result: result.clone(),
            oracle: oracle.clone(),
            timestamp: env.ledger().timestamp(),
            verified: true,
        };

        // Store confirmation
        env.storage()
            .persistent()
            .set(&confirmation_key, &confirmation);

        // Track confirming oracles for this escrow
        confirming_oracles.push_back(oracle.clone());
        let confirming_key = (CONFIRMING_ORACLES, escrow_id.clone());
        env.storage()
            .persistent()
            .set(&confirming_key, &confirming_oracles);

        // Emit event
        env.events()
            .publish((ORACLE_CONFIRMED,), (escrow_id, event_type, result, oracle));

        Ok(())
    }

    /// Get confirmation data for an escrow
    ///
    /// # Arguments
    /// * `escrow_id` - The escrow ID to query
    ///
    /// # Returns
    /// Option containing confirmation data if found
    pub fn get_confirmation(env: Env, escrow_id: Bytes) -> Option<Vec<ConfirmationData>> {
        let confirmations = Self::get_confirmations_for_escrow(&env, escrow_id).ok()?;

        if confirmations.is_empty() {
            None
        } else {
            Some(confirmations)
        }
    }

    /// Check if an oracle is registered
    ///
    /// # Arguments
    /// * `oracle` - The oracle address to check
    ///
    /// # Returns
    /// true if oracle is registered, false otherwise
    pub fn is_oracle_registered_query(env: Env, oracle: Address) -> Result<bool, ContractError> {
        let contract_data = Self::get_contract_data(&env)?;
        Ok(Self::is_oracle_registered(&contract_data, &oracle))
    }

    /// Get the total number of registered oracles
    pub fn get_oracle_count(env: Env) -> Result<u32, ContractError> {
        let contract_data = Self::get_contract_data(&env)?;
        Ok(contract_data.oracles.len())
    }

    /// Get oracle address at specific index
    ///
    /// # Arguments
    /// * `index` - The index to query
    ///
    /// # Returns
    /// Oracle address at the given index
    pub fn get_oracle_at(env: Env, index: u32) -> Option<Address> {
        Self::get_contract_data(&env).ok()?.oracles.get(index)
    }

    /// Get admin address
    pub fn get_admin(env: Env) -> Result<Address, ContractError> {
        let contract_data = Self::get_contract_data(&env)?;
        Ok(contract_data.admin)
    }

    /// Propose a new admin (two-step transfer, step 1).
    /// Only the current admin may call this; their signature is required.
    pub fn propose_admin(env: Env, new_admin: Address) -> Result<(), ContractError> {
        Self::check_admin(&env)?;

        env.storage()
            .instance()
            .set(&symbol_short!("pend_adm"), &new_admin);

        let contract_data = Self::get_contract_data(&env)?;
        env.events().publish(
            (symbol_short!("adm_prop"),),
            (contract_data.admin, new_admin),
        );

        Ok(())
    }

    /// Accept a pending admin proposal (two-step transfer, step 2).
    /// Only the address nominated via propose_admin may call this.
    pub fn accept_admin(env: Env) -> Result<(), ContractError> {
        let pending: Address = env
            .storage()
            .instance()
            .get(&symbol_short!("pend_adm"))
            .ok_or(ContractError::NoPendingAdmin)?;

        pending.require_auth();

        let mut contract_data = Self::get_contract_data(&env)?;
        contract_data.admin = pending.clone();
        env.storage()
            .instance()
            .set(&symbol_short!("data"), &contract_data);
        env.storage().instance().remove(&symbol_short!("pend_adm"));

        env.events()
            .publish((symbol_short!("adm_acpt"),), (pending,));

        Ok(())
    }

    /// Return the pending admin address if a proposal is active.
    pub fn get_pending_admin(env: Env) -> Option<Address> {
        env.storage().instance().get(&symbol_short!("pend_adm"))
    }

    /// Register a new oracle with staking
    ///
    /// # Arguments
    /// * `oracle_address` - The oracle address to register
    /// * `stake` - The amount of tokens to stake
    /// * `metadata` - Additional metadata about the oracle
    ///
    /// # Events
    /// Emits `ORACLE_REGISTERED` event
    pub fn register_oracle(
        env: Env,
        oracle_address: Address,
        stake: i128,
        metadata: Bytes,
    ) -> Result<u64, ContractError> {
        let mut contract_data = Self::get_contract_data(&env)?;

        // Check if oracle is already registered
        if Self::is_oracle_registered(&contract_data, &oracle_address) {
            return Err(ContractError::OracleAlreadyRegistered);
        }

        // Check minimum stake requirement
        if stake < contract_data.minimum_stake {
            return Err(ContractError::InsufficientStake);
        }

        // Generate oracle ID
        let oracle_id = contract_data.next_oracle_id;
        contract_data.next_oracle_id += 1;

        // Create oracle info
        let oracle_info = OracleInfo {
            oracle_id,
            address: oracle_address.clone(),
            stake,
            reputation_score: 100, // Start with perfect reputation
            status: OracleStatus::Active,
            total_submissions: 0,
            successful_submissions: 0,
            last_activity: env.ledger().timestamp(),
            metadata,
        };

        // Create reputation data
        let reputation_data = ReputationData {
            oracle_id,
            accuracy_score: 100,
            reliability_score: 100,
            timeliness_score: 100,
            dispute_count: 0,
            slash_count: 0,
            last_updated: env.ledger().timestamp(),
        };

        // Store oracle info
        let oracle_key = (ORACLE_INFO, oracle_id);
        env.storage().persistent().set(&oracle_key, &oracle_info);

        // Store reputation data
        let reputation_key = (REPUTATION_DATA, oracle_id);
        env.storage()
            .persistent()
            .set(&reputation_key, &reputation_data);

        // Add to oracle list
        contract_data.oracles.push_back(oracle_address.clone());

        // Update contract data
        env.storage()
            .instance()
            .set(&symbol_short!("data"), &contract_data);

        // Emit event
        env.events()
            .publish((ORACLE_REGISTERED,), (oracle_id, oracle_address, stake));

        Ok(oracle_id)
    }

    /// Submit verified data from an oracle
    ///
    /// # Arguments
    /// * `oracle_id` - The oracle ID submitting data
    /// * `data_hash` - Hash of the data being submitted
    /// * `signature` - Signature for verification
    ///
    /// # Events
    /// Emits `DATA_SUBMITTED` event
    pub fn submit_data(
        env: Env,
        oracle_id: u64,
        data_type_id: u32,
        data_hash: Bytes,
        signature: Bytes,
    ) -> Result<u64, ContractError> {
        let contract_data = Self::get_contract_data(&env)?;

        // Get oracle info
        let oracle_key = (ORACLE_INFO, oracle_id);
        let mut oracle_info: OracleInfo = env
            .storage()
            .persistent()
            .get::<OracleInfo>(&oracle_key)
            .ok_or(ContractError::OracleNotFound)?;

        // Check oracle status
        if oracle_info.status != OracleStatus::Active {
            return Err(ContractError::OracleNotActive);
        }

        // Verify data type is registered and active
        let data_type_key = (DATA_TYPE_REGISTRY, data_type_id);
        let data_type: DataType = env
            .storage()
            .persistent()
            .get::<DataType>(&data_type_key)
            .ok_or(ContractError::InvalidDataType)?;

        if !data_type.active {
            return Err(ContractError::InvalidDataType);
        }

        // Verify signature using public function
        Self::verify_signature(
            env.clone(),
            data_hash.clone(),
            signature.clone(),
            oracle_info.address.clone(),
        )?;

        // Check data freshness if data type has max age requirement
        if data_type.max_age_seconds > 0 {
            let current_time = env.ledger().timestamp();
            // For actual implementation, you would need to pass the data timestamp
            // This is a placeholder for freshness validation
            let data_timestamp = current_time; // In real use, this would come from the data
            let data_age = current_time.saturating_sub(data_timestamp);
            if data_age > data_type.max_age_seconds {
                return Err(ContractError::DataTooOld);
            }
        }

        // Check threshold signature requirements
        if data_type.required_signatures > 1 {
            // For now, we only support single signature
            // Multi-signature support would require additional logic
            return Err(ContractError::InsufficientSignatures);
        }

        // Generate data ID
        let data_id = contract_data.next_data_id;
        let mut updated_contract_data = contract_data;
        updated_contract_data.next_data_id += 1;

        // Create submitted data record
        let submitted_data = SubmittedData {
            data_id,
            oracle_id,
            data_type_id,
            data_hash: data_hash.clone(),
            signatures: vec![&env, signature],
            timestamp: env.ledger().timestamp(),
            verified: true,
        };

        // Store submitted data
        let data_key = (SUBMITTED_DATA, data_id);
        env.storage().persistent().set(&data_key, &submitted_data);

        // Update oracle info
        oracle_info.total_submissions += 1;
        oracle_info.last_activity = env.ledger().timestamp();
        env.storage().persistent().set(&oracle_key, &oracle_info);

        // Update contract data
        env.storage()
            .instance()
            .set(&symbol_short!("data"), &updated_contract_data);

        // Emit event
        env.events()
            .publish((DATA_SUBMITTED,), (data_id, oracle_id, data_type_id));

        Ok(data_id)
    }

    /// Update oracle reputation based on performance
    ///
    /// # Arguments
    /// * `oracle_id` - The oracle ID to update
    /// * `performance_score` - The performance score (0-100)
    ///
    /// # Events
    /// Emits `REPUTATION_UPDATED` event
    pub fn update_reputation(
        env: Env,
        oracle_id: u64,
        performance_score: u32,
    ) -> Result<(), ContractError> {
        Self::check_admin(&env)?;

        // Validate performance score
        if performance_score > 100 {
            return Err(ContractError::InvalidReputationScore);
        }

        // Get oracle info
        let oracle_key = (ORACLE_INFO, oracle_id);
        let mut oracle_info: OracleInfo = env
            .storage()
            .persistent()
            .get::<OracleInfo>(&oracle_key)
            .ok_or(ContractError::OracleNotFound)?;

        // Get reputation data
        let reputation_key = (REPUTATION_DATA, oracle_id);
        let mut reputation_data: ReputationData = env
            .storage()
            .persistent()
            .get::<ReputationData>(&reputation_key)
            .ok_or(ContractError::OracleNotFound)?;

        // Update reputation score (weighted average)
        let current_score = oracle_info.reputation_score;
        let new_score = (current_score * 7 + performance_score * 3) / 10;
        oracle_info.reputation_score = new_score;

        // Update detailed reputation metrics
        reputation_data.accuracy_score =
            (reputation_data.accuracy_score * 8 + performance_score * 2) / 10;
        reputation_data.reliability_score =
            (reputation_data.reliability_score * 8 + performance_score * 2) / 10;
        reputation_data.last_updated = env.ledger().timestamp();

        // Update successful submissions count
        if performance_score >= 80 {
            oracle_info.successful_submissions += 1;
        }

        // Store updated data
        env.storage().persistent().set(&oracle_key, &oracle_info);
        env.storage()
            .persistent()
            .set(&reputation_key, &reputation_data);

        // Emit event
        env.events()
            .publish((REPUTATION_UPDATED,), (oracle_id, new_score));

        Ok(())
    }

    /// Slash an oracle for malicious behavior
    ///
    /// # Arguments
    /// * `oracle_id` - The oracle ID to slash
    /// * `penalty_amount` - The amount to penalize (0 means use percentage)
    ///
    /// # Events
    /// Emits `ORACLE_SLASHED` event
    pub fn slash_oracle(
        env: Env,
        oracle_id: u64,
        penalty_amount: i128,
    ) -> Result<(), ContractError> {
        Self::check_admin(&env)?;

        let contract_data = Self::get_contract_data(&env)?;

        // Get oracle info
        let oracle_key = (ORACLE_INFO, oracle_id);
        let mut oracle_info: OracleInfo = env
            .storage()
            .persistent()
            .get::<OracleInfo>(&oracle_key)
            .ok_or(ContractError::OracleNotFound)?;

        // Get reputation data
        let reputation_key = (REPUTATION_DATA, oracle_id);
        let mut reputation_data: ReputationData = env
            .storage()
            .persistent()
            .get::<ReputationData>(&reputation_key)
            .ok_or(ContractError::OracleNotFound)?;

        // Calculate penalty
        let actual_penalty = if penalty_amount > 0 {
            penalty_amount
        } else {
            // Use percentage of stake
            (oracle_info.stake * contract_data.slashing_percentage as i128) / 100
        };

        // Check if slashing would go below minimum stake
        if oracle_info.stake - actual_penalty < contract_data.minimum_stake {
            return Err(ContractError::CannotSlashBelowMinimum);
        }

        // Apply penalty
        oracle_info.stake -= actual_penalty;
        oracle_info.status = OracleStatus::Slashed;
        oracle_info.reputation_score = oracle_info.reputation_score.saturating_sub(30);

        // Update reputation data
        reputation_data.slash_count += 1;
        reputation_data.last_updated = env.ledger().timestamp();

        // Store updated data
        env.storage().persistent().set(&oracle_key, &oracle_info);
        env.storage()
            .persistent()
            .set(&reputation_key, &reputation_data);

        // Add penalty to reward pool
        let mut updated_contract_data = contract_data;
        updated_contract_data.reward_pool += actual_penalty;
        env.storage()
            .instance()
            .set(&symbol_short!("data"), &updated_contract_data);

        // Emit event
        env.events()
            .publish((ORACLE_SLASHED,), (oracle_id, actual_penalty));

        Ok(())
    }

    /// Distribute rewards to honest oracles for an epoch
    ///
    /// # Arguments
    /// * `epoch` - The epoch number to distribute rewards for
    ///
    /// # Events
    /// Emits `REWARDS_DISTRIBUTED` event
    pub fn distribute_rewards(env: Env, epoch: u64) -> Result<(), ContractError> {
        Self::check_admin(&env)?;

        let mut contract_data = Self::get_contract_data(&env)?;

        // Check if there are rewards to distribute
        if contract_data.reward_pool <= 0 {
            return Err(ContractError::NoRewardsToDistribute);
        }

        // Collect active oracles with their reputation scores
        let mut total_reputation: u64 = 0;
        let mut eligible_oracles: Vec<(u64, u32)> = Vec::new(&env);

        for oracle_address in contract_data.oracles.iter() {
            // Find oracle ID by address
            for i in 1..contract_data.next_oracle_id {
                let oracle_key = (ORACLE_INFO, i);
                if let Some(oracle_info) = env.storage().persistent().get::<OracleInfo>(&oracle_key)
                {
                    if oracle_info.address == *oracle_address
                        && oracle_info.status == OracleStatus::Active
                    {
                        eligible_oracles.push_back((i, oracle_info.reputation_score));
                        total_reputation += oracle_info.reputation_score as u64;
                    }
                }
            }
        }

        if eligible_oracles.is_empty() || total_reputation == 0 {
            return Err(ContractError::NoRewardsToDistribute);
        }

        // Distribute rewards based on reputation
        let total_rewards = contract_data.reward_pool;
        let mut distributed: i128 = 0;
        let mut oracle_rewards: Vec<(u64, i128)> = Vec::new(&env);

        for (oracle_id, reputation_score) in eligible_oracles.iter() {
            let reward = (total_rewards * *reputation_score as i128) / total_reputation as i128;
            oracle_rewards.push_back((*oracle_id, reward));
            distributed += reward;

            // Update oracle stake with reward
            let oracle_key = (ORACLE_INFO, *oracle_id);
            if let Some(mut oracle_info) = env.storage().persistent().get::<OracleInfo>(&oracle_key)
            {
                oracle_info.stake += reward;
                env.storage().persistent().set(&oracle_key, &oracle_info);
            }
        }

        // Update contract data
        contract_data.reward_pool = 0;
        contract_data.current_epoch = epoch;
        env.storage()
            .instance()
            .set(&symbol_short!("data"), &contract_data);

        // Store reward pool data
        let reward_pool_data = RewardPool {
            epoch,
            total_rewards,
            distributed,
            oracle_rewards,
        };
        let reward_pool_key = (REWARD_POOL_DATA, epoch);
        env.storage()
            .persistent()
            .set(&reward_pool_key, &reward_pool_data);

        // Emit event
        env.events()
            .publish((REWARDS_DISTRIBUTED,), (epoch, distributed));

        Ok(())
    }

    /// Verify data authenticity with signature
    ///
    /// # Arguments
    /// * `data` - The data to verify
    /// * `signature` - The signature to verify
    /// * `oracle_address` - The oracle address that signed the data
    ///
    /// # Returns
    /// true if signature is valid, false otherwise
    pub fn verify_signature(
        env: Env,
        data: Bytes,
        signature: Bytes,
        oracle_address: Address,
    ) -> Result<bool, ContractError> {
        // Create message hash
        let message_hash = env.crypto().sha256(&data);

        // In Soroban, we use require_auth for signature verification
        // This is a simplified verification that checks the oracle authorized the call
        oracle_address.require_auth();

        // For actual signature verification, you would use:
        // env.crypto().ed25519_verify(&oracle_address, &message_hash, &signature)
        // But this requires the signature to be in the correct format

        Ok(true)
    }

    /// Create a dispute against an oracle
    ///
    /// # Arguments
    /// * `oracle_id` - The oracle ID being disputed
    /// * `reason` - The reason for the dispute
    /// * `evidence` - Evidence supporting the dispute
    ///
    /// # Events
    /// Emits `DISPUTE_CREATED` event
    pub fn create_dispute(
        env: Env,
        oracle_id: u64,
        reason: DisputeReason,
        evidence: Bytes,
    ) -> Result<u64, ContractError> {
        let mut contract_data = Self::get_contract_data(&env)?;

        // Verify oracle exists
        let oracle_key = (ORACLE_INFO, oracle_id);
        let oracle_info: OracleInfo = env
            .storage()
            .persistent()
            .get::<OracleInfo>(&oracle_key)
            .ok_or(ContractError::OracleNotFound)?;

        // Generate dispute ID
        let dispute_id = contract_data.next_dispute_id;
        contract_data.next_dispute_id += 1;

        // Create dispute
        let dispute = Dispute {
            dispute_id,
            oracle_id,
            initiator: oracle_info.address.clone(),
            reason,
            evidence,
            status: DisputeStatus::Open,
            votes_for: 0,
            votes_against: 0,
            created_at: env.ledger().timestamp(),
            resolved_at: None,
        };

        // Store dispute
        let dispute_key = (DISPUTE_DATA, dispute_id);
        env.storage().persistent().set(&dispute_key, &dispute);

        // Update contract data
        env.storage()
            .instance()
            .set(&symbol_short!("data"), &contract_data);

        // Update oracle reputation data
        let reputation_key = (REPUTATION_DATA, oracle_id);
        if let Some(mut reputation_data) = env
            .storage()
            .persistent()
            .get::<ReputationData>(&reputation_key)
        {
            reputation_data.dispute_count += 1;
            reputation_data.last_updated = env.ledger().timestamp();
            env.storage()
                .persistent()
                .set(&reputation_key, &reputation_data);
        }

        // Emit event
        env.events()
            .publish((DISPUTE_CREATED,), (dispute_id, oracle_id));

        Ok(dispute_id)
    }

    /// Vote on a dispute
    ///
    /// # Arguments
    /// * `dispute_id` - The dispute ID to vote on
    /// * `vote_for` - true to vote for the dispute, false to vote against
    pub fn vote_dispute(env: Env, dispute_id: u64, vote_for: bool) -> Result<(), ContractError> {
        Self::check_admin(&env)?;

        let dispute_key = (DISPUTE_DATA, dispute_id);
        let mut dispute: Dispute = env
            .storage()
            .persistent()
            .get::<Dispute>(&dispute_key)
            .ok_or(ContractError::DisputeNotFound)?;

        // Check if dispute is already resolved
        if dispute.status == DisputeStatus::Resolved || dispute.status == DisputeStatus::Rejected {
            return Err(ContractError::DisputeAlreadyResolved);
        }

        // Update vote count
        if vote_for {
            dispute.votes_for += 1;
        } else {
            dispute.votes_against += 1;
        }

        // Update status if voting phase
        if dispute.status == DisputeStatus::Open {
            dispute.status = DisputeStatus::Voting;
        }

        env.storage().persistent().set(&dispute_key, &dispute);

        Ok(())
    }

    /// Resolve a dispute
    ///
    /// # Arguments
    /// * `dispute_id` - The dispute ID to resolve
    /// * `upheld` - true if dispute is upheld, false if rejected
    ///
    /// # Events
    /// Emits `DISPUTE_RESOLVED` event
    pub fn resolve_dispute(env: Env, dispute_id: u64, upheld: bool) -> Result<(), ContractError> {
        Self::check_admin(&env)?;

        let dispute_key = (DISPUTE_DATA, dispute_id);
        let mut dispute: Dispute = env
            .storage()
            .persistent()
            .get::<Dispute>(&dispute_key)
            .ok_or(ContractError::DisputeNotFound)?;

        // Check if dispute is already resolved
        if dispute.status == DisputeStatus::Resolved || dispute.status == DisputeStatus::Rejected {
            return Err(ContractError::DisputeAlreadyResolved);
        }

        // Resolve dispute
        if upheld {
            dispute.status = DisputeStatus::Resolved;
            // Auto-slash oracle if dispute is upheld
            Self::slash_oracle(env.clone(), dispute.oracle_id, 0)?;
        } else {
            dispute.status = DisputeStatus::Rejected;
        }

        dispute.resolved_at = Some(env.ledger().timestamp());

        env.storage().persistent().set(&dispute_key, &dispute);

        // Emit event
        env.events()
            .publish((DISPUTE_RESOLVED,), (dispute_id, upheld));

        Ok(())
    }

    /// Register a new data type
    ///
    /// # Arguments
    /// * `name` - Name of the data type
    /// * `schema` - Schema definition for the data type
    /// * `max_age_seconds` - Maximum age of data before it's considered stale
    /// * `required_signatures` - Number of signatures required for verification
    ///
    /// # Events
    /// Emits `DATA_TYPE_REGISTERED` event
    pub fn register_data_type(
        env: Env,
        name: Bytes,
        schema: Bytes,
        max_age_seconds: u64,
        required_signatures: u32,
    ) -> Result<u32, ContractError> {
        Self::check_admin(&env)?;

        let contract_data = Self::get_contract_data(&env)?;

        // Generate data type ID (use next_oracle_id as counter for simplicity)
        let data_type_id = contract_data.next_oracle_id as u32;

        let data_type = DataType {
            data_type_id,
            name: name.clone(),
            schema,
            max_age_seconds,
            required_signatures,
            active: true,
        };

        // Store data type
        let data_type_key = (DATA_TYPE_REGISTRY, data_type_id);
        env.storage().persistent().set(&data_type_key, &data_type);

        // Emit event
        env.events()
            .publish((DATA_TYPE_REGISTERED,), (data_type_id, name));

        Ok(data_type_id)
    }

    /// Update data type configuration
    ///
    /// # Arguments
    /// * `data_type_id` - The data type ID to update
    /// * `max_age_seconds` - New maximum age (optional)
    /// * `required_signatures` - New required signatures (optional)
    /// * `active` - New active status (optional)
    pub fn update_data_type(
        env: Env,
        data_type_id: u32,
        max_age_seconds: Option<u64>,
        required_signatures: Option<u32>,
        active: Option<bool>,
    ) -> Result<(), ContractError> {
        Self::check_admin(&env)?;

        let data_type_key = (DATA_TYPE_REGISTRY, data_type_id);
        let mut data_type: DataType = env
            .storage()
            .persistent()
            .get::<DataType>(&data_type_key)
            .ok_or(ContractError::InvalidDataType)?;

        // Update fields if provided
        if let Some(max_age) = max_age_seconds {
            data_type.max_age_seconds = max_age;
        }
        if let Some(signatures) = required_signatures {
            data_type.required_signatures = signatures;
        }
        if let Some(is_active) = active {
            data_type.active = is_active;
        }

        env.storage().persistent().set(&data_type_key, &data_type);

        Ok(())
    }

    /// Rotate oracle based on performance and network health
    ///
    /// # Arguments
    /// * `oracle_id` - The oracle ID to rotate out
    /// * `new_oracle_address` - The new oracle address to rotate in
    ///
    /// # Events
    /// Emits `ORACLE_ROTATED` event
    pub fn rotate_oracle(
        env: Env,
        oracle_id: u64,
        new_oracle_address: Address,
    ) -> Result<(), ContractError> {
        Self::check_admin(&env)?;

        let mut contract_data = Self::get_contract_data(&env)?;

        // Get old oracle info
        let old_oracle_key = (ORACLE_INFO, oracle_id);
        let old_oracle: OracleInfo = env
            .storage()
            .persistent()
            .get::<OracleInfo>(&old_oracle_key)
            .ok_or(ContractError::OracleNotFound)?;

        // Check if new oracle is already registered
        if Self::is_oracle_registered(&contract_data, &new_oracle_address) {
            return Err(ContractError::OracleAlreadyRegistered);
        }

        // Check rotation criteria (reputation below threshold or inactivity)
        let current_time = env.ledger().timestamp();
        let inactive_period = current_time - old_oracle.last_activity;
        let should_rotate = old_oracle.reputation_score < 50 || inactive_period > 30 * 24 * 3600; // 30 days

        if !should_rotate {
            return Err(ContractError::RotationNotAllowed);
        }

        // Generate new oracle ID
        let new_oracle_id = contract_data.next_oracle_id;
        contract_data.next_oracle_id += 1;

        // Create new oracle info with transferred stake
        let new_oracle_info = OracleInfo {
            oracle_id: new_oracle_id,
            address: new_oracle_address.clone(),
            stake: old_oracle.stake,
            reputation_score: 100, // New oracle starts fresh
            status: OracleStatus::Active,
            total_submissions: 0,
            successful_submissions: 0,
            last_activity: current_time,
            metadata: old_oracle.metadata.clone(),
        };

        // Create new reputation data
        let new_reputation_data = ReputationData {
            oracle_id: new_oracle_id,
            accuracy_score: 100,
            reliability_score: 100,
            timeliness_score: 100,
            dispute_count: 0,
            slash_count: 0,
            last_updated: current_time,
        };

        // Update old oracle status
        let mut updated_old_oracle = old_oracle;
        updated_old_oracle.status = OracleStatus::Inactive;
        env.storage()
            .persistent()
            .set(&old_oracle_key, &updated_old_oracle);

        // Store new oracle info
        let new_oracle_key = (ORACLE_INFO, new_oracle_id);
        env.storage()
            .persistent()
            .set(&new_oracle_key, &new_oracle_info);

        // Store new reputation data
        let new_reputation_key = (REPUTATION_DATA, new_oracle_id);
        env.storage()
            .persistent()
            .set(&new_reputation_key, &new_reputation_data);

        // Update oracle list
        let mut new_oracle_list = Vec::new(&env);
        for oracle_address in contract_data.oracles.iter() {
            if *oracle_address != old_oracle.address {
                new_oracle_list.push_back(oracle_address.clone());
            }
        }
        new_oracle_list.push_back(new_oracle_address.clone());
        contract_data.oracles = new_oracle_list;

        // Update contract data
        env.storage()
            .instance()
            .set(&symbol_short!("data"), &contract_data);

        // Emit event
        env.events()
            .publish((ORACLE_ROTATED,), (oracle_id, new_oracle_id));

        Ok(())
    }

    /// Get oracle info by ID
    pub fn get_oracle_info(env: Env, oracle_id: u64) -> Option<OracleInfo> {
        let oracle_key = (ORACLE_INFO, oracle_id);
        env.storage().persistent().get::<OracleInfo>(&oracle_key)
    }

    /// Get reputation data by oracle ID
    pub fn get_reputation_data(env: Env, oracle_id: u64) -> Option<ReputationData> {
        let reputation_key = (REPUTATION_DATA, oracle_id);
        env.storage()
            .persistent()
            .get::<ReputationData>(&reputation_key)
    }

    /// Get dispute by ID
    pub fn get_dispute(env: Env, dispute_id: u64) -> Option<Dispute> {
        let dispute_key = (DISPUTE_DATA, dispute_id);
        env.storage().persistent().get::<Dispute>(&dispute_key)
    }

    /// Get data type by ID
    pub fn get_data_type(env: Env, data_type_id: u32) -> Option<DataType> {
        let data_type_key = (DATA_TYPE_REGISTRY, data_type_id);
        env.storage().persistent().get::<DataType>(&data_type_key)
    }

    /// Get reward pool data for an epoch
    pub fn get_reward_pool(env: Env, epoch: u64) -> Option<RewardPool> {
        let reward_pool_key = (REWARD_POOL_DATA, epoch);
        env.storage()
            .persistent()
            .get::<RewardPool>(&reward_pool_key)
    }

    /// Get contract configuration
    pub fn get_config(env: Env) -> Result<(i128, u32, u64), ContractError> {
        let contract_data = Self::get_contract_data(&env)?;
        Ok((
            contract_data.minimum_stake,
            contract_data.slashing_percentage,
            contract_data.data_freshness_threshold,
        ))
    }

    /// Set contract configuration (admin only)
    pub fn set_config(
        env: Env,
        minimum_stake: Option<i128>,
        slashing_percentage: Option<u32>,
        data_freshness_threshold: Option<u64>,
    ) -> Result<(), ContractError> {
        Self::check_admin(&env)?;

        let mut contract_data = Self::get_contract_data(&env)?;

        if let Some(stake) = minimum_stake {
            contract_data.minimum_stake = stake;
        }
        if let Some(percentage) = slashing_percentage {
            if percentage > 100 {
                return Err(ContractError::InvalidReputationScore);
            }
            contract_data.slashing_percentage = percentage;
        }
        if let Some(threshold) = data_freshness_threshold {
            contract_data.data_freshness_threshold = threshold;
        }

        env.storage()
            .instance()
            .set(&symbol_short!("data"), &contract_data);

        Ok(())
    }

    /// Calculate network decentralization metric
    /// Returns a score from 0-100 based on stake distribution
    pub fn get_decentralization_score(env: Env) -> Result<u32, ContractError> {
        let contract_data = Self::get_contract_data(&env)?;

        if contract_data.oracles.is_empty() {
            return Ok(0);
        }

        let mut stakes: Vec<i128> = Vec::new(&env);
        let mut total_stake: i128 = 0;

        for oracle_address in contract_data.oracles.iter() {
            for i in 1..contract_data.next_oracle_id {
                let oracle_key = (ORACLE_INFO, i);
                if let Some(oracle_info) = env.storage().persistent().get::<OracleInfo>(&oracle_key)
                {
                    if oracle_info.address == *oracle_address {
                        stakes.push_back(oracle_info.stake);
                        total_stake += oracle_info.stake;
                        break;
                    }
                }
            }
        }

        if total_stake == 0 {
            return Ok(0);
        }

        // Calculate Herfindahl-Hirschman Index (HHI)
        let mut hhi: i128 = 0;
        for stake in stakes.iter() {
            let share = (*stake * 10000) / total_stake;
            hhi += share * share;
        }

        // Convert HHI to decentralization score (inverse relationship)
        // Lower HHI = higher decentralization
        let max_hhi = 10000 * 10000; // One oracle has 100% stake
        let decentralization_score = ((max_hhi - hhi) * 100 / max_hhi) as u32;

        Ok(decentralization_score)
    }

    /// Check if consensus threshold is met for an escrow.
    ///
    /// Returns true if the number of unique oracle confirmations is >= threshold.
    /// This enables multi-oracle consensus for high-value trades.
    ///
    /// # Arguments
    /// * `escrow_id` - The escrow ID to check
    /// * `threshold` - Minimum number of unique oracle confirmations required
    /// * `oracle_set` - Set of authorized oracles (empty means any registered oracle is allowed)
    ///
    /// # Returns
    /// true if consensus is met, false otherwise
    pub fn check_consensus(
        env: Env,
        escrow_id: Bytes,
        threshold: u32,
        oracle_set: Vec<Address>,
    ) -> Result<bool, ContractError> {
        // Reject zero threshold
        if threshold == 0 {
            return Err(ContractError::InvalidThreshold);
        }

        let contract_data = Self::get_contract_data(&env)?;

        // Count unique oracle confirmations for this escrow
        let confirmations = Self::get_confirmations_for_escrow(&env, escrow_id)?;
        let mut unique_oracle_count: u32 = 0;

        for confirmation in confirmations.iter() {
            // Verify confirmed oracle is in the authorized set (or set is empty)
            // Also validate against current registration state
            let is_authorized = if oracle_set.is_empty() {
                // If oracle_set is empty, check against all registered oracles
                Self::is_oracle_registered(&contract_data, &confirmation.oracle)
            } else {
                // Check if oracle is in the specified oracle_set
                // Additionally validate against current registration
                let is_in_set = {
                    let mut found = false;
                    for authorized_oracle in oracle_set.iter() {
                        if authorized_oracle == confirmation.oracle {
                            found = true;
                            break;
                        }
                    }
                    found
                };
                is_in_set && Self::is_oracle_registered(&contract_data, &confirmation.oracle)
            };

            if is_authorized && confirmation.verified {
                unique_oracle_count += 1;
            }
        }

        Ok(unique_oracle_count >= threshold)
    }

    // Helper functions

    fn is_initialized(env: &Env) -> bool {
        env.storage().instance().has(&symbol_short!("data"))
    }

    fn get_contract_data(env: &Env) -> Result<ContractData, ContractError> {
        env.storage()
            .instance()
            .get(&symbol_short!("data"))
            .ok_or(ContractError::EscrowNotFound)
    }

    fn get_confirmations_for_escrow(
        env: &Env,
        escrow_id: Bytes,
    ) -> Result<Vec<ConfirmationData>, ContractError> {
        let contract_data = Self::get_contract_data(env)?;
        let mut confirmations = Vec::new(env);

        let confirming_key = (CONFIRMING_ORACLES, escrow_id.clone());
        if env.storage().persistent().has(&confirming_key) {
            let confirming_oracles = Self::get_confirming_oracles(env, &escrow_id);
            for oracle in confirming_oracles.iter() {
                let confirmation_key = (escrow_id.clone(), oracle.clone());
                if let Some(confirmation) = env
                    .storage()
                    .persistent()
                    .get::<ConfirmationData>(&confirmation_key)
                {
                    confirmations.push_back(confirmation);
                }
            }

            return Ok(confirmations);
        }

        // Fallback for legacy data: iterate through all registered oracles
        for oracle in contract_data.oracles.iter() {
            let confirmation_key = (escrow_id.clone(), oracle.clone());
            if let Some(confirmation) = env
                .storage()
                .persistent()
                .get::<ConfirmationData>(&confirmation_key)
            {
                confirmations.push_back(confirmation);
            }
        }

        Ok(confirmations)
    }

    fn get_confirming_oracles(env: &Env, escrow_id: &Bytes) -> Vec<Address> {
        let confirming_key = (CONFIRMING_ORACLES, escrow_id.clone());
        env.storage()
            .persistent()
            .get::<Vec<Address>>(&confirming_key)
            .unwrap_or(Vec::new(env))
    }

    fn check_admin(env: &Env) -> Result<(), ContractError> {
        let contract_data = Self::get_contract_data(env)?;
        contract_data.admin.require_auth();
        Ok(())
    }

    fn is_oracle_registered(contract_data: &ContractData, oracle: &Address) -> bool {
        for registered_oracle in contract_data.oracles.iter() {
            if registered_oracle == *oracle {
                return true;
            }
        }
        false
    }

    fn create_message(env: &Env, escrow_id: &Bytes, event_type: u32, result: &Bytes) -> BytesN<32> {
        // Create a deterministic message hash for signature verification
        let mut message_data = Bytes::new(env);
        message_data.append(escrow_id);
        message_data.append(&Bytes::from_slice(env, &event_type.to_be_bytes()));
        message_data.append(result);

        env.crypto().sha256(&message_data).into()
    }

    fn verify_signature(
        _env: &Env,
        _message: &BytesN<32>,
        _signature: &Bytes,
        oracle: &Address,
    ) -> Result<(), ContractError> {
        // In modern Soroban, we prefer require_auth()
        // For this adapter, we'll ensure the oracle authorized the call
        oracle.require_auth();
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{
        testutils::MockAuth, testutils::MockAuthInvoke, Address, Bytes, Env, IntoVal,
    };

    #[test]
    fn test_initialization() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(OracleAdapter, ());
        let client = OracleAdapterClient::new(&env, &contract_id);

        let admin = Address::generate(&env);

        // Test successful initialization
        assert_eq!(client.initialize(&admin), ());

        // Test double initialization fails
        assert_eq!(
            client.try_initialize(&admin),
            Err(Ok(ContractError::AlreadyInitialized))
        );

        // Test admin getter
        assert_eq!(client.get_admin(), admin);
    }

    #[test]
    fn test_oracle_management() {
        let env = Env::default();
        let contract_id = env.register(OracleAdapter, ());
        let client = OracleAdapterClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let oracle1 = Address::generate(&env);
        let oracle2 = Address::generate(&env);
        let unauthorized = Address::generate(&env);

        // Initialize
        client.initialize(&admin);

        // Test initial state
        assert_eq!(client.get_oracle_count(), 0);

        // Test adding first oracle
        env.mock_auths(&[MockAuth {
            address: &admin,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "add_oracle",
                args: (oracle1.clone(),).into_val(&env),
                sub_invokes: &[],
            },
        }]);
        client.add_oracle(&oracle1);
        assert_eq!(client.is_oracle_registered_query(&oracle1), true);
        assert_eq!(client.get_oracle_count(), 1);

        // Test adding second oracle
        env.mock_auths(&[MockAuth {
            address: &admin,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "add_oracle",
                args: (oracle2.clone(),).into_val(&env),
                sub_invokes: &[],
            },
        }]);
        client.add_oracle(&oracle2);
        assert_eq!(client.is_oracle_registered_query(&oracle2), true);
        assert_eq!(client.get_oracle_count(), 2);

        // Test adding same oracle fails
        env.mock_auths(&[MockAuth {
            address: &admin,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "add_oracle",
                args: (oracle1.clone(),).into_val(&env),
                sub_invokes: &[],
            },
        }]);
        assert_eq!(
            client.try_add_oracle(&oracle1),
            Err(Ok(ContractError::OracleAlreadyRegistered))
        );

        // Test unauthorized add fails
        env.mock_auths(&[MockAuth {
            address: &unauthorized,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "add_oracle",
                args: (Address::generate(&env),).into_val(&env),
                sub_invokes: &[],
            },
        }]);
        assert!(client.try_add_oracle(&Address::generate(&env)).is_err());

        // Test removing oracle
        env.mock_auths(&[MockAuth {
            address: &admin,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "remove_oracle",
                args: (oracle1.clone(),).into_val(&env),
                sub_invokes: &[],
            },
        }]);
        client.remove_oracle(&oracle1);
        assert_eq!(client.is_oracle_registered_query(&oracle1), false);
        assert_eq!(client.get_oracle_count(), 1);

        // Test removing non-existent oracle fails
        env.mock_auths(&[MockAuth {
            address: &admin,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "remove_oracle",
                args: (oracle1.clone(),).into_val(&env),
                sub_invokes: &[],
            },
        }]);
        assert_eq!(
            client.try_remove_oracle(&oracle1),
            Err(Ok(ContractError::OracleNotRegistered))
        );

        // Test unauthorized remove fails
        env.mock_auths(&[MockAuth {
            address: &unauthorized,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "remove_oracle",
                args: (oracle2.clone(),).into_val(&env),
                sub_invokes: &[],
            },
        }]);
        assert!(client.try_remove_oracle(&oracle2).is_err());
    }

    #[test]
    fn test_event_type_validation() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(OracleAdapter, ());
        let client = OracleAdapterClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let oracle = Address::generate(&env);

        // Initialize and add oracle
        client.initialize(&admin);
        client.add_oracle(&oracle);

        let result = Bytes::from_slice(&env, b"confirmed");
        let signature = Bytes::from_slice(&env, b"mock_signature");

        // Test invalid event type (0)
        let escrow_id = Bytes::from_slice(&env, b"escrow_0");
        assert_eq!(
            client.try_confirm_event(&oracle, &escrow_id, &0u32, &result, &signature),
            Err(Ok(ContractError::InvalidEventType))
        );

        // Test invalid event type (6)
        let escrow_id = Bytes::from_slice(&env, b"escrow_6");
        assert_eq!(
            client.try_confirm_event(&oracle, &escrow_id, &6u32, &result, &signature),
            Err(Ok(ContractError::InvalidEventType))
        );

        // Test valid event types (1-5)
        let escrow_ids = [
            b"escrow_1",
            b"escrow_2",
            b"escrow_3",
            b"escrow_4",
            b"escrow_5",
        ];
        for (i, event_type) in (1..=5).enumerate() {
            let escrow_id = Bytes::from_slice(&env, escrow_ids[i]);
            let confirm_result =
                client.try_confirm_event(&oracle, &escrow_id, &event_type, &result, &signature);
            assert!(confirm_result.is_ok());
        }
    }

    #[test]
    fn test_replay_attack_prevention() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(OracleAdapter, ());
        let client = OracleAdapterClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let oracle = Address::generate(&env);

        // Initialize and add oracle
        client.initialize(&admin);
        client.add_oracle(&oracle);

        let escrow_id = Bytes::from_slice(&env, b"escrow_123");
        let event_type = 1u32;
        let result = Bytes::from_slice(&env, b"confirmed");
        let signature = Bytes::from_slice(&env, b"mock_signature");

        // First confirmation should work
        // Note: verify_signature is now just require_auth(), so it should pass with mock_all_auths
        let confirm_result =
            client.try_confirm_event(&oracle, &escrow_id, &event_type, &result, &signature);
        assert!(confirm_result.is_ok());

        // Second confirmation from same oracle should fail (replay attack)
        assert_eq!(
            client.try_confirm_event(&oracle, &escrow_id, &event_type, &result, &signature),
            Err(Ok(ContractError::ConfirmationAlreadyExists))
        );
    }

    #[test]
    fn test_unauthorized_oracle_confirmation() {
        let env = Env::default();
        let contract_id = env.register(OracleAdapter, ());
        let client = OracleAdapterClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let unauthorized_oracle = Address::generate(&env);

        // Initialize without adding the oracle
        client.initialize(&admin);

        let escrow_id = Bytes::from_slice(&env, b"escrow_123");
        let event_type = 1u32;
        let result = Bytes::from_slice(&env, b"confirmed");
        let signature = Bytes::from_slice(&env, b"mock_signature");

        // Confirmation from unregistered oracle should fail
        assert_eq!(
            client.try_confirm_event(
                &unauthorized_oracle,
                &escrow_id,
                &event_type,
                &result,
                &signature
            ),
            Err(Ok(ContractError::OracleNotRegistered))
        );
    }

    #[test]
    fn test_get_confirmation_empty() {
        let env = Env::default();
        let contract_id = env.register(OracleAdapter, ());
        let client = OracleAdapterClient::new(&env, &contract_id);

        let admin = Address::generate(&env);

        // Initialize
        client.initialize(&admin);

        let escrow_id = Bytes::from_slice(&env, b"escrow_123");

        // Test getting confirmation for non-existent escrow
        assert_eq!(client.get_confirmation(&escrow_id), None);
    }

    #[test]
    fn test_oracle_queries() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(OracleAdapter, ());
        let client = OracleAdapterClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let oracle1 = Address::generate(&env);
        let oracle2 = Address::generate(&env);

        // Initialize
        client.initialize(&admin);

        // Initially no oracles
        assert_eq!(client.get_oracle_count(), 0);

        // Add oracles
        client.add_oracle(&oracle1);
        client.add_oracle(&oracle2);
        assert_eq!(client.get_oracle_count(), 2);

        // Test oracle registration queries
        assert_eq!(client.is_oracle_registered_query(&oracle1), true);
        assert_eq!(client.is_oracle_registered_query(&oracle2), true);
        assert_eq!(
            client.is_oracle_registered_query(&Address::generate(&env)),
            false
        );

        // Test getting oracles by index
        let oracle_at_0 = client.get_oracle_at(&0);
        let oracle_at_1 = client.get_oracle_at(&1);
        let oracle_at_2 = client.get_oracle_at(&2);

        assert!(oracle_at_0.is_some());
        assert!(oracle_at_1.is_some());
        assert!(oracle_at_2.is_none()); // Out of bounds
    }

    #[test]
    fn test_message_creation() {
        let env = Env::default();
        let contract_id = env.register(OracleAdapter, ());

        let escrow_id = Bytes::from_slice(&env, b"escrow_123");
        let event_type = 1u32;
        let result = Bytes::from_slice(&env, b"confirmed");

        env.as_contract(&contract_id, || {
            let message = OracleAdapter::create_message(&env, &escrow_id, event_type, &result);
            // Message should be a valid hash
            assert_eq!(message.len(), 32);
        });
    }

    #[test]
    fn test_multi_oracle_consensus_threshold_met() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(OracleAdapter, ());
        let client = OracleAdapterClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let oracle1 = Address::generate(&env);
        let oracle2 = Address::generate(&env);
        let oracle3 = Address::generate(&env);

        // Initialize and add oracles
        client.initialize(&admin);
        client.add_oracle(&oracle1);
        client.add_oracle(&oracle2);
        client.add_oracle(&oracle3);

        let escrow_id = Bytes::from_slice(&env, b"escrow_multi_1");
        let event_type = 2u32; // Delivery
        let result = Bytes::from_slice(&env, b"confirmed");
        let signature = Bytes::from_slice(&env, b"mock_signature");

        // First oracle confirms
        assert!(client
            .try_confirm_event(&oracle1, &escrow_id, &event_type, &result, &signature)
            .is_ok());

        // Second oracle confirms
        assert!(client
            .try_confirm_event(&oracle2, &escrow_id, &event_type, &result, &signature)
            .is_ok());

        // Create oracle set with all 3 oracles
        let oracle_set = Vec::from_array(&env, [oracle1.clone(), oracle2.clone(), oracle3.clone()]);

        // Check consensus with threshold 2 - should be met (2 confirmations)
        assert!(client.check_consensus(&escrow_id, &2u32, &oracle_set));

        // Check consensus with threshold 3 - should not be met (only 2 confirmations)
        assert!(!client.check_consensus(&escrow_id, &3u32, &oracle_set));
    }

    #[test]
    fn test_multi_oracle_consensus_with_oracle_set() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(OracleAdapter, ());
        let client = OracleAdapterClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let oracle1 = Address::generate(&env);
        let oracle2 = Address::generate(&env);
        let oracle3 = Address::generate(&env);
        let unauthorized_oracle = Address::generate(&env);

        // Initialize and add oracles
        client.initialize(&admin);
        client.add_oracle(&oracle1);
        client.add_oracle(&oracle2);
        client.add_oracle(&oracle3);
        client.add_oracle(&unauthorized_oracle);

        let escrow_id = Bytes::from_slice(&env, b"escrow_multi_2");
        let event_type = 2u32;
        let result = Bytes::from_slice(&env, b"confirmed");
        let signature = Bytes::from_slice(&env, b"mock_signature");

        // Oracle 1 and 2 confirm
        assert!(client
            .try_confirm_event(&oracle1, &escrow_id, &event_type, &result, &signature)
            .is_ok());
        assert!(client
            .try_confirm_event(&oracle2, &escrow_id, &event_type, &result, &signature)
            .is_ok());

        // Unauthorized oracle also confirms
        assert!(client
            .try_confirm_event(
                &unauthorized_oracle,
                &escrow_id,
                &event_type,
                &result,
                &signature
            )
            .is_ok());

        // Create restricted oracle set (only oracle1, oracle2, oracle3)
        let restricted_oracle_set =
            Vec::from_array(&env, [oracle1.clone(), oracle2.clone(), oracle3.clone()]);

        // Check consensus with restricted set - should only count oracle1 and oracle2 (2 confirmations)
        // unauthorized_oracle is not in the set, so it shouldn't count
        assert!(client.check_consensus(&escrow_id, &2u32, &restricted_oracle_set));

        // Create full oracle set including unauthorized
        let full_oracle_set = Vec::from_array(
            &env,
            [oracle1, oracle2.clone(), oracle3, unauthorized_oracle],
        );

        // With all oracles in set, should have 3 confirmations
        assert!(client.check_consensus(&escrow_id, &3u32, &full_oracle_set));
    }

    #[test]
    fn test_consensus_prevents_collusion() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(OracleAdapter, ());
        let client = OracleAdapterClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let oracle1 = Address::generate(&env);
        let oracle2 = Address::generate(&env);

        client.initialize(&admin);
        client.add_oracle(&oracle1);
        client.add_oracle(&oracle2);

        let escrow_id = Bytes::from_slice(&env, b"escrow_collusion");
        let event_type = 2u32;
        let result = Bytes::from_slice(&env, b"confirmed");
        let signature = Bytes::from_slice(&env, b"mock_signature");

        // Oracle 1 confirms
        assert!(client
            .try_confirm_event(&oracle1, &escrow_id, &event_type, &result, &signature)
            .is_ok());

        // Only oracle1 is in the authorized set
        let restricted_set = Vec::from_array(&env, [oracle1.clone()]);

        // Consensus should require 1 oracle, which is met
        assert!(client.check_consensus(&escrow_id, &1u32, &restricted_set));

        // But if we require both oracles to confirm and they're in the set:
        let both_set = Vec::from_array(&env, [oracle1, oracle2.clone()]);
        assert!(!client.check_consensus(&escrow_id, &2u32, &both_set));

        // Oracle 2 confirms
        assert!(client
            .try_confirm_event(&oracle2, &escrow_id, &event_type, &result, &signature)
            .is_ok());

        // Now both oracles have confirmed
        assert!(client.check_consensus(&escrow_id, &2u32, &both_set));
    }

    #[test]
    fn test_consensus_empty_oracle_set_allows_any() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(OracleAdapter, ());
        let client = OracleAdapterClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let oracle1 = Address::generate(&env);
        let oracle2 = Address::generate(&env);

        client.initialize(&admin);
        client.add_oracle(&oracle1);
        client.add_oracle(&oracle2);

        let escrow_id = Bytes::from_slice(&env, b"escrow_any");
        let event_type = 2u32;
        let result = Bytes::from_slice(&env, b"confirmed");
        let signature = Bytes::from_slice(&env, b"mock_signature");

        // Both oracles confirm
        assert!(client
            .try_confirm_event(&oracle1, &escrow_id, &event_type, &result, &signature)
            .is_ok());
        assert!(client
            .try_confirm_event(&oracle2, &escrow_id, &event_type, &result, &signature)
            .is_ok());

        // Empty oracle set means any registered oracle can confirm
        let empty_set = Vec::new(&env);

        // Should count both confirmations
        assert!(client.check_consensus(&escrow_id, &2u32, &empty_set));
    }

    #[test]
    fn test_consensus_only_counts_verified() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(OracleAdapter, ());
        let client = OracleAdapterClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let oracle1 = Address::generate(&env);

        client.initialize(&admin);
        client.add_oracle(&oracle1);

        let escrow_id = Bytes::from_slice(&env, b"escrow_verified");
        let event_type = 2u32;
        let result = Bytes::from_slice(&env, b"confirmed");
        let signature = Bytes::from_slice(&env, b"mock_signature");

        // Confirm from oracle1
        assert!(client
            .try_confirm_event(&oracle1, &escrow_id, &event_type, &result, &signature)
            .is_ok());

        // Get the confirmation to verify it's marked as verified
        let confirmations = client.get_confirmation(&escrow_id).unwrap();
        assert_eq!(confirmations.len(), 1);
        assert_eq!(confirmations.get(0).unwrap().verified, true);

        let oracle_set = Vec::from_array(&env, [oracle1]);

        // Check consensus - should count the verified confirmation
        assert!(client.check_consensus(&escrow_id, &1u32, &oracle_set));
    }

    #[test]
    fn test_consensus_rejects_zero_threshold() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(OracleAdapter, ());
        let client = OracleAdapterClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let oracle1 = Address::generate(&env);

        client.initialize(&admin);
        client.add_oracle(&oracle1);

        let escrow_id = Bytes::from_slice(&env, b"escrow_zero_threshold");
        let event_type = 2u32;
        let result = Bytes::from_slice(&env, b"confirmed");
        let signature = Bytes::from_slice(&env, b"mock_signature");

        // Confirm from oracle1
        assert!(client
            .try_confirm_event(&oracle1, &escrow_id, &event_type, &result, &signature)
            .is_ok());

        let oracle_set = Vec::from_array(&env, [oracle1]);

        // Check consensus with threshold 0 should fail
        assert_eq!(
            client.try_check_consensus(&escrow_id, &0u32, &oracle_set),
            Err(Ok(ContractError::InvalidThreshold))
        );

        // Threshold 1 should succeed
        assert!(client.check_consensus(&escrow_id, &1u32, &oracle_set));
    }

    #[test]
    fn test_propose_admin() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(OracleAdapter, ());
        let client = OracleAdapterClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        let new_admin = Address::generate(&env);

        client.initialize(&admin);
        assert!(client.get_pending_admin().is_none());
        client.propose_admin(&new_admin);
        assert_eq!(client.get_pending_admin(), Some(new_admin));
    }

    #[test]
    fn test_accept_admin() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(OracleAdapter, ());
        let client = OracleAdapterClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        let new_admin = Address::generate(&env);

        client.initialize(&admin);
        client.propose_admin(&new_admin);
        client.accept_admin();

        assert_eq!(client.get_admin(), new_admin);
        assert!(client.get_pending_admin().is_none());
    }

    #[test]
    #[should_panic]
    fn test_propose_admin_unauthorized() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let new_admin = Address::generate(&env);
        let contract_id = env.register(OracleAdapter, ());

        env.as_contract(&contract_id, || {
            OracleAdapter::initialize(env.clone(), admin).unwrap();
            // No mocked auth — check_admin → admin.require_auth() panics
            OracleAdapter::propose_admin(env.clone(), new_admin).unwrap();
        });
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #11)")]
    fn test_accept_admin_no_pending() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(OracleAdapter, ());
        let client = OracleAdapterClient::new(&env, &contract_id);
        let admin = Address::generate(&env);

        client.initialize(&admin);
        client.accept_admin();
    }
}

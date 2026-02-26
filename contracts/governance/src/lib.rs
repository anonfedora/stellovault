//! Decentralized Governance Contract for StelloVault
//!
//! This contract enables stakeholders to propose and vote on protocol parameter changes.
//! Features include:
//! - Proposal creation with target contract and parameter specifications
//! - Token-weighted voting based on locked balances
//! - Quorum and majority requirements
//! - Timelock grace period between proposal passing and execution
//! - Cross-contract calls to update risk parameters

#![no_std]

use soroban_sdk::{contract, contractclient, contractimpl, contracttype, symbol_short, Address, Env, Symbol};

// ============================================================================
// Error Types
// ============================================================================

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContractError {
    // Initialization errors
    Unauthorized = 1,
    AlreadyInitialized = 2,

    // Proposal errors
    ProposalNotFound = 3,
    ProposalNotActive = 4,
    ProposalAlreadyExecuted = 5,
    ProposalFailed = 6,

    // Voting errors
    VotingEnded = 7,
    AlreadyVoted = 8,
    InsufficientVotingPower = 9,

    // Execution errors
    TimelockNotExpired = 10,
    QuorumNotReached = 11,
    MajorityNotReached = 12,

    // Parameter errors
    InvalidParameter = 13,
    InvalidValue = 14,

    // Math errors
    MathOverflow = 15,
}

impl From<ContractError> for soroban_sdk::Error {
    fn from(err: ContractError) -> Self {
        soroban_sdk::Error::from_contract_error(err as u32)
    }
}

impl From<&ContractError> for soroban_sdk::Error {
    fn from(err: &ContractError) -> Self {
        soroban_sdk::Error::from_contract_error(*err as u32)
    }
}

impl TryFrom<soroban_sdk::Error> for ContractError {
    type Error = soroban_sdk::Error;

    fn try_from(err: soroban_sdk::Error) -> Result<Self, Self::Error> {
        Err(err)
    }
}

// ============================================================================
// Data Structures
// ============================================================================

/// Governance proposal
#[contracttype]
#[derive(Clone, Debug)]
pub struct Proposal {
    pub id: u64,
    pub proposer: Address,
    pub target_contract: Address,
    pub parameter_symbol: Symbol,
    pub new_value: i128,
    pub voting_end_ts: u64,
    pub votes_for: i128,
    pub votes_against: i128,
    pub executed: bool,
    pub execution_ts: u64,
    pub created_at: u64,
}

/// Vote record
#[contracttype]
#[derive(Clone, Debug)]
pub struct Vote {
    pub voter: Address,
    pub proposal_id: u64,
    pub support: bool,
    pub voting_power: i128,
    pub timestamp: u64,
}

/// Proposal lifecycle status
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProposalStatus {
    Active = 0,    // Voting is ongoing
    Succeeded = 1, // Voting ended, quorum + majority reached
    Failed = 2,    // Voting ended, quorum or majority not reached
    Executed = 3,  // Proposal executed on-chain
    Cancelled = 4, // Cancelled by admin emergency
}

/// Governance configuration
#[contracttype]
#[derive(Clone, Debug)]
pub struct GovernanceConfig {
    pub voting_period: u64,     // Duration of voting in seconds
    pub timelock_period: u64,   // Grace period before execution
    pub quorum_bps: u32,        // Quorum in basis points (e.g., 1000 = 10%)
    pub majority_bps: u32,      // Majority threshold in basis points (e.g., 5000 = 50%)
    pub min_voting_power: i128, // Minimum tokens to create proposal
}

impl Default for GovernanceConfig {
    fn default() -> Self {
        Self {
            voting_period: 604800,  // 7 days
            timelock_period: 86400, // 24 hours
            quorum_bps: 1000,       // 10%
            majority_bps: 5000,     // 50%
            min_voting_power: 1000, // 1000 tokens minimum
        }
    }
}

// ============================================================================
// RiskAssessment Cross-Contract Interface
// ============================================================================

/// Mirror of RiskAssessment's RiskParameters (XDR-compatible)
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

/// Client interface for calling RiskAssessment contract
#[contractclient(name = "RiskAssessmentClient")]
pub trait IRiskAssessment {
    fn get_risk_parameters(env: Env) -> RiskParameters;
    fn update_risk_parameters(env: Env, new_params: RiskParameters);
}

// ============================================================================
// Event Symbols
// ============================================================================

const EVT_INIT: Symbol = symbol_short!("gov_init");
const EVT_PROPOSAL: Symbol = symbol_short!("proposal");
const EVT_VOTE: Symbol = symbol_short!("vote");
const EVT_EXECUTED: Symbol = symbol_short!("executed");
const EVT_CANCELLED: Symbol = symbol_short!("cancelled");

// ============================================================================
// Contract Definition
// ============================================================================

#[contract]
pub struct Governance;

#[contractimpl]
impl Governance {
    // ========================================================================
    // Initialization
    // ========================================================================

    /// Initialize the governance contract
    ///
    /// # Arguments
    /// * `admin` - Admin address for emergency controls
    /// * `token_contract` - Address of the governance token contract
    /// * `risk_assessment` - Address of the RiskAssessment contract
    pub fn initialize(
        env: Env,
        admin: Address,
        token_contract: Address,
        risk_assessment: Address,
    ) -> Result<(), ContractError> {
        if env.storage().instance().has(&symbol_short!("admin")) {
            return Err(ContractError::AlreadyInitialized);
        }

        env.storage()
            .instance()
            .set(&symbol_short!("admin"), &admin);
        env.storage()
            .instance()
            .set(&symbol_short!("token"), &token_contract);
        env.storage()
            .instance()
            .set(&symbol_short!("risk_asm"), &risk_assessment);

        let config = GovernanceConfig::default();
        env.storage()
            .instance()
            .set(&symbol_short!("config"), &config);

        // Initialize proposal counter
        env.storage()
            .instance()
            .set(&symbol_short!("prop_cnt"), &0u64);

        // Initialize total voting power (simplified - in production, query from token contract)
        env.storage()
            .instance()
            .set(&symbol_short!("total_pwr"), &1000000i128);

        env.events()
            .publish((EVT_INIT,), (admin.clone(), token_contract));

        Ok(())
    }

    /// Get admin address
    pub fn admin(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&symbol_short!("admin"))
            .unwrap()
    }

    /// Get governance configuration
    pub fn get_config(env: Env) -> GovernanceConfig {
        env.storage()
            .instance()
            .get(&symbol_short!("config"))
            .unwrap_or_default()
    }

    // ========================================================================
    // Proposal Management
    // ========================================================================

    /// Create a new proposal to change a risk parameter
    ///
    /// # Arguments
    /// * `proposer` - Address creating the proposal
    /// * `target_contract` - Contract address to update (e.g., RiskAssessment)
    /// * `parameter_symbol` - Symbol identifying the parameter to change
    /// * `new_value` - New value for the parameter
    pub fn create_proposal(
        env: Env,
        proposer: Address,
        target_contract: Address,
        parameter_symbol: Symbol,
        new_value: i128,
    ) -> Result<u64, ContractError> {
        proposer.require_auth();

        let config = Self::get_config(env.clone());

        // Check proposer has minimum voting power
        let voting_power = Self::get_voting_power(&env, &proposer);
        if voting_power < config.min_voting_power {
            return Err(ContractError::InsufficientVotingPower);
        }

        // Validate parameter and value
        Self::validate_parameter(&parameter_symbol, new_value)?;

        // Get and increment proposal counter
        let proposal_id: u64 = env
            .storage()
            .instance()
            .get(&symbol_short!("prop_cnt"))
            .unwrap_or(0);

        let next_id = proposal_id
            .checked_add(1)
            .ok_or(ContractError::MathOverflow)?;
        env.storage()
            .instance()
            .set(&symbol_short!("prop_cnt"), &next_id);

        let current_ts = env.ledger().timestamp();
        let voting_end_ts = current_ts
            .checked_add(config.voting_period)
            .ok_or(ContractError::MathOverflow)?;

        let execution_ts = voting_end_ts
            .checked_add(config.timelock_period)
            .ok_or(ContractError::MathOverflow)?;

        let proposal = Proposal {
            id: proposal_id,
            proposer: proposer.clone(),
            target_contract,
            parameter_symbol: parameter_symbol.clone(),
            new_value,
            voting_end_ts,
            votes_for: 0,
            votes_against: 0,
            executed: false,
            execution_ts,
            created_at: current_ts,
        };

        env.storage()
            .persistent()
            .set(&(symbol_short!("prop"), proposal_id), &proposal);

        env.events().publish(
            (EVT_PROPOSAL,),
            (proposal_id, proposer, parameter_symbol, new_value),
        );

        Ok(proposal_id)
    }

    /// Get a proposal by ID
    pub fn get_proposal(env: Env, proposal_id: u64) -> Result<Proposal, ContractError> {
        env.storage()
            .persistent()
            .get(&(symbol_short!("prop"), proposal_id))
            .ok_or(ContractError::ProposalNotFound)
    }

    /// Get all proposal IDs (for querying)
    pub fn get_proposal_count(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&symbol_short!("prop_cnt"))
            .unwrap_or(0)
    }

    // ========================================================================
    // Voting
    // ========================================================================

    /// Cast a vote on a proposal
    ///
    /// # Arguments
    /// * `proposal_id` - ID of the proposal to vote on
    /// * `voter` - Address of the voter
    /// * `support` - true for "for", false for "against"
    pub fn cast_vote(
        env: Env,
        proposal_id: u64,
        voter: Address,
        support: bool,
    ) -> Result<(), ContractError> {
        voter.require_auth();

        // Get proposal
        let mut proposal = Self::get_proposal(env.clone(), proposal_id)?;

        // Check voting is still active
        let current_ts = env.ledger().timestamp();
        if current_ts >= proposal.voting_end_ts {
            return Err(ContractError::VotingEnded);
        }

        // Check if already voted
        let vote_key = (symbol_short!("vote"), proposal_id, voter.clone());
        if env.storage().persistent().has(&vote_key) {
            return Err(ContractError::AlreadyVoted);
        }

        // Get voting power (weighted by locked token balance)
        let voting_power = Self::get_voting_power(&env, &voter);
        if voting_power == 0 {
            return Err(ContractError::InsufficientVotingPower);
        }

        // Record vote
        let vote = Vote {
            voter: voter.clone(),
            proposal_id,
            support,
            voting_power,
            timestamp: current_ts,
        };

        env.storage().persistent().set(&vote_key, &vote);

        // Update proposal vote counts
        if support {
            proposal.votes_for = proposal
                .votes_for
                .checked_add(voting_power)
                .ok_or(ContractError::MathOverflow)?;
        } else {
            proposal.votes_against = proposal
                .votes_against
                .checked_add(voting_power)
                .ok_or(ContractError::MathOverflow)?;
        }

        env.storage()
            .persistent()
            .set(&(symbol_short!("prop"), proposal_id), &proposal);

        env.events()
            .publish((EVT_VOTE,), (proposal_id, voter, support, voting_power));

        Ok(())
    }

    /// Get vote for a specific voter on a proposal
    pub fn get_vote(env: Env, proposal_id: u64, voter: Address) -> Option<Vote> {
        let vote_key = (symbol_short!("vote"), proposal_id, voter);
        env.storage().persistent().get(&vote_key)
    }

    // ========================================================================
    // Proposal Execution
    // ========================================================================

    /// Execute a passed proposal after timelock period
    ///
    /// # Arguments
    /// * `proposal_id` - ID of the proposal to execute
    pub fn execute_proposal(env: Env, proposal_id: u64) -> Result<(), ContractError> {
        let mut proposal = Self::get_proposal(env.clone(), proposal_id)?;

        // Check not already executed
        if proposal.executed {
            return Err(ContractError::ProposalAlreadyExecuted);
        }

        let current_ts = env.ledger().timestamp();

        // Check voting has ended
        if current_ts < proposal.voting_end_ts {
            return Err(ContractError::ProposalNotActive);
        }

        // Check timelock has expired
        if current_ts < proposal.execution_ts {
            return Err(ContractError::TimelockNotExpired);
        }

        let config = Self::get_config(env.clone());

        // Verify quorum
        let total_votes = proposal
            .votes_for
            .checked_add(proposal.votes_against)
            .ok_or(ContractError::MathOverflow)?;

        let total_voting_power: i128 = env
            .storage()
            .instance()
            .get(&symbol_short!("total_pwr"))
            .unwrap_or(1000000);

        let quorum_required = total_voting_power
            .checked_mul(config.quorum_bps as i128)
            .ok_or(ContractError::MathOverflow)?
            / 10000;

        if total_votes < quorum_required {
            return Err(ContractError::QuorumNotReached);
        }

        // Verify majority
        let majority_required = total_votes
            .checked_mul(config.majority_bps as i128)
            .ok_or(ContractError::MathOverflow)?
            / 10000;

        if proposal.votes_for < majority_required {
            return Err(ContractError::MajorityNotReached);
        }

        // Execute cross-contract call to update risk parameters
        Self::execute_parameter_update(
            &env,
            &proposal.target_contract,
            &proposal.parameter_symbol,
            proposal.new_value,
        )?;

        // Mark as executed
        proposal.executed = true;
        env.storage()
            .persistent()
            .set(&(symbol_short!("prop"), proposal_id), &proposal);

        env.events().publish(
            (EVT_EXECUTED,),
            (
                proposal_id,
                proposal.parameter_symbol.clone(),
                proposal.new_value,
            ),
        );

        Ok(())
    }

    /// Check if a proposal has passed (quorum + majority)
    pub fn has_proposal_passed(env: Env, proposal_id: u64) -> Result<bool, ContractError> {
        let proposal = Self::get_proposal(env.clone(), proposal_id)?;

        let config = Self::get_config(env.clone());

        let total_votes = proposal
            .votes_for
            .checked_add(proposal.votes_against)
            .ok_or(ContractError::MathOverflow)?;

        let total_voting_power: i128 = env
            .storage()
            .instance()
            .get(&symbol_short!("total_pwr"))
            .unwrap_or(1000000);

        let quorum_required = total_voting_power
            .checked_mul(config.quorum_bps as i128)
            .ok_or(ContractError::MathOverflow)?
            / 10000;

        if total_votes < quorum_required {
            return Ok(false);
        }

        let majority_required = total_votes
            .checked_mul(config.majority_bps as i128)
            .ok_or(ContractError::MathOverflow)?
            / 10000;

        Ok(proposal.votes_for >= majority_required)
    }

    // ========================================================================
    // Admin Functions
    // ========================================================================

    /// Update governance configuration (admin only)
    pub fn update_config(env: Env, new_config: GovernanceConfig) -> Result<(), ContractError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&symbol_short!("admin"))
            .ok_or(ContractError::Unauthorized)?;

        admin.require_auth();

        env.storage()
            .instance()
            .set(&symbol_short!("config"), &new_config);

        Ok(())
    }

    /// Cancel a proposal (admin only, for emergencies)
    pub fn cancel_proposal(env: Env, proposal_id: u64) -> Result<(), ContractError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&symbol_short!("admin"))
            .ok_or(ContractError::Unauthorized)?;

        admin.require_auth();

        let mut proposal = Self::get_proposal(env.clone(), proposal_id)?;

        if proposal.executed {
            return Err(ContractError::ProposalAlreadyExecuted);
        }

        // Mark as executed to prevent execution
        proposal.executed = true;
        env.storage()
            .persistent()
            .set(&(symbol_short!("prop"), proposal_id), &proposal);

        env.events().publish((EVT_CANCELLED,), (proposal_id,));

        Ok(())
    }

    /// Set total voting power (admin only)
    pub fn set_total_voting_power(env: Env, total_power: i128) -> Result<(), ContractError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&symbol_short!("admin"))
            .ok_or(ContractError::Unauthorized)?;

        admin.require_auth();

        env.storage()
            .instance()
            .set(&symbol_short!("total_pwr"), &total_power);

        Ok(())
    }

    // ========================================================================
    // Internal Helper Functions
    // ========================================================================

    /// Get voting power for an address
    /// In production, this would query the token contract for locked balance
    fn get_voting_power(env: &Env, voter: &Address) -> i128 {
        // For testing, use stored test data
        let key = (symbol_short!("vp"), voter.clone());
        env.storage().persistent().get(&key).unwrap_or(100)
    }

    /// Set voting power for testing
    #[cfg(test)]
    pub fn set_voting_power(env: Env, voter: Address, power: i128) {
        let key = (symbol_short!("vp"), voter);
        env.storage().persistent().set(&key, &power);
    }

    /// Validate parameter symbol and value
    fn validate_parameter(parameter: &Symbol, value: i128) -> Result<(), ContractError> {
        // Validate based on parameter type
        // Use symbol comparison instead of string conversion
        let liq_thr = symbol_short!("liq_thr");
        let liq_pen = symbol_short!("liq_pen");
        let min_hf = symbol_short!("min_hf");
        let max_liq = symbol_short!("max_liq");
        let grace_pd = symbol_short!("grace_pd");
        let liq_bon = symbol_short!("liq_bon");

        if parameter == &liq_thr {
            // Liquidation threshold: 50-95% (5000-9500 bps)
            if !(5000..=9500).contains(&value) {
                return Err(ContractError::InvalidValue);
            }
        } else if parameter == &liq_pen {
            // Liquidation penalty: 1-10% (100-1000 bps)
            if !(100..=1000).contains(&value) {
                return Err(ContractError::InvalidValue);
            }
        } else if parameter == &min_hf {
            // Min health factor: 1.0-1.5 (10000-15000 bps)
            if !(10000..=15000).contains(&value) {
                return Err(ContractError::InvalidValue);
            }
        } else if parameter == &max_liq {
            // Max liquidation ratio: 25-50% (2500-5000 bps)
            if !(2500..=5000).contains(&value) {
                return Err(ContractError::InvalidValue);
            }
        } else if parameter == &grace_pd {
            // Grace period: 5 min - 24 hours (300-86400 seconds)
            if !(300..=86400).contains(&value) {
                return Err(ContractError::InvalidValue);
            }
        } else if parameter == &liq_bon {
            // Liquidator bonus: 1-10% (100-1000 bps)
            if !(100..=1000).contains(&value) {
                return Err(ContractError::InvalidValue);
            }
        } else {
            return Err(ContractError::InvalidParameter);
        }

        Ok(())
    }

    /// Execute parameter update via cross-contract call to RiskAssessment
    fn execute_parameter_update(
        env: &Env,
        target_contract: &Address,
        parameter: &Symbol,
        value: i128,
    ) -> Result<(), ContractError> {
        let client = RiskAssessmentClient::new(env, target_contract);

        // Fetch current parameters from RiskAssessment
        let mut params = client.get_risk_parameters();

        // Apply the single parameter change
        let liq_thr = symbol_short!("liq_thr");
        let liq_pen = symbol_short!("liq_pen");
        let min_hf = symbol_short!("min_hf");
        let max_liq = symbol_short!("max_liq");
        let grace_pd = symbol_short!("grace_pd");
        let liq_bon = symbol_short!("liq_bon");

        if parameter == &liq_thr {
            params.liquidation_threshold = value as u32;
        } else if parameter == &liq_pen {
            params.liquidation_penalty = value as u32;
        } else if parameter == &min_hf {
            params.min_health_factor = value as u32;
        } else if parameter == &max_liq {
            params.max_liquidation_ratio = value as u32;
        } else if parameter == &grace_pd {
            params.grace_period = value as u64;
        } else if parameter == &liq_bon {
            params.liquidator_bonus = value as u32;
        }

        // Cross-contract call to RiskAssessment::update_risk_parameters
        client.update_risk_parameters(&params);

        Ok(())
    }

    /// Get the current lifecycle status of a proposal
    pub fn get_proposal_status(env: Env, proposal_id: u64) -> Result<ProposalStatus, ContractError> {
        let proposal = Self::get_proposal(env.clone(), proposal_id)?;

        if proposal.executed {
            return Ok(ProposalStatus::Executed);
        }

        let current_ts = env.ledger().timestamp();

        // Still in voting window
        if current_ts < proposal.voting_end_ts {
            return Ok(ProposalStatus::Active);
        }

        // Voting ended — check quorum + majority
        let config = Self::get_config(env.clone());
        let total_votes = proposal
            .votes_for
            .checked_add(proposal.votes_against)
            .unwrap_or(0);

        let total_voting_power: i128 = env
            .storage()
            .instance()
            .get(&symbol_short!("total_pwr"))
            .unwrap_or(1_000_000);

        let quorum_required = total_voting_power
            .checked_mul(config.quorum_bps as i128)
            .unwrap_or(0)
            / 10000;

        if total_votes < quorum_required {
            return Ok(ProposalStatus::Failed);
        }

        let majority_required = total_votes
            .checked_mul(config.majority_bps as i128)
            .unwrap_or(0)
            / 10000;

        if proposal.votes_for >= majority_required {
            Ok(ProposalStatus::Succeeded)
        } else {
            Ok(ProposalStatus::Failed)
        }
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{
        contract, contractimpl,
        testutils::{Address as _, Ledger as _},
        Env,
    };

    // ── Mock RiskAssessment ──────────────────────────────────────────────────
    // Used so execute_proposal's cross-contract call has a real target.

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

    // ── Helpers ──────────────────────────────────────────────────────────────

    /// Returns (env, admin, token, mock_risk_assessment_address)
    fn setup_env() -> (Env, Address, Address, Address) {
        let env = Env::default();
        let admin = Address::generate(&env);
        let token = Address::generate(&env);
        let mock_risk = env.register_contract(None, MockRiskAssessment);
        (env, admin, token, mock_risk)
    }

    // ========================================================================
    // Initialization Tests
    // ========================================================================

    #[test]
    fn test_initialize_success() {
        let (env, admin, token, risk_assessment) = setup_env();
        let contract_id = env.register_contract(None, Governance);

        env.mock_all_auths();

        env.as_contract(&contract_id, || {
            let result = Governance::initialize(
                env.clone(),
                admin.clone(),
                token.clone(),
                risk_assessment.clone(),
            );
            assert!(result.is_ok());

            let stored_admin = Governance::admin(env.clone());
            assert_eq!(stored_admin, admin);

            let config = Governance::get_config(env.clone());
            assert_eq!(config.voting_period, 604800);
            assert_eq!(config.timelock_period, 86400);
            assert_eq!(config.quorum_bps, 1000);
            assert_eq!(config.majority_bps, 5000);
        });
    }

    #[test]
    fn test_initialize_already_initialized() {
        let (env, admin, token, risk_assessment) = setup_env();
        let contract_id = env.register_contract(None, Governance);

        env.mock_all_auths();

        env.as_contract(&contract_id, || {
            Governance::initialize(
                env.clone(),
                admin.clone(),
                token.clone(),
                risk_assessment.clone(),
            )
            .unwrap();

            let result = Governance::initialize(
                env.clone(),
                admin.clone(),
                token.clone(),
                risk_assessment.clone(),
            );
            assert_eq!(result, Err(ContractError::AlreadyInitialized));
        });
    }

    // ========================================================================
    // Proposal Tests
    // ========================================================================

    #[test]
    fn test_create_proposal_success() {
        let (env, admin, token, risk_assessment) = setup_env();
        let contract_id = env.register_contract(None, Governance);

        env.mock_all_auths();

        env.as_contract(&contract_id, || {
            Governance::initialize(
                env.clone(),
                admin.clone(),
                token.clone(),
                risk_assessment.clone(),
            )
            .unwrap();

            let proposer = Address::generate(&env);
            Governance::set_voting_power(env.clone(), proposer.clone(), 2000);

            let proposal_id = Governance::create_proposal(
                env.clone(),
                proposer.clone(),
                risk_assessment.clone(),
                symbol_short!("liq_thr"),
                7500,
            )
            .unwrap();

            assert_eq!(proposal_id, 0);

            let proposal = Governance::get_proposal(env.clone(), proposal_id).unwrap();
            assert_eq!(proposal.proposer, proposer);
            assert_eq!(proposal.new_value, 7500);
            assert_eq!(proposal.votes_for, 0);
            assert_eq!(proposal.votes_against, 0);
            assert!(!proposal.executed);
        });
    }

    #[test]
    fn test_create_proposal_insufficient_voting_power() {
        let (env, admin, token, risk_assessment) = setup_env();
        let contract_id = env.register_contract(None, Governance);

        env.mock_all_auths();

        env.as_contract(&contract_id, || {
            Governance::initialize(
                env.clone(),
                admin.clone(),
                token.clone(),
                risk_assessment.clone(),
            )
            .unwrap();

            let proposer = Address::generate(&env);
            Governance::set_voting_power(env.clone(), proposer.clone(), 500); // Below minimum

            let result = Governance::create_proposal(
                env.clone(),
                proposer.clone(),
                risk_assessment.clone(),
                symbol_short!("liq_thr"),
                7500,
            );

            assert_eq!(result, Err(ContractError::InsufficientVotingPower));
        });
    }

    #[test]
    fn test_create_proposal_invalid_parameter() {
        let (env, admin, token, risk_assessment) = setup_env();
        let contract_id = env.register_contract(None, Governance);

        env.mock_all_auths();

        env.as_contract(&contract_id, || {
            Governance::initialize(
                env.clone(),
                admin.clone(),
                token.clone(),
                risk_assessment.clone(),
            )
            .unwrap();

            let proposer = Address::generate(&env);
            Governance::set_voting_power(env.clone(), proposer.clone(), 2000);

            // Invalid parameter symbol
            let result = Governance::create_proposal(
                env.clone(),
                proposer.clone(),
                risk_assessment.clone(),
                symbol_short!("invalid"),
                7500,
            );

            assert_eq!(result, Err(ContractError::InvalidParameter));
        });
    }

    #[test]
    fn test_create_proposal_invalid_value() {
        let (env, admin, token, risk_assessment) = setup_env();
        let contract_id = env.register_contract(None, Governance);

        env.mock_all_auths();

        env.as_contract(&contract_id, || {
            Governance::initialize(
                env.clone(),
                admin.clone(),
                token.clone(),
                risk_assessment.clone(),
            )
            .unwrap();

            let proposer = Address::generate(&env);
            Governance::set_voting_power(env.clone(), proposer.clone(), 2000);

            // Value too high for liquidation threshold
            let result = Governance::create_proposal(
                env.clone(),
                proposer.clone(),
                risk_assessment.clone(),
                symbol_short!("liq_thr"),
                10000, // > 9500 max
            );

            assert_eq!(result, Err(ContractError::InvalidValue));
        });
    }

    #[test]
    fn test_get_proposal_count() {
        let (env, admin, token, risk_assessment) = setup_env();
        let contract_id = env.register_contract(None, Governance);

        env.mock_all_auths();

        env.as_contract(&contract_id, || {
            Governance::initialize(
                env.clone(),
                admin.clone(),
                token.clone(),
                risk_assessment.clone(),
            )
            .unwrap();

            assert_eq!(Governance::get_proposal_count(env.clone()), 0);

            let proposer = Address::generate(&env);
            Governance::set_voting_power(env.clone(), proposer.clone(), 2000);

            Governance::create_proposal(
                env.clone(),
                proposer.clone(),
                risk_assessment.clone(),
                symbol_short!("liq_thr"),
                7500,
            )
            .unwrap();

            assert_eq!(Governance::get_proposal_count(env.clone()), 1);
        });
    }

    // ========================================================================
    // Voting Tests
    // ========================================================================

    #[test]
    fn test_cast_vote_success() {
        let (env, admin, token, risk_assessment) = setup_env();
        let contract_id = env.register_contract(None, Governance);

        env.mock_all_auths();

        env.as_contract(&contract_id, || {
            Governance::initialize(
                env.clone(),
                admin.clone(),
                token.clone(),
                risk_assessment.clone(),
            )
            .unwrap();

            let proposer = Address::generate(&env);
            Governance::set_voting_power(env.clone(), proposer.clone(), 2000);

            let proposal_id = Governance::create_proposal(
                env.clone(),
                proposer.clone(),
                risk_assessment.clone(),
                symbol_short!("liq_thr"),
                7500,
            )
            .unwrap();

            let voter = Address::generate(&env);
            Governance::set_voting_power(env.clone(), voter.clone(), 5000);

            let result = Governance::cast_vote(env.clone(), proposal_id, voter.clone(), true);
            assert!(result.is_ok());

            let proposal = Governance::get_proposal(env.clone(), proposal_id).unwrap();
            assert_eq!(proposal.votes_for, 5000);
            assert_eq!(proposal.votes_against, 0);

            let vote = Governance::get_vote(env.clone(), proposal_id, voter).unwrap();
            assert_eq!(vote.support, true);
            assert_eq!(vote.voting_power, 5000);
        });
    }

    #[test]
    fn test_cast_vote_against() {
        let (env, admin, token, risk_assessment) = setup_env();
        let contract_id = env.register_contract(None, Governance);

        env.mock_all_auths();

        env.as_contract(&contract_id, || {
            Governance::initialize(
                env.clone(),
                admin.clone(),
                token.clone(),
                risk_assessment.clone(),
            )
            .unwrap();

            let proposer = Address::generate(&env);
            Governance::set_voting_power(env.clone(), proposer.clone(), 2000);

            let proposal_id = Governance::create_proposal(
                env.clone(),
                proposer.clone(),
                risk_assessment.clone(),
                symbol_short!("liq_thr"),
                7500,
            )
            .unwrap();

            let voter = Address::generate(&env);
            Governance::set_voting_power(env.clone(), voter.clone(), 3000);

            Governance::cast_vote(env.clone(), proposal_id, voter.clone(), false).unwrap();

            let proposal = Governance::get_proposal(env.clone(), proposal_id).unwrap();
            assert_eq!(proposal.votes_for, 0);
            assert_eq!(proposal.votes_against, 3000);
        });
    }

    #[test]
    fn test_cast_vote_voting_ended() {
        let (env, admin, token, risk_assessment) = setup_env();
        let contract_id = env.register_contract(None, Governance);

        env.mock_all_auths();

        env.as_contract(&contract_id, || {
            Governance::initialize(
                env.clone(),
                admin.clone(),
                token.clone(),
                risk_assessment.clone(),
            )
            .unwrap();

            let proposer = Address::generate(&env);
            Governance::set_voting_power(env.clone(), proposer.clone(), 2000);

            let proposal_id = Governance::create_proposal(
                env.clone(),
                proposer.clone(),
                risk_assessment.clone(),
                symbol_short!("liq_thr"),
                7500,
            )
            .unwrap();

            // Advance time past voting period
            env.ledger()
                .set_timestamp(env.ledger().timestamp() + 604801);

            let voter = Address::generate(&env);
            Governance::set_voting_power(env.clone(), voter.clone(), 5000);

            let result = Governance::cast_vote(env.clone(), proposal_id, voter.clone(), true);
            assert_eq!(result, Err(ContractError::VotingEnded));
        });
    }

    #[test]
    fn test_cast_vote_insufficient_power() {
        let (env, admin, token, risk_assessment) = setup_env();
        let contract_id = env.register_contract(None, Governance);

        env.mock_all_auths();

        env.as_contract(&contract_id, || {
            Governance::initialize(
                env.clone(),
                admin.clone(),
                token.clone(),
                risk_assessment.clone(),
            )
            .unwrap();

            let proposer = Address::generate(&env);
            Governance::set_voting_power(env.clone(), proposer.clone(), 2000);

            let proposal_id = Governance::create_proposal(
                env.clone(),
                proposer.clone(),
                risk_assessment.clone(),
                symbol_short!("liq_thr"),
                7500,
            )
            .unwrap();

            let voter = Address::generate(&env);
            Governance::set_voting_power(env.clone(), voter.clone(), 0);

            let result = Governance::cast_vote(env.clone(), proposal_id, voter.clone(), true);
            assert_eq!(result, Err(ContractError::InsufficientVotingPower));
        });
    }

    #[test]
    fn test_multiple_voters() {
        let (env, admin, token, risk_assessment) = setup_env();
        let contract_id = env.register_contract(None, Governance);

        env.mock_all_auths();

        env.as_contract(&contract_id, || {
            Governance::initialize(
                env.clone(),
                admin.clone(),
                token.clone(),
                risk_assessment.clone(),
            )
            .unwrap();

            let proposer = Address::generate(&env);
            Governance::set_voting_power(env.clone(), proposer.clone(), 2000);

            let proposal_id = Governance::create_proposal(
                env.clone(),
                proposer.clone(),
                risk_assessment.clone(),
                symbol_short!("liq_thr"),
                7500,
            )
            .unwrap();

            // Voter 1: For
            let voter1 = Address::generate(&env);
            Governance::set_voting_power(env.clone(), voter1.clone(), 5000);
            Governance::cast_vote(env.clone(), proposal_id, voter1.clone(), true).unwrap();

            // Voter 2: For
            let voter2 = Address::generate(&env);
            Governance::set_voting_power(env.clone(), voter2.clone(), 3000);
            Governance::cast_vote(env.clone(), proposal_id, voter2.clone(), true).unwrap();

            // Voter 3: Against
            let voter3 = Address::generate(&env);
            Governance::set_voting_power(env.clone(), voter3.clone(), 2000);
            Governance::cast_vote(env.clone(), proposal_id, voter3.clone(), false).unwrap();

            let proposal = Governance::get_proposal(env.clone(), proposal_id).unwrap();
            assert_eq!(proposal.votes_for, 8000);
            assert_eq!(proposal.votes_against, 2000);
        });
    }

    // ========================================================================
    // Execution Tests
    // ========================================================================

    #[test]
    fn test_execute_proposal_success() {
        let (env, admin, token, risk_assessment) = setup_env();
        let contract_id = env.register_contract(None, Governance);

        env.mock_all_auths();

        env.as_contract(&contract_id, || {
            Governance::initialize(
                env.clone(),
                admin.clone(),
                token.clone(),
                risk_assessment.clone(),
            )
            .unwrap();

            // Set total voting power
            Governance::set_total_voting_power(env.clone(), 100000).unwrap();

            let proposer = Address::generate(&env);
            Governance::set_voting_power(env.clone(), proposer.clone(), 2000);

            let proposal_id = Governance::create_proposal(
                env.clone(),
                proposer.clone(),
                risk_assessment.clone(),
                symbol_short!("liq_thr"),
                7500,
            )
            .unwrap();

            // Cast votes to reach quorum (10%) and majority (50%)
            let voter1 = Address::generate(&env);
            Governance::set_voting_power(env.clone(), voter1.clone(), 60000);
            Governance::cast_vote(env.clone(), proposal_id, voter1.clone(), true).unwrap();

            // Advance time past voting period and timelock
            env.ledger()
                .set_timestamp(env.ledger().timestamp() + 604800 + 86400 + 1);

            let result = Governance::execute_proposal(env.clone(), proposal_id);
            assert!(result.is_ok());

            let proposal = Governance::get_proposal(env.clone(), proposal_id).unwrap();
            assert!(proposal.executed);
        });
    }

    #[test]
    fn test_execute_proposal_timelock_not_expired() {
        let (env, admin, token, risk_assessment) = setup_env();
        let contract_id = env.register_contract(None, Governance);

        env.mock_all_auths();

        env.as_contract(&contract_id, || {
            Governance::initialize(
                env.clone(),
                admin.clone(),
                token.clone(),
                risk_assessment.clone(),
            )
            .unwrap();

            Governance::set_total_voting_power(env.clone(), 100000).unwrap();

            let proposer = Address::generate(&env);
            Governance::set_voting_power(env.clone(), proposer.clone(), 2000);

            let proposal_id = Governance::create_proposal(
                env.clone(),
                proposer.clone(),
                risk_assessment.clone(),
                symbol_short!("liq_thr"),
                7500,
            )
            .unwrap();

            let voter1 = Address::generate(&env);
            Governance::set_voting_power(env.clone(), voter1.clone(), 60000);
            Governance::cast_vote(env.clone(), proposal_id, voter1.clone(), true).unwrap();

            // Advance time past voting but not timelock
            env.ledger()
                .set_timestamp(env.ledger().timestamp() + 604801);

            let result = Governance::execute_proposal(env.clone(), proposal_id);
            assert_eq!(result, Err(ContractError::TimelockNotExpired));
        });
    }

    #[test]
    fn test_execute_proposal_quorum_not_reached() {
        let (env, admin, token, risk_assessment) = setup_env();
        let contract_id = env.register_contract(None, Governance);

        env.mock_all_auths();

        env.as_contract(&contract_id, || {
            Governance::initialize(
                env.clone(),
                admin.clone(),
                token.clone(),
                risk_assessment.clone(),
            )
            .unwrap();

            Governance::set_total_voting_power(env.clone(), 100000).unwrap();

            let proposer = Address::generate(&env);
            Governance::set_voting_power(env.clone(), proposer.clone(), 2000);

            let proposal_id = Governance::create_proposal(
                env.clone(),
                proposer.clone(),
                risk_assessment.clone(),
                symbol_short!("liq_thr"),
                7500,
            )
            .unwrap();

            // Only 5% votes (below 10% quorum)
            let voter1 = Address::generate(&env);
            Governance::set_voting_power(env.clone(), voter1.clone(), 5000);
            Governance::cast_vote(env.clone(), proposal_id, voter1.clone(), true).unwrap();

            env.ledger()
                .set_timestamp(env.ledger().timestamp() + 604800 + 86400 + 1);

            let result = Governance::execute_proposal(env.clone(), proposal_id);
            assert_eq!(result, Err(ContractError::QuorumNotReached));
        });
    }

    #[test]
    fn test_execute_proposal_majority_not_reached() {
        let (env, admin, token, risk_assessment) = setup_env();
        let contract_id = env.register_contract(None, Governance);

        env.mock_all_auths();

        env.as_contract(&contract_id, || {
            Governance::initialize(
                env.clone(),
                admin.clone(),
                token.clone(),
                risk_assessment.clone(),
            )
            .unwrap();

            Governance::set_total_voting_power(env.clone(), 100000).unwrap();

            let proposer = Address::generate(&env);
            Governance::set_voting_power(env.clone(), proposer.clone(), 2000);

            let proposal_id = Governance::create_proposal(
                env.clone(),
                proposer.clone(),
                risk_assessment.clone(),
                symbol_short!("liq_thr"),
                7500,
            )
            .unwrap();

            // Quorum reached but majority not reached (40% for, 60% against)
            let voter1 = Address::generate(&env);
            Governance::set_voting_power(env.clone(), voter1.clone(), 40000);
            Governance::cast_vote(env.clone(), proposal_id, voter1.clone(), true).unwrap();

            let voter2 = Address::generate(&env);
            Governance::set_voting_power(env.clone(), voter2.clone(), 60000);
            Governance::cast_vote(env.clone(), proposal_id, voter2.clone(), false).unwrap();

            env.ledger()
                .set_timestamp(env.ledger().timestamp() + 604800 + 86400 + 1);

            let result = Governance::execute_proposal(env.clone(), proposal_id);
            assert_eq!(result, Err(ContractError::MajorityNotReached));
        });
    }

    #[test]
    fn test_execute_proposal_already_executed() {
        let (env, admin, token, risk_assessment) = setup_env();
        let contract_id = env.register_contract(None, Governance);

        env.mock_all_auths();

        env.as_contract(&contract_id, || {
            Governance::initialize(
                env.clone(),
                admin.clone(),
                token.clone(),
                risk_assessment.clone(),
            )
            .unwrap();

            Governance::set_total_voting_power(env.clone(), 100000).unwrap();

            let proposer = Address::generate(&env);
            Governance::set_voting_power(env.clone(), proposer.clone(), 2000);

            let proposal_id = Governance::create_proposal(
                env.clone(),
                proposer.clone(),
                risk_assessment.clone(),
                symbol_short!("liq_thr"),
                7500,
            )
            .unwrap();

            let voter1 = Address::generate(&env);
            Governance::set_voting_power(env.clone(), voter1.clone(), 60000);
            Governance::cast_vote(env.clone(), proposal_id, voter1.clone(), true).unwrap();

            env.ledger()
                .set_timestamp(env.ledger().timestamp() + 604800 + 86400 + 1);

            Governance::execute_proposal(env.clone(), proposal_id).unwrap();

            let result = Governance::execute_proposal(env.clone(), proposal_id);
            assert_eq!(result, Err(ContractError::ProposalAlreadyExecuted));
        });
    }

    #[test]
    fn test_has_proposal_passed() {
        let (env, admin, token, risk_assessment) = setup_env();
        let contract_id = env.register_contract(None, Governance);

        env.mock_all_auths();

        env.as_contract(&contract_id, || {
            Governance::initialize(
                env.clone(),
                admin.clone(),
                token.clone(),
                risk_assessment.clone(),
            )
            .unwrap();

            Governance::set_total_voting_power(env.clone(), 100000).unwrap();

            let proposer = Address::generate(&env);
            Governance::set_voting_power(env.clone(), proposer.clone(), 2000);

            let proposal_id = Governance::create_proposal(
                env.clone(),
                proposer.clone(),
                risk_assessment.clone(),
                symbol_short!("liq_thr"),
                7500,
            )
            .unwrap();

            // Initially not passed
            assert!(!Governance::has_proposal_passed(env.clone(), proposal_id).unwrap());

            // Cast votes to pass
            let voter1 = Address::generate(&env);
            Governance::set_voting_power(env.clone(), voter1.clone(), 60000);
            Governance::cast_vote(env.clone(), proposal_id, voter1.clone(), true).unwrap();

            // Now passed
            assert!(Governance::has_proposal_passed(env.clone(), proposal_id).unwrap());
        });
    }

    // ========================================================================
    // Admin Tests
    // ========================================================================

    #[test]
    fn test_update_config() {
        let (env, admin, token, risk_assessment) = setup_env();
        let contract_id = env.register_contract(None, Governance);

        env.mock_all_auths();

        env.as_contract(&contract_id, || {
            Governance::initialize(
                env.clone(),
                admin.clone(),
                token.clone(),
                risk_assessment.clone(),
            )
            .unwrap();

            let new_config = GovernanceConfig {
                voting_period: 1209600,  // 14 days
                timelock_period: 172800, // 48 hours
                quorum_bps: 1500,        // 15%
                majority_bps: 6000,      // 60%
                min_voting_power: 2000,
            };

            let result = Governance::update_config(env.clone(), new_config.clone());
            assert!(result.is_ok());

            let config = Governance::get_config(env.clone());
            assert_eq!(config.voting_period, 1209600);
            assert_eq!(config.timelock_period, 172800);
            assert_eq!(config.quorum_bps, 1500);
            assert_eq!(config.majority_bps, 6000);
        });
    }

    #[test]
    fn test_cancel_proposal() {
        let (env, admin, token, risk_assessment) = setup_env();
        let contract_id = env.register_contract(None, Governance);

        env.mock_all_auths();

        env.as_contract(&contract_id, || {
            Governance::initialize(
                env.clone(),
                admin.clone(),
                token.clone(),
                risk_assessment.clone(),
            )
            .unwrap();

            let proposer = Address::generate(&env);
            Governance::set_voting_power(env.clone(), proposer.clone(), 2000);

            let proposal_id = Governance::create_proposal(
                env.clone(),
                proposer.clone(),
                risk_assessment.clone(),
                symbol_short!("liq_thr"),
                7500,
            )
            .unwrap();

            let result = Governance::cancel_proposal(env.clone(), proposal_id);
            assert!(result.is_ok());

            let proposal = Governance::get_proposal(env.clone(), proposal_id).unwrap();
            assert!(proposal.executed); // Marked as executed to prevent execution
        });
    }

    #[test]
    fn test_get_proposal_status_lifecycle() {
        let (env, admin, token, risk_assessment) = setup_env();
        let contract_id = env.register_contract(None, Governance);

        env.mock_all_auths();

        env.as_contract(&contract_id, || {
            Governance::initialize(
                env.clone(),
                admin.clone(),
                token.clone(),
                risk_assessment.clone(),
            )
            .unwrap();

            Governance::set_total_voting_power(env.clone(), 100000).unwrap();

            let proposer = Address::generate(&env);
            Governance::set_voting_power(env.clone(), proposer.clone(), 2000);

            let proposal_id = Governance::create_proposal(
                env.clone(),
                proposer.clone(),
                risk_assessment.clone(),
                symbol_short!("liq_thr"),
                7500,
            )
            .unwrap();

            // Should be Active during voting
            assert_eq!(
                Governance::get_proposal_status(env.clone(), proposal_id).unwrap(),
                ProposalStatus::Active
            );

            // Cast enough votes to pass
            let voter = Address::generate(&env);
            Governance::set_voting_power(env.clone(), voter.clone(), 60000);
            Governance::cast_vote(env.clone(), proposal_id, voter.clone(), true).unwrap();

            // Advance past voting period only → Succeeded (not yet executed)
            env.ledger()
                .set_timestamp(env.ledger().timestamp() + 604801);

            assert_eq!(
                Governance::get_proposal_status(env.clone(), proposal_id).unwrap(),
                ProposalStatus::Succeeded
            );

            // Advance past timelock and execute
            env.ledger()
                .set_timestamp(env.ledger().timestamp() + 86401);

            Governance::execute_proposal(env.clone(), proposal_id).unwrap();

            assert_eq!(
                Governance::get_proposal_status(env.clone(), proposal_id).unwrap(),
                ProposalStatus::Executed
            );
        });
    }

    #[test]
    fn test_get_proposal_status_failed() {
        let (env, admin, token, risk_assessment) = setup_env();
        let contract_id = env.register_contract(None, Governance);

        env.mock_all_auths();

        env.as_contract(&contract_id, || {
            Governance::initialize(
                env.clone(),
                admin.clone(),
                token.clone(),
                risk_assessment.clone(),
            )
            .unwrap();

            Governance::set_total_voting_power(env.clone(), 100000).unwrap();

            let proposer = Address::generate(&env);
            Governance::set_voting_power(env.clone(), proposer.clone(), 2000);

            let proposal_id = Governance::create_proposal(
                env.clone(),
                proposer.clone(),
                risk_assessment.clone(),
                symbol_short!("liq_thr"),
                7500,
            )
            .unwrap();

            // Only 2% votes → below 10% quorum → Failed
            let voter = Address::generate(&env);
            Governance::set_voting_power(env.clone(), voter.clone(), 2000);
            Governance::cast_vote(env.clone(), proposal_id, voter.clone(), true).unwrap();

            env.ledger()
                .set_timestamp(env.ledger().timestamp() + 604801);

            assert_eq!(
                Governance::get_proposal_status(env.clone(), proposal_id).unwrap(),
                ProposalStatus::Failed
            );
        });
    }
}

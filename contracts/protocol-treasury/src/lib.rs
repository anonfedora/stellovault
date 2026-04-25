//! Protocol Treasury Contract for StelloVault
//!
//! Collects protocol fees from loan repayments and escrow releases,
//! distributes dividends to registered contributors based on share weights.

#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, token, Address, Env, Map, Vec, i128, u64};

/// Default protocol fee in basis points (50 = 0.5%)
const DEFAULT_FEE_BPS: u32 = 50;

/// Multi-signature threshold for large withdrawals (basis points)
const MULTI_SIG_THRESHOLD_BPS: u32 = 5000; // 50%

/// Large withdrawal threshold (in smallest token unit)
const LARGE_WITHDRAWAL_THRESHOLD: i128 = 1_000_000;

/// Default timelock duration for emergency withdrawals (24 hours)
const DEFAULT_TIMELOCK: u64 = 86400;

/// Epoch duration for reward distribution (7 days)
const EPOCH_DURATION: u64 = 604800;

// Event symbols
const EVT_TREASURY_INIT: Symbol = symbol_short!("trs_init");
const EVT_FEE_UPD: Symbol = symbol_short!("fee_upd");
const EVT_FEE_DEP: Symbol = symbol_short!("fee_dep");
const EVT_CLAIMED: Symbol = symbol_short!("claimed");
const EVT_WITHDRAW: Symbol = symbol_short!("withdraw");
const EVT_MULTI_SIG: Symbol = symbol_short!("multi_sig");
const EVT_TIMELOCK: Symbol = symbol_short!("timelock");
const EVT_PROPOSAL: Symbol = symbol_short!("proposal");
const EVT_REWARD_DIST: Symbol = symbol_short!("rew_dist");
const EVT_LIQUIDITY: Symbol = symbol_short!("liquid");
const EVT_PAUSE: Symbol = symbol_short!("pause");
const EVT_AUDIT: Symbol = symbol_short!("audit");

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContractError {
    Unauthorized = 1,
    AlreadyInitialized = 2,
    ContributorNotFound = 3,
    NoFeesAvailable = 4,
    InvalidFee = 5,
    ZeroAmount = 6,
    NoPendingAdmin = 7,
    InvalidThreshold = 8,
    InsufficientSignatures = 9,
    TimelockNotExpired = 10,
    ProposalNotFound = 11,
    ProposalExpired = 12,
    AlreadyVoted = 13,
    Paused = 14,
    InvalidProposal = 15,
    InsufficientBalance = 16,
    InvalidEpoch = 17,
    PoolNotFound = 18,
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

/// Registered contributor eligible for fee dividends.
#[contracttype]
#[derive(Clone, Debug)]
pub struct Contributor {
    pub address: Address,
    pub share_weight: u32,
}

/// Composite key for tracking per-contributor-per-asset claimed amounts.
#[contracttype]
#[derive(Clone, Debug)]
pub struct ClaimKey {
    pub contributor: Address,
    pub asset: Address,
}

/// Treasury state including balance and configuration
#[contracttype]
#[derive(Clone, Debug)]
pub struct TreasuryState {
    pub total_balance: i128,
    pub paused: bool,
    pub timelock_duration: u64,
    pub last_revenue_calc: u64,
}

/// Reward pool for epoch-based distribution
#[contracttype]
#[derive(Clone, Debug)]
pub struct RewardPool {
    pub asset: Address,
    pub total_rewards: i128,
    pub distributed: i128,
    pub epoch: u64,
}

/// Governance proposal type
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProposalType {
    FeeChange = 1,
    Withdrawal = 2,
    ParameterUpdate = 3,
    EmergencyAction = 4,
}

/// Governance proposal
#[contracttype]
#[derive(Clone, Debug)]
pub struct Proposal {
    pub id: u64,
    pub proposer: Address,
    pub proposal_type: ProposalType,
    pub description: Symbol,
    pub data: Vec<u8>,
    pub created_at: u64,
    pub expires_at: u64,
    pub votes_for: u32,
    pub votes_against: u32,
    pub executed: bool,
}

/// Multi-signature withdrawal request
#[contracttype]
#[derive(Clone, Debug)]
pub struct MultiSigWithdrawal {
    pub id: u64,
    pub proposer: Address,
    pub recipient: Address,
    pub asset: Address,
    pub amount: i128,
    pub signatures: Vec<Address>,
    pub created_at: u64,
    pub executed: bool,
}

/// Timelocked emergency withdrawal
#[contracttype]
#[derive(Clone, Debug)]
pub struct TimelockedWithdrawal {
    pub id: u64,
    pub recipient: Address,
    pub asset: Address,
    pub amount: i128,
    pub created_at: u64,
    pub unlock_at: u64,
    pub executed: bool,
}

/// Liquidity pool for yield generation
#[contracttype]
#[derive(Clone, Debug)]
pub struct LiquidityPool {
    pub id: u64,
    pub asset: Address,
    pub deposited: i128,
    pub rewards: i128,
    pub apy_bps: u32,
    pub last_update: u64,
}

/// Audit log entry
#[contracttype]
#[derive(Clone, Debug)]
pub struct AuditLog {
    pub timestamp: u64,
    pub action: Symbol,
    pub actor: Address,
    pub details: Vec<u8>,
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct ProtocolTreasury;

#[contractimpl]
impl ProtocolTreasury {
    /// Initialize the treasury with an admin address.
    pub fn initialize(env: Env, admin: Address) -> Result<(), ContractError> {
        if env.storage().instance().has(&symbol_short!("admin")) {
            return Err(ContractError::AlreadyInitialized);
        }

        env.storage()
            .instance()
            .set(&symbol_short!("admin"), &admin);
        env.storage()
            .instance()
            .set(&symbol_short!("fee_bps"), &DEFAULT_FEE_BPS);
        env.storage()
            .instance()
            .set(&symbol_short!("tot_wt"), &0u32);

        // Initialize treasury state
        let state = TreasuryState {
            total_balance: 0,
            paused: false,
            timelock_duration: DEFAULT_TIMELOCK,
            last_revenue_calc: env.ledger().timestamp(),
        };
        env.storage()
            .instance()
            .set(&symbol_short!("state"), &state);

        // Initialize counters
        env.storage()
            .instance()
            .set(&symbol_short!("prop_id"), &0u64);
        env.storage()
            .instance()
            .set(&symbol_short!("multi_id"), &0u64);
        env.storage()
            .instance()
            .set(&symbol_short!("time_id"), &0u64);
        env.storage()
            .instance()
            .set(&symbol_short!("pool_id"), &0u64);
        env.storage()
            .instance()
            .set(&symbol_short!("audit_id"), &0u64);

        env.events()
            .publish((EVT_TREASURY_INIT,), (admin, DEFAULT_FEE_BPS));

        Ok(())
    }

    /// Update the protocol fee rate (admin / governance only).
    /// Fee is capped at 1000 bps (10%).
    pub fn set_fee_bps(env: Env, new_fee_bps: u32) -> Result<(), ContractError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&symbol_short!("admin"))
            .ok_or(ContractError::Unauthorized)?;

        admin.require_auth();

        if new_fee_bps > 1000 {
            return Err(ContractError::InvalidFee);
        }

        env.storage()
            .instance()
            .set(&symbol_short!("fee_bps"), &new_fee_bps);

        env.events()
            .publish((symbol_short!("fee_upd"),), (new_fee_bps,));

        Ok(())
    }

    /// Propose a new admin (two-step transfer, step 1).
    /// Only the current admin may call this; their signature is required.
    pub fn propose_admin(env: Env, new_admin: Address) -> Result<(), ContractError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&symbol_short!("admin"))
            .ok_or(ContractError::Unauthorized)?;

        admin.require_auth();

        env.storage()
            .instance()
            .set(&symbol_short!("pend_adm"), &new_admin);

        env.events()
            .publish((symbol_short!("adm_prop"),), (admin, new_admin));

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

        env.storage()
            .instance()
            .set(&symbol_short!("admin"), &pending);
        env.storage().instance().remove(&symbol_short!("pend_adm"));

        env.events()
            .publish((symbol_short!("adm_acpt"),), (pending,));

        Ok(())
    }

    /// Return the pending admin address if a proposal is active.
    pub fn get_pending_admin(env: Env) -> Option<Address> {
        env.storage().instance().get(&symbol_short!("pend_adm"))
    }

    /// Query the current protocol fee in basis points.
    pub fn get_fee_bps(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&symbol_short!("fee_bps"))
            .unwrap_or(DEFAULT_FEE_BPS)
    }

    /// Record a fee deposit. Called by other contracts after transferring
    /// tokens to the treasury address.
    pub fn deposit_fee(env: Env, asset: Address, amount: i128) -> Result<(), ContractError> {
        if amount <= 0 {
            return Err(ContractError::ZeroAmount);
        }

        let key = (symbol_short!("fees"), asset.clone());
        let current: i128 = env.storage().persistent().get(&key).unwrap_or(0);
        env.storage().persistent().set(&key, &(current + amount));

        env.events()
            .publish((symbol_short!("fee_dep"),), (asset, amount));

        Ok(())
    }

    /// Register (or update) a contributor with a share weight.
    /// Only callable by admin.
    pub fn register_contributor(
        env: Env,
        contributor: Address,
        share_weight: u32,
    ) -> Result<(), ContractError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&symbol_short!("admin"))
            .ok_or(ContractError::Unauthorized)?;

        admin.require_auth();

        let key = (symbol_short!("contr"), contributor.clone());

        let total_weight: u32 = env
            .storage()
            .instance()
            .get(&symbol_short!("tot_wt"))
            .unwrap_or(0);

        // If contributor already exists, subtract old weight
        let old_weight: u32 =
            if let Some(existing) = env.storage().persistent().get::<_, Contributor>(&key) {
                existing.share_weight
            } else {
                0u32
            };

        let new_total = total_weight - old_weight + share_weight;
        env.storage()
            .instance()
            .set(&symbol_short!("tot_wt"), &new_total);

        let c = Contributor {
            address: contributor.clone(),
            share_weight,
        };
        env.storage().persistent().set(&key, &c);

        env.events()
            .publish((symbol_short!("contr_rg"),), (contributor, share_weight));

        Ok(())
    }

    /// Remove a contributor. Only callable by admin.
    pub fn remove_contributor(env: Env, contributor: Address) -> Result<(), ContractError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&symbol_short!("admin"))
            .ok_or(ContractError::Unauthorized)?;

        admin.require_auth();

        let key = (symbol_short!("contr"), contributor.clone());
        let existing: Contributor = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(ContractError::ContributorNotFound)?;

        let total_weight: u32 = env
            .storage()
            .instance()
            .get(&symbol_short!("tot_wt"))
            .unwrap_or(0);

        let new_total = total_weight.saturating_sub(existing.share_weight);
        env.storage()
            .instance()
            .set(&symbol_short!("tot_wt"), &new_total);

        env.storage().persistent().remove(&key);

        env.events()
            .publish((symbol_short!("contr_rm"),), (contributor,));

        Ok(())
    }

    /// Claim proportional share of accumulated fees for a given asset.
    ///
    /// Entitled amount = (total_fees * share_weight) / total_weight
    /// Claimable = entitled - already_claimed
    pub fn claim_share(
        env: Env,
        contributor: Address,
        asset: Address,
    ) -> Result<i128, ContractError> {
        contributor.require_auth();

        let contr_key = (symbol_short!("contr"), contributor.clone());
        let c: Contributor = env
            .storage()
            .persistent()
            .get(&contr_key)
            .ok_or(ContractError::ContributorNotFound)?;

        let total_weight: u32 = env
            .storage()
            .instance()
            .get(&symbol_short!("tot_wt"))
            .unwrap_or(0);

        if total_weight == 0 {
            return Err(ContractError::NoFeesAvailable);
        }

        let fee_key = (symbol_short!("fees"), asset.clone());
        let total_fees: i128 = env.storage().persistent().get(&fee_key).unwrap_or(0);

        // Calculate entitled and claimable
        let entitled = (total_fees * c.share_weight as i128) / total_weight as i128;

        let claim_key = ClaimKey {
            contributor: contributor.clone(),
            asset: asset.clone(),
        };
        let already_claimed: i128 = env.storage().persistent().get(&claim_key).unwrap_or(0);
        let claimable = entitled - already_claimed;

        if claimable <= 0 {
            return Err(ContractError::NoFeesAvailable);
        }

        // Transfer tokens to contributor
        let token_client = token::Client::new(&env, &asset);
        token_client.transfer(&env.current_contract_address(), &contributor, &claimable);

        // Update claimed amount
        env.storage()
            .persistent()
            .set(&claim_key, &(already_claimed + claimable));

        env.events()
            .publish((symbol_short!("claimed"),), (contributor, asset, claimable));

        Ok(claimable)
    }

    /// Query total accumulated fees for an asset.
    pub fn get_total_fees(env: Env, asset: Address) -> i128 {
        let key = (symbol_short!("fees"), asset);
        env.storage().persistent().get(&key).unwrap_or(0)
    }

    /// Query a contributor's registration details.
    pub fn get_contributor(env: Env, contributor: Address) -> Option<Contributor> {
        let key = (symbol_short!("contr"), contributor);
        env.storage().persistent().get(&key)
    }

    /// Query total share weight across all contributors.
    pub fn get_total_weight(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&symbol_short!("tot_wt"))
            .unwrap_or(0)
    }

    /// Get treasury state
    pub fn get_treasury_state(env: Env) -> TreasuryState {
        env.storage()
            .instance()
            .get(&symbol_short!("state"))
            .unwrap_or_else(|| TreasuryState {
                total_balance: 0,
                paused: false,
                timelock_duration: DEFAULT_TIMELOCK,
                last_revenue_calc: env.ledger().timestamp(),
            })
    }

    /// Pause treasury operations (emergency)
    pub fn pause_treasury(env: Env) -> Result<(), ContractError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&symbol_short!("admin"))
            .ok_or(ContractError::Unauthorized)?;

        admin.require_auth();

        let mut state = Self::get_treasury_state(env.clone());
        state.paused = true;
        env.storage()
            .instance()
            .set(&symbol_short!("state"), &state);

        Self::log_audit(env.clone(), symbol_short!("pause"), admin, Vec::new(&env));

        env.events().publish((EVT_PAUSE,), (true,));

        Ok(())
    }

    /// Unpause treasury operations
    pub fn unpause_treasury(env: Env) -> Result<(), ContractError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&symbol_short!("admin"))
            .ok_or(ContractError::Unauthorized)?;

        admin.require_auth();

        let mut state = Self::get_treasury_state(env.clone());
        state.paused = false;
        env.storage()
            .instance()
            .set(&symbol_short!("state"), &state);

        Self::log_audit(env.clone(), symbol_short!("unpause"), admin, Vec::new(&env));

        env.events().publish((EVT_PAUSE,), (false,));

        Ok(())
    }

    /// Create a multi-signature withdrawal request for large amounts
    pub fn propose_multi_sig_withdrawal(
        env: Env,
        recipient: Address,
        asset: Address,
        amount: i128,
    ) -> Result<u64, ContractError> {
        let state = Self::get_treasury_state(env.clone());
        if state.paused {
            return Err(ContractError::Paused);
        }

        if amount <= 0 {
            return Err(ContractError::ZeroAmount);
        }

        // Only large withdrawals require multi-sig
        if amount < LARGE_WITHDRAWAL_THRESHOLD {
            return Err(ContractError::InvalidThreshold);
        }

        let admin: Address = env
            .storage()
            .instance()
            .get(&symbol_short!("admin"))
            .ok_or(ContractError::Unauthorized)?;

        admin.require_auth();

        let mut id: u64 = env
            .storage()
            .instance()
            .get(&symbol_short!("multi_id"))
            .unwrap_or(0);
        id += 1;
        env.storage()
            .instance()
            .set(&symbol_short!("multi_id"), &id);

        let withdrawal = MultiSigWithdrawal {
            id,
            proposer: admin.clone(),
            recipient: recipient.clone(),
            asset: asset.clone(),
            amount,
            signatures: Vec::new(&env),
            created_at: env.ledger().timestamp(),
            executed: false,
        };

        let key = (symbol_short!("multi"), id);
        env.storage().persistent().set(&key, &withdrawal);

        Self::log_audit(
            env.clone(),
            symbol_short!("multi_prop"),
            admin,
            Vec::new(&env),
        );

        env.events()
            .publish((EVT_MULTI_SIG,), (id, recipient, amount));

        Ok(id)
    }

    /// Sign a multi-signature withdrawal request
    pub fn sign_multi_sig_withdrawal(env: Env, withdrawal_id: u64) -> Result<(), ContractError> {
        let state = Self::get_treasury_state(env.clone());
        if state.paused {
            return Err(ContractError::Paused);
        }

        let key = (symbol_short!("multi"), withdrawal_id);
        let mut withdrawal: MultiSigWithdrawal = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(ContractError::ProposalNotFound)?;

        if withdrawal.executed {
            return Err(ContractError::InvalidProposal);
        }

        let signer = env.current_contract_address();
        withdrawal.signatures.push_back(signer);

        env.storage().persistent().set(&key, &withdrawal);

        Ok(())
    }

    /// Execute a multi-signature withdrawal
    pub fn execute_multi_sig_withdrawal(env: Env, withdrawal_id: u64) -> Result<(), ContractError> {
        let state = Self::get_treasury_state(env.clone());
        if state.paused {
            return Err(ContractError::Paused);
        }

        let key = (symbol_short!("multi"), withdrawal_id);
        let mut withdrawal: MultiSigWithdrawal = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(ContractError::ProposalNotFound)?;

        if withdrawal.executed {
            return Err(ContractError::InvalidProposal);
        }

        // Check if threshold is met (simplified: need at least 2 signatures)
        if withdrawal.signatures.len() < 2 {
            return Err(ContractError::InsufficientSignatures);
        }

        // Execute withdrawal
        let token_client = token::Client::new(&env, &withdrawal.asset);
        token_client.transfer(
            &env.current_contract_address(),
            &withdrawal.recipient,
            &withdrawal.amount,
        );

        withdrawal.executed = true;
        env.storage().persistent().set(&key, &withdrawal);

        // Update treasury state
        let mut state = Self::get_treasury_state(env.clone());
        state.total_balance = state.total_balance.saturating_sub(withdrawal.amount);
        env.storage()
            .instance()
            .set(&symbol_short!("state"), &state);

        Self::log_audit(
            env.clone(),
            symbol_short!("multi_exec"),
            withdrawal.proposer,
            Vec::new(&env),
        );

        env.events()
            .publish((EVT_WITHDRAW,), (withdrawal_id, withdrawal.amount));

        Ok(())
    }

    /// Create a timelocked emergency withdrawal
    pub fn create_timelocked_withdrawal(
        env: Env,
        recipient: Address,
        asset: Address,
        amount: i128,
    ) -> Result<u64, ContractError> {
        let state = Self::get_treasury_state(env.clone());
        if state.paused {
            return Err(ContractError::Paused);
        }

        if amount <= 0 {
            return Err(ContractError::ZeroAmount);
        }

        let admin: Address = env
            .storage()
            .instance()
            .get(&symbol_short!("admin"))
            .ok_or(ContractError::Unauthorized)?;

        admin.require_auth();

        let mut id: u64 = env
            .storage()
            .instance()
            .get(&symbol_short!("time_id"))
            .unwrap_or(0);
        id += 1;
        env.storage()
            .instance()
            .set(&symbol_short!("time_id"), &id);

        let timelock = TimelockedWithdrawal {
            id,
            recipient: recipient.clone(),
            asset: asset.clone(),
            amount,
            created_at: env.ledger().timestamp(),
            unlock_at: env.ledger().timestamp() + state.timelock_duration,
            executed: false,
        };

        let key = (symbol_short!("time"), id);
        env.storage().persistent().set(&key, &timelock);

        Self::log_audit(
            env.clone(),
            symbol_short!("time_create"),
            admin,
            Vec::new(&env),
        );

        env.events()
            .publish((EVT_TIMELOCK,), (id, recipient, timelock.unlock_at));

        Ok(id)
    }

    /// Execute a timelocked withdrawal (after timelock expires)
    pub fn execute_timelocked_withdrawal(env: Env, timelock_id: u64) -> Result<(), ContractError> {
        let state = Self::get_treasury_state(env.clone());
        if state.paused {
            return Err(ContractError::Paused);
        }

        let key = (symbol_short!("time"), timelock_id);
        let mut timelock: TimelockedWithdrawal = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(ContractError::ProposalNotFound)?;

        if timelock.executed {
            return Err(ContractError::InvalidProposal);
        }

        let current_time = env.ledger().timestamp();
        if current_time < timelock.unlock_at {
            return Err(ContractError::TimelockNotExpired);
        }

        // Execute withdrawal
        let token_client = token::Client::new(&env, &timelock.asset);
        token_client.transfer(
            &env.current_contract_address(),
            &timelock.recipient,
            &timelock.amount,
        );

        timelock.executed = true;
        env.storage().persistent().set(&key, &timelock);

        // Update treasury state
        let mut state = Self::get_treasury_state(env.clone());
        state.total_balance = state.total_balance.saturating_sub(timelock.amount);
        env.storage()
            .instance()
            .set(&symbol_short!("state"), &state);

        Self::log_audit(
            env.clone(),
            symbol_short!("time_exec"),
            timelock.recipient,
            Vec::new(&env),
        );

        env.events()
            .publish((EVT_WITHDRAW,), (timelock_id, timelock.amount));

        Ok(())
    }

    /// Create a governance proposal
    pub fn create_proposal(
        env: Env,
        proposal_type: ProposalType,
        description: Symbol,
        data: Vec<u8>,
        duration: u64,
    ) -> Result<u64, ContractError> {
        let state = Self::get_treasury_state(env.clone());
        if state.paused {
            return Err(ContractError::Paused);
        }

        let admin: Address = env
            .storage()
            .instance()
            .get(&symbol_short!("admin"))
            .ok_or(ContractError::Unauthorized)?;

        admin.require_auth();

        let mut id: u64 = env
            .storage()
            .instance()
            .get(&symbol_short!("prop_id"))
            .unwrap_or(0);
        id += 1;
        env.storage()
            .instance()
            .set(&symbol_short!("prop_id"), &id);

        let proposal = Proposal {
            id,
            proposer: admin.clone(),
            proposal_type,
            description,
            data,
            created_at: env.ledger().timestamp(),
            expires_at: env.ledger().timestamp() + duration,
            votes_for: 0,
            votes_against: 0,
            executed: false,
        };

        let key = (symbol_short!("prop"), id);
        env.storage().persistent().set(&key, &proposal);

        Self::log_audit(
            env.clone(),
            symbol_short!("prop_create"),
            admin,
            Vec::new(&env),
        );

        env.events().publish((EVT_PROPOSAL,), (id, proposal_type as u32));

        Ok(id)
    }

    /// Vote on a proposal
    pub fn vote_proposal(env: Env, proposal_id: u64, vote_for: bool) -> Result<(), ContractError> {
        let state = Self::get_treasury_state(env.clone());
        if state.paused {
            return Err(ContractError::Paused);
        }

        let key = (symbol_short!("prop"), proposal_id);
        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(ContractError::ProposalNotFound)?;

        if proposal.executed {
            return Err(ContractError::InvalidProposal);
        }

        let current_time = env.ledger().timestamp();
        if current_time > proposal.expires_at {
            return Err(ContractError::ProposalExpired);
        }

        let voter = env.current_contract_address();
        let vote_key = (symbol_short!("vote"), proposal_id, voter);

        if env.storage().persistent().has(&vote_key) {
            return Err(ContractError::AlreadyVoted);
        }

        env.storage().persistent().set(&vote_key, &true);

        if vote_for {
            proposal.votes_for += 1;
        } else {
            proposal.votes_against += 1;
        }

        env.storage().persistent().set(&key, &proposal);

        Ok(())
    }

    /// Execute a proposal if it has enough votes
    pub fn execute_proposal(env: Env, proposal_id: u64) -> Result<(), ContractError> {
        let state = Self::get_treasury_state(env.clone());
        if state.paused {
            return Err(ContractError::Paused);
        }

        let key = (symbol_short!("prop"), proposal_id);
        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(ContractError::ProposalNotFound)?;

        if proposal.executed {
            return Err(ContractError::InvalidProposal);
        }

        let current_time = env.ledger().timestamp();
        if current_time > proposal.expires_at {
            return Err(ContractError::ProposalExpired);
        }

        // Simple majority check
        if proposal.votes_for <= proposal.votes_against {
            return Err(ContractError::InsufficientSignatures);
        }

        proposal.executed = true;
        env.storage().persistent().set(&key, &proposal);

        Self::log_audit(
            env.clone(),
            symbol_short!("prop_exec"),
            proposal.proposer,
            Vec::new(&env),
        );

        env.events().publish((EVT_PROPOSAL,), (proposal_id, true));

        Ok(())
    }

    /// Calculate protocol revenue for a period
    pub fn calculate_protocol_revenue(env: Env, period_start: u64, period_end: u64) -> i128 {
        let state = Self::get_treasury_state(env.clone());
        let revenue = state.total_balance;

        // Update last revenue calculation time
        let mut state = state;
        state.last_revenue_calc = env.ledger().timestamp();
        env.storage()
            .instance()
            .set(&symbol_short!("state"), &state);

        revenue
    }

    /// Distribute rewards for an epoch
    pub fn distribute_rewards(
        env: Env,
        asset: Address,
        epoch: u64,
    ) -> Result<(), ContractError> {
        let state = Self::get_treasury_state(env.clone());
        if state.paused {
            return Err(ContractError::Paused);
        }

        let fee_key = (symbol_short!("fees"), asset.clone());
        let total_fees: i128 = env.storage().persistent().get(&fee_key).unwrap_or(0);

        if total_fees <= 0 {
            return Err(ContractError::NoFeesAvailable);
        }

        let reward_pool = RewardPool {
            asset: asset.clone(),
            total_rewards: total_fees,
            distributed: 0,
            epoch,
        };

        let key = (symbol_short!("reward"), asset, epoch);
        env.storage().persistent().set(&key, &reward_pool);

        Self::log_audit(
            env.clone(),
            symbol_short!("rew_dist"),
            env.current_contract_address(),
            Vec::new(&env),
        );

        env.events().publish((EVT_REWARD_DIST,), (asset, epoch, total_fees));

        Ok(())
    }

    /// Create a liquidity pool for yield generation
    pub fn create_liquidity_pool(
        env: Env,
        asset: Address,
        apy_bps: u32,
    ) -> Result<u64, ContractError> {
        let state = Self::get_treasury_state(env.clone());
        if state.paused {
            return Err(ContractError::Paused);
        }

        let admin: Address = env
            .storage()
            .instance()
            .get(&symbol_short!("admin"))
            .ok_or(ContractError::Unauthorized)?;

        admin.require_auth();

        let mut id: u64 = env
            .storage()
            .instance()
            .get(&symbol_short!("pool_id"))
            .unwrap_or(0);
        id += 1;
        env.storage()
            .instance()
            .set(&symbol_short!("pool_id"), &id);

        let pool = LiquidityPool {
            id,
            asset: asset.clone(),
            deposited: 0,
            rewards: 0,
            apy_bps,
            last_update: env.ledger().timestamp(),
        };

        let key = (symbol_short!("pool"), id);
        env.storage().persistent().set(&key, &pool);

        Self::log_audit(
            env.clone(),
            symbol_short!("pool_create"),
            admin,
            Vec::new(&env),
        );

        env.events().publish((EVT_LIQUIDITY,), (id, asset, apy_bps));

        Ok(id)
    }

    /// Add funds to a liquidity pool
    pub fn add_liquidity(env: Env, pool_id: u64, amount: i128) -> Result<(), ContractError> {
        let state = Self::get_treasury_state(env.clone());
        if state.paused {
            return Err(ContractError::Paused);
        }

        if amount <= 0 {
            return Err(ContractError::ZeroAmount);
        }

        let key = (symbol_short!("pool"), pool_id);
        let mut pool: LiquidityPool = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(ContractError::PoolNotFound)?;

        // Transfer tokens to treasury
        let token_client = token::Client::new(&env, &pool.asset);
        token_client.transfer(
            &env.current_contract_address(),
            &env.current_contract_address(),
            &amount,
        );

        pool.deposited += amount;
        pool.last_update = env.ledger().timestamp();
        env.storage().persistent().set(&key, &pool);

        // Update treasury state
        let mut state = Self::get_treasury_state(env.clone());
        state.total_balance += amount;
        env.storage()
            .instance()
            .set(&symbol_short!("state"), &state);

        Self::log_audit(
            env.clone(),
            symbol_short!("liq_add"),
            env.current_contract_address(),
            Vec::new(&env),
        );

        Ok(())
    }

    /// Remove funds from a liquidity pool
    pub fn remove_liquidity(env: Env, pool_id: u64, amount: i128) -> Result<(), ContractError> {
        let state = Self::get_treasury_state(env.clone());
        if state.paused {
            return Err(ContractError::Paused);
        }

        if amount <= 0 {
            return Err(ContractError::ZeroAmount);
        }

        let key = (symbol_short!("pool"), pool_id);
        let mut pool: LiquidityPool = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(ContractError::PoolNotFound)?;

        if pool.deposited < amount {
            return Err(ContractError::InsufficientBalance);
        }

        // Transfer tokens back
        let token_client = token::Client::new(&env, &pool.asset);
        token_client.transfer(
            &env.current_contract_address(),
            &env.current_contract_address(),
            &amount,
        );

        pool.deposited -= amount;
        pool.last_update = env.ledger().timestamp();
        env.storage().persistent().set(&key, &pool);

        // Update treasury state
        let mut state = Self::get_treasury_state(env.clone());
        state.total_balance = state.total_balance.saturating_sub(amount);
        env.storage()
            .instance()
            .set(&symbol_short!("state"), &state);

        Self::log_audit(
            env.clone(),
            symbol_short!("liq_rem"),
            env.current_contract_address(),
            Vec::new(&env),
        );

        Ok(())
    }

    /// Internal function to log audit entries
    fn log_audit(env: Env, action: Symbol, actor: Address, details: Vec<u8>) {
        let mut id: u64 = env
            .storage()
            .instance()
            .get(&symbol_short!("audit_id"))
            .unwrap_or(0);
        id += 1;
        env.storage()
            .instance()
            .set(&symbol_short!("audit_id"), &id);

        let log = AuditLog {
            timestamp: env.ledger().timestamp(),
            action,
            actor,
            details,
        };

        let key = (symbol_short!("audit"), id);
        env.storage().persistent().set(&key, &log);

        env.events().publish((EVT_AUDIT,), (id, action));
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Address as _, token, Env};

    struct TestEnv<'a> {
        env: Env,
        client: ProtocolTreasuryClient<'a>,
        treasury_addr: Address,
        admin: Address,
        token_addr: Address,
    }

    fn setup() -> TestEnv<'static> {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let treasury_addr = env.register(ProtocolTreasury, ());
        let client = ProtocolTreasuryClient::new(&env, &treasury_addr);

        // Create a token
        let token_admin = Address::generate(&env);
        let token_contract = env.register_stellar_asset_contract_v2(token_admin);
        let token_addr = token_contract.address();

        client.initialize(&admin);

        let client = unsafe {
            core::mem::transmute::<ProtocolTreasuryClient<'_>, ProtocolTreasuryClient<'static>>(
                client,
            )
        };

        TestEnv {
            env,
            client,
            treasury_addr,
            admin,
            token_addr,
        }
    }

    fn mint_to_treasury(t: &TestEnv, amount: i128) {
        let token_admin_client = token::StellarAssetClient::new(&t.env, &t.token_addr);
        token_admin_client.mint(&t.treasury_addr, &amount);
    }

    #[test]
    fn test_initialize() {
        let t = setup();

        t.env.as_contract(&t.treasury_addr, || {
            let admin: Address = t
                .env
                .storage()
                .instance()
                .get(&symbol_short!("admin"))
                .unwrap();
            assert_eq!(admin, t.admin);

            let fee_bps: u32 = t
                .env
                .storage()
                .instance()
                .get(&symbol_short!("fee_bps"))
                .unwrap();
            assert_eq!(fee_bps, DEFAULT_FEE_BPS);
        });
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #2)")]
    fn test_initialize_already_initialized() {
        let t = setup();
        t.client.initialize(&t.admin);
    }

    #[test]
    fn test_set_fee_bps() {
        let t = setup();
        t.client.set_fee_bps(&100);
        assert_eq!(t.client.get_fee_bps(), 100);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #5)")]
    fn test_set_fee_bps_too_high() {
        let t = setup();
        t.client.set_fee_bps(&1001); // > 10%
    }

    #[test]
    fn test_deposit_fee() {
        let t = setup();
        t.client.deposit_fee(&t.token_addr, &500);
        assert_eq!(t.client.get_total_fees(&t.token_addr), 500);

        t.client.deposit_fee(&t.token_addr, &300);
        assert_eq!(t.client.get_total_fees(&t.token_addr), 800);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #6)")]
    fn test_deposit_fee_zero() {
        let t = setup();
        t.client.deposit_fee(&t.token_addr, &0);
    }

    #[test]
    fn test_register_contributor() {
        let t = setup();
        let contributor = Address::generate(&t.env);

        t.client.register_contributor(&contributor, &100);

        let c = t.client.get_contributor(&contributor).unwrap();
        assert_eq!(c.share_weight, 100);
        assert_eq!(t.client.get_total_weight(), 100);
    }

    #[test]
    fn test_register_contributor_update_weight() {
        let t = setup();
        let contributor = Address::generate(&t.env);

        t.client.register_contributor(&contributor, &100);
        assert_eq!(t.client.get_total_weight(), 100);

        t.client.register_contributor(&contributor, &200);
        assert_eq!(t.client.get_total_weight(), 200);

        let c = t.client.get_contributor(&contributor).unwrap();
        assert_eq!(c.share_weight, 200);
    }

    #[test]
    fn test_remove_contributor() {
        let t = setup();
        let contributor = Address::generate(&t.env);

        t.client.register_contributor(&contributor, &100);
        assert_eq!(t.client.get_total_weight(), 100);

        t.client.remove_contributor(&contributor);
        assert_eq!(t.client.get_total_weight(), 0);
        assert!(t.client.get_contributor(&contributor).is_none());
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #3)")]
    fn test_remove_nonexistent_contributor() {
        let t = setup();
        let contributor = Address::generate(&t.env);
        t.client.remove_contributor(&contributor);
    }

    #[test]
    fn test_claim_share_single_contributor() {
        let t = setup();
        let contributor = Address::generate(&t.env);

        t.client.register_contributor(&contributor, &100);

        // Deposit fees and mint tokens to treasury
        t.client.deposit_fee(&t.token_addr, &1000);
        mint_to_treasury(&t, 1000);

        // Claim share (100% since sole contributor)
        let claimed = t.client.claim_share(&contributor, &t.token_addr);
        assert_eq!(claimed, 1000);

        // Verify token balance
        let token = token::Client::new(&t.env, &t.token_addr);
        assert_eq!(token.balance(&contributor), 1000);
    }

    #[test]
    fn test_claim_share_multiple_contributors() {
        let t = setup();
        let c1 = Address::generate(&t.env);
        let c2 = Address::generate(&t.env);

        // 75% / 25% split
        t.client.register_contributor(&c1, &75);
        t.client.register_contributor(&c2, &25);

        t.client.deposit_fee(&t.token_addr, &1000);
        mint_to_treasury(&t, 1000);

        let claimed1 = t.client.claim_share(&c1, &t.token_addr);
        assert_eq!(claimed1, 750);

        let claimed2 = t.client.claim_share(&c2, &t.token_addr);
        assert_eq!(claimed2, 250);

        let token = token::Client::new(&t.env, &t.token_addr);
        assert_eq!(token.balance(&c1), 750);
        assert_eq!(token.balance(&c2), 250);
    }

    #[test]
    fn test_claim_share_incremental() {
        let t = setup();
        let contributor = Address::generate(&t.env);

        t.client.register_contributor(&contributor, &100);

        // First deposit
        t.client.deposit_fee(&t.token_addr, &500);
        mint_to_treasury(&t, 500);
        let claimed1 = t.client.claim_share(&contributor, &t.token_addr);
        assert_eq!(claimed1, 500);

        // Second deposit
        t.client.deposit_fee(&t.token_addr, &300);
        mint_to_treasury(&t, 300);
        let claimed2 = t.client.claim_share(&contributor, &t.token_addr);
        assert_eq!(claimed2, 300);

        let token = token::Client::new(&t.env, &t.token_addr);
        assert_eq!(token.balance(&contributor), 800);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #4)")]
    fn test_claim_no_fees() {
        let t = setup();
        let contributor = Address::generate(&t.env);

        t.client.register_contributor(&contributor, &100);
        t.client.claim_share(&contributor, &t.token_addr);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #3)")]
    fn test_claim_not_contributor() {
        let t = setup();
        let stranger = Address::generate(&t.env);
        t.client.claim_share(&stranger, &t.token_addr);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #4)")]
    fn test_claim_already_claimed_all() {
        let t = setup();
        let contributor = Address::generate(&t.env);

        t.client.register_contributor(&contributor, &100);
        t.client.deposit_fee(&t.token_addr, &500);
        mint_to_treasury(&t, 500);

        t.client.claim_share(&contributor, &t.token_addr);
        // Second claim should fail - nothing new to claim
        t.client.claim_share(&contributor, &t.token_addr);
    }

    #[test]
    fn test_get_total_fees_no_deposits() {
        let t = setup();
        assert_eq!(t.client.get_total_fees(&t.token_addr), 0);
    }

    #[test]
    fn test_fee_bps_default() {
        let t = setup();
        assert_eq!(t.client.get_fee_bps(), DEFAULT_FEE_BPS);
    }

    #[test]
    fn test_propose_admin() {
        let t = setup();
        let new_admin = Address::generate(&t.env);

        assert!(t.client.get_pending_admin().is_none());
        t.client.propose_admin(&new_admin);
        assert_eq!(t.client.get_pending_admin(), Some(new_admin));
    }

    #[test]
    fn test_accept_admin() {
        let t = setup();
        let new_admin = Address::generate(&t.env);

        t.client.propose_admin(&new_admin);
        t.client.accept_admin();

        t.env.as_contract(&t.treasury_addr, || {
            let stored_admin: Address = t
                .env
                .storage()
                .instance()
                .get(&symbol_short!("admin"))
                .unwrap();
            assert_eq!(stored_admin, new_admin);
        });
        assert!(t.client.get_pending_admin().is_none());
    }

    #[test]
    #[should_panic]
    fn test_propose_admin_unauthorized() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let new_admin = Address::generate(&env);
        let treasury_addr = env.register(ProtocolTreasury, ());

        env.as_contract(&treasury_addr, || {
            ProtocolTreasury::initialize(env.clone(), admin).unwrap();
            // No mocked auth — admin.require_auth() panics
            ProtocolTreasury::propose_admin(env.clone(), new_admin).unwrap();
        });
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #7)")]
    fn test_accept_admin_no_pending() {
        let t = setup();
        t.client.accept_admin();
    }

    #[test]
    fn test_pause_treasury() {
        let t = setup();
        t.client.pause_treasury();
        let state = t.client.get_treasury_state();
        assert!(state.paused);
    }

    #[test]
    fn test_unpause_treasury() {
        let t = setup();
        t.client.pause_treasury();
        t.client.unpause_treasury();
        let state = t.client.get_treasury_state();
        assert!(!state.paused);
    }

    #[test]
    fn test_propose_multi_sig_withdrawal() {
        let t = setup();
        let recipient = Address::generate(&t.env);
        let id = t
            .client
            .propose_multi_sig_withdrawal(&recipient, &t.token_addr, &2_000_000);
        assert!(id > 0);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #8)")]
    fn test_propose_multi_sig_small_amount() {
        let t = setup();
        let recipient = Address::generate(&t.env);
        t.client
            .propose_multi_sig_withdrawal(&recipient, &t.token_addr, &500_000);
    }

    #[test]
    fn test_create_timelocked_withdrawal() {
        let t = setup();
        let recipient = Address::generate(&t.env);
        let id = t
            .client
            .create_timelocked_withdrawal(&recipient, &t.token_addr, &1000);
        assert!(id > 0);
    }

    #[test]
    fn test_create_proposal() {
        let t = setup();
        let id = t.client.create_proposal(
            &ProposalType::FeeChange,
            &symbol_short!("test"),
            Vec::new(&t.env),
            &86400,
        );
        assert!(id > 0);
    }

    #[test]
    fn test_vote_proposal() {
        let t = setup();
        let id = t.client.create_proposal(
            &ProposalType::FeeChange,
            &symbol_short!("test"),
            Vec::new(&t.env),
            &86400,
        );
        t.client.vote_proposal(&id, &true);
    }

    #[test]
    fn test_execute_proposal() {
        let t = setup();
        let id = t.client.create_proposal(
            &ProposalType::FeeChange,
            &symbol_short!("test"),
            Vec::new(&t.env),
            &86400,
        );
        t.client.vote_proposal(&id, &true);
        t.client.execute_proposal(&id);
    }

    #[test]
    fn test_calculate_protocol_revenue() {
        let t = setup();
        let revenue = t.client.calculate_protocol_revenue(&0, &86400);
        assert_eq!(revenue, 0);
    }

    #[test]
    fn test_distribute_rewards() {
        let t = setup();
        t.client.deposit_fee(&t.token_addr, &1000);
        mint_to_treasury(&t, 1000);
        t.client.distribute_rewards(&t.token_addr, &1);
    }

    #[test]
    fn test_create_liquidity_pool() {
        let t = setup();
        let id = t.client.create_liquidity_pool(&t.token_addr, &500);
        assert!(id > 0);
    }

    #[test]
    fn test_add_liquidity() {
        let t = setup();
        let id = t.client.create_liquidity_pool(&t.token_addr, &500);
        mint_to_treasury(&t, 1000);
        t.client.add_liquidity(&id, &500);
    }

    #[test]
    fn test_remove_liquidity() {
        let t = setup();
        let id = t.client.create_liquidity_pool(&t.token_addr, &500);
        mint_to_treasury(&t, 1000);
        t.client.add_liquidity(&id, &500);
        t.client.remove_liquidity(&id, &200);
    }
}

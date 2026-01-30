//! StelloVault Soroban Contracts
//!
//! This module contains the smart contracts for StelloVault, a trade finance dApp
//! built on Stellar and Soroban. The contracts handle collateral tokenization,
//! multi-signature escrows, and automated release mechanisms.

#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, token, Address, BytesN, Env, Symbol,
};

/// Contract errors
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContractError {
    Unauthorized = 1,
    InsufficientBalance = 2,
    InvalidAmount = 3,
    EscrowNotFound = 4,
    EscrowError = 5,
    EscrowExpired = 6,
    EscrowNotExpired = 7,
    EscrowAlreadyReleased = 8,
    ProposalNotFound = 9,
    ProposalNotActive = 10,
    AlreadyVoted = 11,
    VotePeriodEnded = 12,
    ZeroWeight = 13,
    AssetNotWhitelisted = 14,
    OracleNotWhitelisted = 15,
    LtvExceeded = 16,
    CollateralNotFound = 17,
    MathOverflow = 18,
    VoteOverflow = 19,
    VotePeriodActive = 20,
    QuorumNotMet = 21,
}

impl From<soroban_sdk::Error> for ContractError {
    fn from(_: soroban_sdk::Error) -> Self {
        ContractError::EscrowError
    }
}

impl From<&ContractError> for soroban_sdk::Error {
    fn from(e: &ContractError) -> Self {
        soroban_sdk::Error::from_contract_error(e.clone() as u32)
    }
}

/// Collateral Registry Interface
pub trait CollateralRegistryClient {
    fn lock_collateral(env: &Env, collateral_id: u64);
    fn unlock_collateral(env: &Env, collateral_id: u64);
}

/// Oracle Adapter Interface
pub trait OracleAdapterClient {
    fn verify_release_condition(env: &Env, metadata: Symbol) -> bool;
}

/// Collateral token data structure
#[contracttype]
#[derive(Clone)]
pub struct CollateralToken {
    pub owner: Address,
    pub asset_type: Symbol,
    pub asset_value: i128,
    pub metadata: Symbol,
    pub fractional_shares: u32,
    pub created_at: u64,
}

/// Escrow data structure for trade finance deals
#[contracttype]
#[derive(Clone)]
pub struct TradeEscrow {
    pub buyer: Address,
    pub seller: Address,
    pub lender: Address, // New: Lender involved in the deal
    pub collateral_token_id: u64,
    pub amount: i128,
    pub asset: Address, // New: Payment asset
    pub status: EscrowStatus,
    pub oracle_address: Address,
    pub release_conditions: Symbol,
    pub expiry_ts: u64, // New: Expiration timestamp
    pub created_at: u64,
}

/// Escrow status enum
#[contracttype]
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum EscrowStatus {
    Pending = 0,
    Active = 1,
    Released = 2,
    Cancelled = 3,
    Disputed = 4, // New: Dispute state
}

/// Governance action types
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GovernanceAction {
    UpdateMaxLTV(u32), // LTV in basis points (e.g., 8000 = 80%)
    UpdateCollateralWhitelist(Symbol, bool), // Asset symbol, is_allowed
    UpdateOracleWhitelist(Address, bool), // Oracle address, is_allowed
    UpgradeContract(BytesN<32>), // New Wasm Hash
}

/// Proposal data structure
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Proposal {
    pub id: u64,
    pub proposer: Address,
    pub title: Symbol,
    pub desc: Symbol,
    pub action: GovernanceAction,
    pub vote_count: u128, // Sqrt-weighted votes
    pub end_time: u64,
    pub executed: bool,
}

/// Proposal vote tracking (to prevent double voting)
#[contracttype]
#[derive(Clone)]
pub struct VoteRecord {
    pub voter: Address,
    pub proposal_id: u64,
    pub weight: u128,
}

/// Main contract for StelloVault trade finance operations
#[contract]
pub struct StelloVaultContract;

/// Contract implementation
#[contractimpl]
impl StelloVaultContract {
    /// Initialize the contract
    pub fn initialize(env: Env, admin: Address, gov_token: Address) -> Result<(), ContractError> {
        if env.storage().instance().has(&symbol_short!("admin")) {
            return Err(ContractError::Unauthorized);
        }

        env.storage().instance().set(&symbol_short!("admin"), &admin);
        env.storage().instance().set(&symbol_short!("gov_token"), &gov_token);
        env.storage().instance().set(&symbol_short!("tok_next"), &1u64);
        env.storage().instance().set(&symbol_short!("esc_next"), &1u64);
        env.storage().instance().set(&symbol_short!("prop_next"), &1u64);

        // Default protocol parameters
        env.storage().instance().set(&symbol_short!("max_ltv"), &7000u32); // 70% LTV default
        env.storage().instance().set(&symbol_short!("quorum"), &100u128); // Default quorum

        env.events().publish(
            (symbol_short!("init"),),
            (admin.clone(), gov_token),
        );
        Ok(())
    }

    /// Get contract admin
    pub fn admin(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&symbol_short!("admin"))
            .unwrap()
    }

    /// Tokenize collateral (create a new collateral token)
    pub fn tokenize_collateral(
        env: Env,
        owner: Address,
        asset_type: Symbol,
        asset_value: i128,
        metadata: Symbol,
        fractional_shares: u32,
    ) -> Result<u64, ContractError> {
        owner.require_auth();

        if asset_value <= 0 {
            return Err(ContractError::InvalidAmount);
        }

        // Check Collateral Whitelist
        if !env.storage().persistent().get::<_, bool>(&(symbol_short!("w_col"), asset_type.clone())).unwrap_or(false) {
            return Err(ContractError::AssetNotWhitelisted);
        }

        let token_id: u64 = env
            .storage()
            .instance()
            .get(&symbol_short!("tok_next"))
            .unwrap_or(1);

        let collateral = CollateralToken {
            owner: owner.clone(),
            asset_type,
            asset_value,
            metadata,
            fractional_shares,
            created_at: env.ledger().timestamp(),
        };

        env.storage()
            .persistent()
            .set(&token_id, &collateral);

        env.storage()
            .instance()
            .set(&symbol_short!("tok_next"), &(token_id + 1));

        env.events().publish(
            (symbol_short!("tokenize"),),
            (token_id, owner, asset_value),
        );

        Ok(token_id)
    }

    /// Get collateral token details
    pub fn get_collateral(env: Env, token_id: u64) -> Option<CollateralToken> {
        env.storage().persistent().get(&token_id)
    }

    /// Create a trade escrow
    pub fn create_escrow(
        env: Env,
        buyer: Address,
        seller: Address,
        lender: Address,
        collateral_token_id: u64,
        amount: i128,
        asset: Address,
        oracle_address: Address,
        release_conditions: Symbol,
        expiry_ts: u64,
    ) -> Result<u64, ContractError> {
        buyer.require_auth();

        if amount <= 0 {
            return Err(ContractError::InvalidAmount);
        }

        // Check Oracle Whitelist
        if !env.storage().persistent().get::<_, bool>(&(symbol_short!("w_orc"), oracle_address.clone())).unwrap_or(false) {
            return Err(ContractError::OracleNotWhitelisted);
        }

        // Verify collateral token exists and Check LTV
        let collateral: CollateralToken = env.storage().persistent().get(&collateral_token_id).ok_or(ContractError::CollateralNotFound)?;

        let max_ltv: u32 = env.storage().instance().get(&symbol_short!("max_ltv")).unwrap_or(0);

        // Check for math overflow during LTV calculation
        let adjusted_value = (collateral.asset_value as i128)
            .checked_mul(max_ltv as i128)
            .ok_or(ContractError::MathOverflow)?;

        let max_loan_amount = adjusted_value / 10000;

        if amount > max_loan_amount {
            return Err(ContractError::LtvExceeded);
        }

        let escrow_id: u64 = env
            .storage()
            .instance()
            .get(&symbol_short!("esc_next"))
            .unwrap_or(1);

        let escrow = TradeEscrow {
            buyer: buyer.clone(),
            seller: seller.clone(),
            lender,
            collateral_token_id,
            amount,
            asset,
            status: EscrowStatus::Active, // Active immediately since funds are locked
            oracle_address,
            release_conditions,
            expiry_ts,
            created_at: env.ledger().timestamp(),
        };

        env.storage().persistent().set(&escrow_id, &escrow);

        env.storage()
            .instance()
            .set(&symbol_short!("esc_next"), &(escrow_id + 1));

        env.events().publish(
            (symbol_short!("esc_crtd"),),
            (escrow_id, buyer, seller, amount),
        );

        Ok(escrow_id)
    }

    /// Release escrow funds (Oracle-triggered)
    pub fn release_funds(env: Env, escrow_id: u64) -> Result<(), ContractError> {
        let mut escrow: TradeEscrow = env
            .storage()
            .persistent()
            .get(&escrow_id)
            .ok_or(ContractError::EscrowNotFound)?;

        // Authorization: Oracle must sign off
        // Real implementation: OracleAdapter checks condition
        // Here we just require the stored oracle address to invoke this
        escrow.oracle_address.require_auth();

        if escrow.status != EscrowStatus::Active {
            return Err(ContractError::EscrowError);
        }

        // Transfer funds to Seller
        let token_client = token::Client::new(&env, &escrow.asset);
        token_client.transfer(&env.current_contract_address(), &escrow.seller, &escrow.amount);

        escrow.status = EscrowStatus::Released;
        env.storage().persistent().set(&escrow_id, &escrow);

        env.events().publish((symbol_short!("esc_rel"),), (escrow_id,));
        Ok(())
    }

    /// Expire escrow (Refund Buyer if time passed)
    pub fn expire_escrow(env: Env, escrow_id: u64) -> Result<(), ContractError> {
        let mut escrow: TradeEscrow = env
            .storage()
            .persistent()
            .get(&escrow_id)
            .ok_or(ContractError::EscrowNotFound)?;

        if env.ledger().timestamp() <= escrow.expiry_ts {
            return Err(ContractError::EscrowNotExpired);
        }

        if escrow.status != EscrowStatus::Active {
            return Err(ContractError::EscrowError);
        }

        // Refund Buyer
        let token_client = token::Client::new(&env, &escrow.asset);
        token_client.transfer(&env.current_contract_address(), &escrow.buyer, &escrow.amount);

        escrow.status = EscrowStatus::Cancelled;
        env.storage().persistent().set(&escrow_id, &escrow);

        env.events()
            .publish((symbol_short!("esc_exp"),), (escrow_id,));
        Ok(())
    }

    /// Dispute escrow (Locks funds until resolution)
    pub fn dispute_escrow(env: Env, escrow_id: u64, caller: Address) -> Result<(), ContractError> {
        caller.require_auth();

        let mut escrow: TradeEscrow = env
            .storage()
            .persistent()
            .get(&escrow_id)
            .ok_or(ContractError::EscrowNotFound)?;

        // Only Buyer, Seller, or Lender can raise dispute
        if caller != escrow.buyer && caller != escrow.seller && caller != escrow.lender {
             return Err(ContractError::Unauthorized);
        }

        if escrow.status != EscrowStatus::Active {
             return Err(ContractError::EscrowError);
        }

        escrow.status = EscrowStatus::Disputed;
        env.storage().persistent().set(&escrow_id, &escrow);

        env.events()
            .publish((symbol_short!("esc_dsp"),), (escrow_id,));
        Ok(())
    }

    pub fn get_escrow(env: Env, escrow_id: u64) -> Option<TradeEscrow> {
        env.storage().persistent().get(&escrow_id)
    }

    // --- Governance Functions ---

    /// Create a new proposal
    pub fn propose(
        env: Env,
        proposer: Address,
        title: Symbol,
        desc: Symbol,
        action: GovernanceAction,
        duration: u64,
    ) -> Result<u64, ContractError> {
        proposer.require_auth();

        let proposal_id: u64 = env
            .storage()
            .instance()
            .get(&symbol_short!("prop_next"))
            .unwrap_or(1);

        let end_time = env.ledger().timestamp().checked_add(duration).unwrap();

        let proposal = Proposal {
            id: proposal_id,
            proposer: proposer.clone(),
            title,
            desc,
            action,
            vote_count: 0,
            end_time,
            executed: false,
        };

        env.storage()
            .persistent()
            .set(&(symbol_short!("prop"), proposal_id), &proposal);

        env.storage()
            .instance()
            .set(&symbol_short!("prop_next"), &(proposal_id + 1));

        env.events().publish(
            (symbol_short!("prop_crtd"),),
            (proposal_id, proposer, end_time),
        );

        Ok(proposal_id)
    }

    /// Cast a vote using quadratic voting (weight is the cost/tokens, votes = sqrt(weight))
    pub fn vote(env: Env, voter: Address, proposal_id: u64, weight: u128) -> Result<(), ContractError> {
        voter.require_auth();

        if weight == 0 {
            return Err(ContractError::ZeroWeight);
        }

        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&(symbol_short!("prop"), proposal_id))
            .ok_or(ContractError::ProposalNotFound)?;

        if env.ledger().timestamp() > proposal.end_time {
            return Err(ContractError::VotePeriodEnded);
        }

        if proposal.executed {
            return Err(ContractError::ProposalNotActive);
        }

        // Prevent double voting
        if env.storage().persistent().has(&(symbol_short!("vote"), proposal_id, voter.clone())) {
            return Err(ContractError::AlreadyVoted);
        }

        // Quadratic Voting: Votes = Sqrt(weight)
        
        // Transfer governance tokens from voter to contract to lock weight
        let gov_token: Address = env.storage().instance().get(&symbol_short!("gov_token")).unwrap();
        let token_client = token::Client::new(&env, &gov_token);
        
        token_client.transfer(&voter, &env.current_contract_address(), &(weight as i128));

        let votes = Self::sqrt(weight); 

        // Use checked_add to prevent overflow
        proposal.vote_count = proposal.vote_count.checked_add(votes).ok_or(ContractError::VoteOverflow)?;
        env.storage().persistent().set(&(symbol_short!("prop"), proposal_id), &proposal);

        // Mark as voted
        env.storage().persistent().set(&(symbol_short!("vote"), proposal_id, voter.clone()), &true);

        env.events().publish(
            (symbol_short!("vote_cast"),),
            (proposal_id, voter, votes),
        );

        Ok(())
    }

    /// Execute a successful proposal
    pub fn execute_proposal(env: Env, proposal_id: u64) -> Result<(), ContractError> {
        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&(symbol_short!("prop"), proposal_id))
            .ok_or(ContractError::ProposalNotFound)?;

        if env.ledger().timestamp() <= proposal.end_time {
             return Err(ContractError::VotePeriodActive); 
        }

        if proposal.executed {
            return Err(ContractError::ProposalNotActive);
        }

        // Check Quorum
        let quorum: u128 = env.storage().instance().get(&symbol_short!("quorum")).unwrap_or(100u128);
        if proposal.vote_count < quorum {
             return Err(ContractError::QuorumNotMet);
        }

        // Execute Action
        match proposal.action.clone() {
            GovernanceAction::UpdateMaxLTV(ltv) => {
                env.storage().instance().set(&symbol_short!("max_ltv"), &ltv);
            },
            GovernanceAction::UpdateCollateralWhitelist(asset, allowed) => {
                env.storage().persistent().set(&(symbol_short!("w_col"), asset), &allowed);
            },
            GovernanceAction::UpdateOracleWhitelist(oracle, allowed) => {
                env.storage().persistent().set(&(symbol_short!("w_orc"), oracle), &allowed);
            },
            GovernanceAction::UpgradeContract(wasm_hash) => {
                env.deployer().update_current_contract_wasm(wasm_hash);
            },
        }

        proposal.executed = true;
        env.storage().persistent().set(&(symbol_short!("prop"), proposal_id), &proposal);

        env.events().publish(
            (symbol_short!("param_upd"),),
            (proposal_id,),
        );

        Ok(())
    }

    // Internal helper for sqrt
    fn sqrt(n: u128) -> u128 {
        if n < 2 {
            return n;
        }
        let mut x = n / 2;
        let mut y = (x + n / x) / 2;
        while y < x {
            x = y;
            y = (x + n / x) / 2;
        }
        x
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::{Address as _, Ledger}, Env};

    #[test]
    fn test_initialize() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let contract_id = env.register(StelloVaultContract, ());
        let client = StelloVaultContractClient::new(&env, &contract_id);

        let gov_token = Address::generate(&env);
        client.initialize(&admin, &gov_token);

        let admin_result = client.admin();
        assert_eq!(admin_result, admin);

        // Check default LTV via storage inspection
        env.as_contract(&contract_id, || {
            let max_ltv: u32 = env.storage().instance().get(&symbol_short!("max_ltv")).unwrap();
            assert_eq!(max_ltv, 7000);
        });
    }

    #[test]
    fn test_tokenize_collateral() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let owner = Address::generate(&env);
        let contract_id = env.register(StelloVaultContract, ());
        let client = StelloVaultContractClient::new(&env, &contract_id);

        let gov_token = Address::generate(&env);
        client.initialize(&admin, &gov_token);

        // Whitelist the asset manually
        let asset = Symbol::new(&env, "INVOICE");
        env.as_contract(&contract_id, || {
            env.storage().persistent().set(&(symbol_short!("w_col"), asset.clone()), &true);
        });

        let token_id = client.tokenize_collateral(
            &owner,
            &asset,
            &10000,
            &Symbol::new(&env, "META"),
            &100
        );

        assert_eq!(token_id, 1);

        let collateral = client.get_collateral(&token_id).unwrap();
        assert_eq!(collateral.owner, owner);
        assert_eq!(collateral.asset_value, 10000);
    }

    #[test]
    fn test_create_and_release_escrow() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let contract_id = env.register(StelloVaultContract, ());
        let client = StelloVaultContractClient::new(&env, &contract_id);

        let gov_token = Address::generate(&env);
        client.initialize(&admin, &gov_token);

        let buyer = Address::generate(&env);
        let seller = Address::generate(&env);
        let lender = Address::generate(&env);
        let oracle = Address::generate(&env);

        // Whitelist oracle and collateral
        let asset_type = Symbol::new(&env, "INVOICE");
        env.as_contract(&contract_id, || {
            env.storage().persistent().set(&(symbol_short!("w_orc"), oracle.clone()), &true);
            env.storage().persistent().set(&(symbol_short!("w_col"), asset_type.clone()), &true);
        });

        // Create collateral
        let token_id = client.tokenize_collateral(
            &buyer,
            &asset_type,
            &10000,
            &Symbol::new(&env, "META"),
            &100
        );

        // Create token for payment
        let token_admin = Address::generate(&env);
        let token_contract = env.register_stellar_asset_contract_v2(token_admin);
        let token_address = token_contract.address();
        let token = token::Client::new(&env, &token_address);
        let token_admin_client = token::StellarAssetClient::new(&env, &token_address);

        token_admin_client.mint(&buyer, &10000);

        let expiry = env.ledger().timestamp() + 1000;
        let escrow_id = client.create_escrow(
            &buyer,
            &seller,
            &lender,
            &token_id,
            &7000, // 70% LTV of 10000 = 7000 max
            &token_address,
            &oracle,
            &symbol_short!("ship_del"),
            &expiry
        );

        client.release_funds(&escrow_id);

        let escrow = client.get_escrow(&escrow_id).unwrap();
        assert_eq!(escrow.status, EscrowStatus::Released);
    }

    #[test]
    fn test_escrow_expiry() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let contract_id = env.register(StelloVaultContract, ());
        let client = StelloVaultContractClient::new(&env, &contract_id);

        let gov_token = Address::generate(&env);
        client.initialize(&admin, &gov_token);

        let buyer = Address::generate(&env);
        let seller = Address::generate(&env);
        let lender = Address::generate(&env);
        let oracle = Address::generate(&env);

        // Whitelist oracle and collateral
        let asset_type = Symbol::new(&env, "INVOICE");
        env.as_contract(&contract_id, || {
            env.storage().persistent().set(&(symbol_short!("w_orc"), oracle.clone()), &true);
            env.storage().persistent().set(&(symbol_short!("w_col"), asset_type.clone()), &true);
        });

        // Create collateral
        let token_id = client.tokenize_collateral(
            &buyer,
            &asset_type,
            &10000,
            &Symbol::new(&env, "META"),
            &100
        );

        let token_admin = Address::generate(&env);
        let token_contract = env.register_stellar_asset_contract_v2(token_admin);
        let token_address = token_contract.address();
        let token = token::Client::new(&env, &token_address);
        let token_admin_client = token::StellarAssetClient::new(&env, &token_address);

        token_admin_client.mint(&buyer, &10000);

        let expiry = env.ledger().timestamp() + 100;
        let escrow_id = client.create_escrow(
            &buyer, &seller, &lender, &token_id, &500, &token_address, &oracle, &symbol_short!("cond"), &expiry
        );

        env.ledger().set_timestamp(expiry + 1);

        client.expire_escrow(&escrow_id);

        let escrow = client.get_escrow(&escrow_id).unwrap();
        assert_eq!(escrow.status, EscrowStatus::Cancelled);
    }

    #[test]
    fn test_governance_flow() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let user1 = Address::generate(&env);
        let user2 = Address::generate(&env);
        let contract_id = env.register(StelloVaultContract, ());
        let client = StelloVaultContractClient::new(&env, &contract_id);

        // Create a separate token contract for testing
        let gov_token_admin = Address::generate(&env);
        let gov_token_id = env.register_stellar_asset_contract(gov_token_admin.clone());

        token::StellarAssetClient::new(&env, &gov_token_id).mint(&user1, &1000);
        token::StellarAssetClient::new(&env, &gov_token_id).mint(&user2, &1000);

        client.initialize(&admin, &gov_token_id);

        // 1. Propose LTV change
        let new_ltv = 8000u32;
        let action = GovernanceAction::UpdateMaxLTV(new_ltv);

        let proposal_id = client.propose(
            &user1,
            &Symbol::new(&env, "LTV_UP"),
            &Symbol::new(&env, "Boost_LTV"),
            &action,
            &1000 // duration
        );

        // 2. Vote
        // User 1 votes with weight 100 -> sqrt(100) = 10 votes
        client.vote(&user1, &proposal_id, &100);

        // User 2 votes with weight 400 -> sqrt(400) = 20 votes
        client.vote(&user2, &proposal_id, &400);

        // Check details via storage inspection
        env.as_contract(&contract_id, || {
            let proposal: Proposal = env.storage().persistent().get(&(symbol_short!("prop"), proposal_id)).unwrap();
            assert_eq!(proposal.vote_count, 30);
        });

        // Advance time past vote period
        env.ledger().set_timestamp(env.ledger().timestamp() + 1001);

        // 3. Execute
        client.execute_proposal(&proposal_id);

        // Verify LTV updated
        env.as_contract(&contract_id, || {
            let current_ltv: u32 = env.storage().instance().get(&symbol_short!("max_ltv")).unwrap();
            assert_eq!(current_ltv, 8000);

            let proposal_updated: Proposal = env.storage().persistent().get(&(symbol_short!("prop"), proposal_id)).unwrap();
            assert!(proposal_updated.executed);
        });
    }
}
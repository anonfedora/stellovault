//! StelloVault Soroban Contracts
//!
//! This module contains the smart contracts for StelloVault, a trade finance dApp
//! built on Stellar and Soroban. The contracts handle collateral tokenization,
//! multi-signature escrows, and automated release mechanisms.

#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, token, Address, Env, Symbol,
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

/// Main contract for StelloVault trade finance operations
#[contract]
pub struct StelloVaultContract;

/// Contract implementation
#[contractimpl]
impl StelloVaultContract {
    /// Initialize the contract with external dependencies
    pub fn initialize(
        env: Env,
        admin: Address,
        collateral_registry: Address,
        oracle_adapter: Address,
    ) -> Result<(), ContractError> {
        if env.storage().instance().has(&symbol_short!("admin")) {
            return Err(ContractError::Unauthorized);
        }

        env.storage().instance().set(&symbol_short!("admin"), &admin);
        env.storage()
            .instance()
            .set(&symbol_short!("col_reg"), &collateral_registry);
        env.storage()
            .instance()
            .set(&symbol_short!("orc_adt"), &oracle_adapter);
        env.storage().instance().set(&symbol_short!("esc_next"), &1u64);

        env.events().publish(
            (symbol_short!("init"),),
            (admin, collateral_registry, oracle_adapter),
        );
        Ok(())
    }

    /// Create a trade escrow (Atomic: Transfer + Lock)
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

        // Transfer funds from Buyer to Contract
        let token_client = token::Client::new(&env, &asset);
        token_client.transfer(&buyer, &env.current_contract_address(), &amount);

        // Lock collateral via External Registry (Mock call logic for now)
        // In a real scenario, we'd call the contract:
        // let registry_addr: Address = env.storage().instance().get(&symbol_short!("col_reg")).unwrap();
        // client::CollateralRegistryClient::new(&env, &registry_addr).lock_collateral(&collateral_token_id);

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
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::{Address as _, Ledger}, Env};

    #[test]
    fn test_create_and_release_escrow() {
        let env = Env::default();
        env.mock_all_auths();
        
        let admin = Address::generate(&env);
        let registry = Address::generate(&env);
        let oracle = Address::generate(&env);
        
        let contract_id = env.register(StelloVaultContract, ());
        let client = StelloVaultContractClient::new(&env, &contract_id);
        
        client.initialize(&admin, &registry, &oracle);
        
        let buyer = Address::generate(&env);
        let seller = Address::generate(&env);
        let lender = Address::generate(&env);
        
        let token_admin = Address::generate(&env);
        let token_contract = env.register_stellar_asset_contract_v2(token_admin);
        let token_address = token_contract.address();
        let token = token::Client::new(&env, &token_address);
        let token_admin_client = token::StellarAssetClient::new(&env, &token_address);
        
        token_admin_client.mint(&buyer, &1000);
        
        let expiry = env.ledger().timestamp() + 1000;
        let escrow_id = client.create_escrow(
            &buyer, 
            &seller, 
            &lender, 
            &1, 
            &500, 
            &token_address,
            &oracle,
            &symbol_short!("ship_del"),
            &expiry
        );
        
        assert_eq!(token.balance(&buyer), 500);
        assert_eq!(token.balance(&contract_id), 500);
        
        client.release_funds(&escrow_id);
        
        assert_eq!(token.balance(&contract_id), 0);
        assert_eq!(token.balance(&seller), 500);
    }
    
    #[test]
    fn test_escrow_expiry() {
        let env = Env::default();
        env.mock_all_auths();
        
        let admin = Address::generate(&env);
        let registry = Address::generate(&env);
        let oracle = Address::generate(&env);
        
        let contract_id = env.register(StelloVaultContract, ());
        let client = StelloVaultContractClient::new(&env, &contract_id);
        client.initialize(&admin, &registry, &oracle);
        
        let buyer = Address::generate(&env);
        let seller = Address::generate(&env);
        let lender = Address::generate(&env);
        
        let token_admin = Address::generate(&env);
        let token_contract = env.register_stellar_asset_contract_v2(token_admin);
        let token_address = token_contract.address();
        let token = token::Client::new(&env, &token_address);
        let token_admin_client = token::StellarAssetClient::new(&env, &token_address);
        
        token_admin_client.mint(&buyer, &1000);
        
        let expiry = env.ledger().timestamp() + 100;
        let escrow_id = client.create_escrow(
            &buyer, &seller, &lender, &1, &500, &token_contract.address(), &oracle, &symbol_short!("cond"), &expiry
        );
        
        env.ledger().set_timestamp(expiry + 1);
        
        client.expire_escrow(&escrow_id);
        
        assert_eq!(token.balance(&buyer), 1000);
    }
}
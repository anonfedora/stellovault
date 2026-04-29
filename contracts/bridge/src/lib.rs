#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol, Vec, Bytes};

// Public error type for the bridge contract. Minimal for skeleton purposes.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BridgeError {
    Unauthorized = 1,
    NotInitialized = 2,
    AlreadyInitialized = 3,
    InvalidParams = 4,
    InsufficientLiquidity = 5,
}

impl From<BridgeError> for soroban_sdk::Error {
    fn from(err: BridgeError) -> Self {
        soroban_sdk::Error::from_contract_error(err as u32)
    }
}

impl From<soroban_sdk::Error> for BridgeError {
    fn from(_e: soroban_sdk::Error) -> Self {
        BridgeError::Unauthorized
    }
}

// Support converting a reference to BridgeError as an error in contract calls
impl From<&BridgeError> for soroban_sdk::Error {
    fn from(err: &BridgeError) -> Self {
        soroban_sdk::Error::from_contract_error(*err as u32)
    }
}

/// Simple bridge status enum persisted in storage
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BridgeStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

/// Contract storage keys/types
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeConfig {
    pub admin: Address,
    pub initialized: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Config,
}

#[contract]
pub struct BridgeContract;

#[contractimpl]
impl BridgeContract {
    // Initialize bridge with an admin address
    /// Initialize bridge with admin address. Only the signer of `admin` can initialize.
    pub fn initialize(env: Env, admin: Address) -> Result<(), BridgeError> {
        // Authorization: only signer can initialize with the provided admin
        admin.require_auth();

        // Check if already initialized
        if env.storage().instance().has(&DataKey::Config) {
            return Err(BridgeError::AlreadyInitialized);
        }

        let config = BridgeConfig { admin: admin.clone(), initialized: true };
        env.storage().instance().set(&DataKey::Config, &config);
        env.events().publish((Symbol::new(&env, "bridge_init"),), admin);
        Ok(())
    }

    // Bridge assets across chains (skeleton)
    /// Bridge assets across chains (skeleton).
    pub fn bridge_asset(
        env: Env,
        source_chain: Bytes,
        target_chain: Bytes,
        amount: i128,
        recipient: Address,
    ) -> Result<(), BridgeError> {
        let config = Self::get_config(&env)?;
        // Basic param checks
        if amount <= 0 {
            return Err(BridgeError::InvalidParams);
        }
        if !config.initialized {
            return Err(BridgeError::NotInitialized);
        }
        // Authorization: only admin can initiate bridge in this skeleton
        config.admin.require_auth();

        // Emit a bridge request event as a placeholder for actual cross-chain transfer logic
        env.events().publish(
            (Symbol::new(&env, "bridge_asset"),),
            (source_chain, target_chain, amount, recipient),
        );
        Ok(())
    }

    // Mint wrapped asset on destination chain (skeleton)
    /// Mint wrapped asset on destination chain (skeleton).
    pub fn mint_wrapped_asset(
        env: Env,
        original_asset: Bytes,
        amount: i128,
        recipient: Address,
    ) -> Result<(), BridgeError> {
        let config = Self::get_config(&env)?;
        if !config.initialized {
            return Err(BridgeError::NotInitialized);
        }
        if amount <= 0 {
            return Err(BridgeError::InvalidParams);
        }
        config.admin.require_auth();
        env.events().publish((Symbol::new(&env, "mint_wrapped_asset"),), (original_asset, amount, recipient));
        Ok(())
    }

    // Burn wrapped asset on source chain (skeleton)
    /// Burn wrapped asset on source chain (skeleton).
    pub fn burn_wrapped_asset(
        env: Env,
        wrapped_asset: Bytes,
        amount: i128,
    ) -> Result<(), BridgeError> {
        let config = Self::get_config(&env)?;
        if !config.initialized {
            return Err(BridgeError::NotInitialized);
        }
        if amount <= 0 {
            return Err(BridgeError::InvalidParams);
        }
        config.admin.require_auth();
        env.events().publish((Symbol::new(&env, "burn_wrapped_asset"),), (wrapped_asset, amount));
        Ok(())
    }

    // Validate a bridge transaction hash coming from source chain (skeleton)
    /// Validate a bridge transaction hash from the source chain (skeleton).
    pub fn validate_bridge_transaction(env: Env, tx_hash: Bytes, source_chain: Bytes) -> bool {
        // Very lightweight validation placeholder
        !tx_hash.is_empty() && !source_chain.is_empty()
    }

    // Calculate bridge fees based on amount and number of destination chains
    /// Calculate bridge fees based on amount and destination chains (skeleton).
    pub fn calculate_bridge_fees(env: Env, amount: i128, chains: Vec<Bytes>) -> i128 {
        // Simple placeholder: base fee plus a small per-chain increment
        let base_fee: i128 = 100;
        let per_chain = 50i128;
        let chain_count = chains.len() as i128;
        let fee = base_fee + (per_chain * chain_count) + (amount / 1000);
        if fee < 0 { 0 } else { fee }
    }

    // Monitor bridge status by id (skeleton)
    /// Monitor bridge status by id (skeleton).
    pub fn monitor_bridge_status(_env: Env, _bridge_id: u64) -> BridgeStatus {
        // Skeleton implementation: always report Pending until real monitor logic is added
        BridgeStatus::Pending
    }

    // Helpers
    fn get_config(env: &Env) -> Result<BridgeConfig, BridgeError> {
        if !env.storage().instance().has(&DataKey::Config) {
            return Err(BridgeError::NotInitialized);
        }
        let cfg: BridgeConfig = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(BridgeError::NotInitialized)?;
        Ok(cfg)
    }
}

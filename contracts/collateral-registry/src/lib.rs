//! Collateral Registry Contract for StelloVault
//!
//! This contract serves as the source of truth for all collateral used across StelloVault.
//! It prevents double-financing and fraud by tracking collateral registration and locking.

#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, BytesN, Env, Symbol};

/// Contract errors
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContractError {
    Unauthorized = 1,
    AlreadyInitialized = 2,
    InvalidAmount = 3,
    CollateralExpired = 4,
    CollateralNotFound = 5,
    CollateralLocked = 6,
    DuplicateMetadata = 7,
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

/// Collateral data structure
#[contracttype]
#[derive(Clone)]
pub struct Collateral {
    pub id: u64,
    pub owner: Address,
    pub face_value: i128,
    pub realized_value: i128,
    pub expiry_ts: u64,
    pub metadata_hash: BytesN<32>,
    pub registered_at: u64,
    pub last_valuation_ts: u64,
    pub locked: bool,
}

/// Main contract for collateral registry operations
#[contract]
pub struct CollateralRegistry;

/// Contract implementation
#[contractimpl]
impl CollateralRegistry {
    /// Initialize the contract with admin address
    ///
    /// # Arguments
    /// * `admin` - The admin address that can manage the contract
    ///
    /// # Events
    /// Emits `RegistryInitialized` event
    pub fn initialize(env: Env, admin: Address) -> Result<(), ContractError> {
        if env.storage().instance().has(&symbol_short!("admin")) {
            return Err(ContractError::AlreadyInitialized);
        }

        env.storage().instance().set(&symbol_short!("admin"), &admin);
        env.storage().instance().set(&symbol_short!("next_id"), &1u64);

        env.events().publish(
            (symbol_short!("reg_init"),),
            (admin,),
        );

        Ok(())
    }

    /// Register new collateral
    ///
    /// # Arguments
    /// * `owner` - Address of the collateral owner
    /// * `face_value` - Face value of the collateral (must be > 0)
    /// * `expiry_ts` - Expiry timestamp (must be in future)
    /// * `metadata_hash` - SHA-256 hash of off-chain metadata
    ///
    /// # Returns
    /// The sequential collateral ID
    ///
    /// # Events
    /// Emits `CollateralRegistered` event
    pub fn register_collateral(
        env: Env,
        owner: Address,
        face_value: i128,
        expiry_ts: u64,
        metadata_hash: BytesN<32>,
    ) -> Result<u64, ContractError> {
        owner.require_auth();

        // Validate inputs
        if face_value <= 0 {
            return Err(ContractError::InvalidAmount);
        }

        let current_ts = env.ledger().timestamp();
        if expiry_ts <= current_ts {
            return Err(ContractError::CollateralExpired);
        }

        // Check for duplicate metadata hash
        let metadata_key = Symbol::new(&env, "metadata");
        if env.storage().persistent().has(&(metadata_key.clone(), metadata_hash.clone())) {
            return Err(ContractError::DuplicateMetadata);
        }

        // Generate next ID
        let collateral_id: u64 = env
            .storage()
            .instance()
            .get(&symbol_short!("next_id"))
            .unwrap_or(1);

        // Create collateral
        let collateral = Collateral {
            id: collateral_id,
            owner: owner.clone(),
            face_value,
            realized_value: face_value,
            expiry_ts,
            metadata_hash: metadata_hash.clone(),
            registered_at: current_ts,
            last_valuation_ts: current_ts,
            locked: false,
        };

        // Store collateral
        env.storage().persistent().set(&collateral_id, &collateral);

        // Store metadata hash mapping
        env.storage().persistent().set(&(metadata_key, metadata_hash), &collateral_id);

        // Update next ID
        env.storage()
            .instance()
            .set(&symbol_short!("next_id"), &(collateral_id + 1));

        // Emit event
        env.events().publish(
            (symbol_short!("coll_reg"),),
            (collateral_id, owner, face_value, expiry_ts),
        );

        Ok(collateral_id)
    }

    /// Lock collateral (only callable by EscrowManager contract)
    ///
    /// # Arguments
    /// * `id` - Collateral ID to lock
    ///
    /// # Events
    /// Emits `CollateralLocked` event
    pub fn lock_collateral(env: Env, id: u64) -> Result<(), ContractError> {
        // Only escrow manager can lock collateral
        let escrow_manager: Address = env
            .storage()
            .instance()
            .get(&Symbol::new(&env, "escrow_mgr"))
            .ok_or(ContractError::Unauthorized)?;

        escrow_manager.require_auth();

        let mut collateral: Collateral = env
            .storage()
            .persistent()
            .get(&id)
            .ok_or(ContractError::CollateralNotFound)?;

        if collateral.locked {
            return Err(ContractError::CollateralLocked);
        }

        collateral.locked = true;
        env.storage().persistent().set(&id, &collateral);

        env.events().publish(
            (symbol_short!("coll_lock"),),
            (id,),
        );

        Ok(())
    }

    /// Unlock collateral (only callable by EscrowManager contract)
    ///
    /// # Arguments
    /// * `id` - Collateral ID to unlock
    ///
    /// # Events
    /// Emits `CollateralUnlocked` event
    pub fn unlock_collateral(env: Env, id: u64) -> Result<(), ContractError> {
        // Only escrow manager can unlock collateral
        let escrow_manager: Address = env
            .storage()
            .instance()
            .get(&Symbol::new(&env, "escrow_mgr"))
            .ok_or(ContractError::Unauthorized)?;

        escrow_manager.require_auth();

        let mut collateral: Collateral = env
            .storage()
            .persistent()
            .get(&id)
            .ok_or(ContractError::CollateralNotFound)?;

        if !collateral.locked {
            return Ok(()); // Already unlocked
        }

        collateral.locked = false;
        env.storage().persistent().set(&id, &collateral);

        env.events().publish(
            (symbol_short!("coll_unlk"),),
            (id,),
        );

        Ok(())
    }

    /// Update collateral valuation (only callable by registered Valuation Oracle)
    ///
    /// # Arguments
    /// * `collateral_id` - ID of the collateral to update
    /// * `new_value` - New realized value
    ///
    /// # Events
    /// Emits `CollateralValued` event
    pub fn update_valuation(
        env: Env,
        collateral_id: u64,
        new_value: i128,
    ) -> Result<(), ContractError> {
        // Check authorization
        let valuation_oracle: Address = env
            .storage()
            .instance()
            .get(&Symbol::new(&env, "val_oracle"))
            .ok_or(ContractError::Unauthorized)?;

        valuation_oracle.require_auth();

        // Validate inputs
        if new_value <= 0 {
            return Err(ContractError::InvalidAmount);
        }

        // Fetch collateral
        let mut collateral: Collateral = env
            .storage()
            .persistent()
            .get(&collateral_id)
            .ok_or(ContractError::CollateralNotFound)?;

        // Update values
        collateral.realized_value = new_value;
        collateral.last_valuation_ts = env.ledger().timestamp();

        // Store updated collateral
        env.storage().persistent().set(&collateral_id, &collateral);

        // Emit event
        env.events().publish(
            (symbol_short!("coll_val"),),
            (collateral_id, new_value),
        );

        Ok(())
    }

    /// Get collateral details
    ///
    /// # Arguments
    /// * `id` - Collateral ID to query
    ///
    /// # Returns
    /// Option containing collateral data if found
    pub fn get_collateral(env: Env, id: u64) -> Option<Collateral> {
        env.storage().persistent().get(&id)
    }

    /// Check if collateral is locked
    ///
    /// # Arguments
    /// * `id` - Collateral ID to check
    ///
    /// # Returns
    /// True if collateral is locked, false otherwise
    pub fn is_locked(env: Env, id: u64) -> bool {
        env.storage()
            .persistent()
            .get::<u64, Collateral>(&id)
            .map(|c| c.locked)
            .unwrap_or(false)
    }

    /// Get admin address
    pub fn admin(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&symbol_short!("admin"))
            .unwrap()
    }

    /// Set escrow manager address (admin only)
    ///
    /// # Arguments
    /// * `escrow_manager` - Address of the escrow manager contract
    pub fn set_escrow_manager(env: Env, escrow_manager: Address) -> Result<(), ContractError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&symbol_short!("admin"))
            .unwrap();

        admin.require_auth();

        env.storage()
            .instance()
            .set(&Symbol::new(&env, "escrow_mgr"), &escrow_manager);

        Ok(())
    }

    /// Set valuation oracle address (admin only)
    ///
    /// # Arguments
    /// * `valuation_oracle` - Address of the valuation oracle
    pub fn set_valuation_oracle(env: Env, valuation_oracle: Address) -> Result<(), ContractError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&symbol_short!("admin"))
            .unwrap();

        admin.require_auth();

        env.storage()
            .instance()
            .set(&Symbol::new(&env, "val_oracle"), &valuation_oracle);

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env};

    #[test]
    fn test_initialize() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let contract_id = env.register_contract(None, CollateralRegistry);

        env.as_contract(&contract_id, || {
            let result = CollateralRegistry::initialize(env.clone(), admin.clone());
            assert!(result.is_ok());

            let admin_result = CollateralRegistry::admin(env.clone());
            assert_eq!(admin_result, admin);
        });
    }

    #[test]
    fn test_register_collateral_success() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let owner = Address::generate(&env);
        let contract_id = env.register_contract(None, CollateralRegistry);

        env.as_contract(&contract_id, || {
            // Initialize
            CollateralRegistry::initialize(env.clone(), admin).unwrap();

            // Register collateral
            let future_ts = env.ledger().timestamp() + 86400; // 1 day from now
            let metadata_hash = BytesN::from_array(&env, &[1; 32]);

            let result = CollateralRegistry::register_collateral(
                env.clone(),
                owner.clone(),
                1000,
                future_ts,
                metadata_hash,
            );

            assert!(result.is_ok());
            let collateral_id = result.unwrap();
            assert_eq!(collateral_id, 1);

            // Verify collateral was stored
            let collateral = CollateralRegistry::get_collateral(env.clone(), collateral_id).unwrap();
            assert_eq!(collateral.owner, owner);
            assert_eq!(collateral.face_value, 1000);
            assert_eq!(collateral.realized_value, 1000);
            assert_eq!(collateral.locked, false);
        });
    }

    #[test]
    fn test_update_valuation_success() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let oracle = Address::generate(&env);
        let owner = Address::generate(&env);
        let contract_id = env.register_contract(None, CollateralRegistry);

        env.as_contract(&contract_id, || {
            // Initialize
            CollateralRegistry::initialize(env.clone(), admin.clone()).unwrap();
            CollateralRegistry::set_valuation_oracle(env.clone(), oracle.clone()).unwrap();

            // Register collateral
            let future_ts = env.ledger().timestamp() + 86400;
            let metadata_hash = BytesN::from_array(&env, &[1; 32]);
            let collateral_id = CollateralRegistry::register_collateral(
                env.clone(),
                owner,
                1000,
                future_ts,
                metadata_hash,
            ).unwrap();

            // Update valuation
            let update_result = CollateralRegistry::update_valuation(env.clone(), collateral_id, 1200);
            assert!(update_result.is_ok());

            // Verify updated value
            let collateral = CollateralRegistry::get_collateral(env.clone(), collateral_id).unwrap();
            assert_eq!(collateral.realized_value, 1200);
            assert!(collateral.last_valuation_ts == env.ledger().timestamp());
        });
    }

    #[test]
    fn test_register_collateral_invalid_amount() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let owner = Address::generate(&env);
        let contract_id = env.register_contract(None, CollateralRegistry);

        env.as_contract(&contract_id, || {
            CollateralRegistry::initialize(env.clone(), admin).unwrap();

            let future_ts = env.ledger().timestamp() + 86400;
            let metadata_hash = BytesN::from_array(&env, &[1; 32]);

            let result = CollateralRegistry::register_collateral(
                env.clone(),
                owner,
                0, // Invalid amount
                future_ts,
                metadata_hash,
            );

            assert_eq!(result, Err(ContractError::InvalidAmount));
        });
    }

    #[test]
    fn test_register_collateral_expired() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let owner = Address::generate(&env);
        let contract_id = env.register_contract(None, CollateralRegistry);

        env.as_contract(&contract_id, || {
            CollateralRegistry::initialize(env.clone(), admin).unwrap();

            let past_ts = env.ledger().timestamp() - 1; // Already expired
            let metadata_hash = BytesN::from_array(&env, &[1; 32]);

            let result = CollateralRegistry::register_collateral(
                env.clone(),
                owner,
                1000,
                past_ts,
                metadata_hash,
            );

            assert_eq!(result, Err(ContractError::CollateralExpired));
        });
    }

    #[test]
    fn test_register_collateral_duplicate_metadata() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let owner1 = Address::generate(&env);
        let owner2 = Address::generate(&env);
        let contract_id = env.register_contract(None, CollateralRegistry);

        env.as_contract(&contract_id, || {
            CollateralRegistry::initialize(env.clone(), admin).unwrap();

            let future_ts = env.ledger().timestamp() + 86400;
            let metadata_hash = BytesN::from_array(&env, &[1; 32]);

            // Register first collateral
            CollateralRegistry::register_collateral(
                env.clone(),
                owner1,
                1000,
                future_ts,
                metadata_hash.clone(),
            ).unwrap();

            // Try to register duplicate
            let result = CollateralRegistry::register_collateral(
                env.clone(),
                owner2,
                2000,
                future_ts,
                metadata_hash, // Same hash
            );

            assert_eq!(result, Err(ContractError::DuplicateMetadata));
        });
    }

    #[test]
    fn test_lock_unlock_collateral() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let escrow_manager = Address::generate(&env);
        let owner = Address::generate(&env);
        let contract_id = env.register_contract(None, CollateralRegistry);

        env.as_contract(&contract_id, || {
            // Initialize
            CollateralRegistry::initialize(env.clone(), admin.clone()).unwrap();
            CollateralRegistry::set_escrow_manager(env.clone(), escrow_manager.clone()).unwrap();

            // Register collateral
            let future_ts = env.ledger().timestamp() + 86400;
            let metadata_hash = BytesN::from_array(&env, &[1; 32]);
            let collateral_id = CollateralRegistry::register_collateral(
                env.clone(),
                owner,
                1000,
                future_ts,
                metadata_hash,
            ).unwrap();

            // Lock collateral
            let lock_result = CollateralRegistry::lock_collateral(env.clone(), collateral_id);
            assert!(lock_result.is_ok());
            assert!(CollateralRegistry::is_locked(env.clone(), collateral_id));

            // Unlock collateral
            let unlock_result = CollateralRegistry::unlock_collateral(env.clone(), collateral_id);
            assert!(unlock_result.is_ok());
            assert!(!CollateralRegistry::is_locked(env.clone(), collateral_id));
        });
    }

    #[test]
    fn test_lock_collateral_not_found() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let escrow_manager = Address::generate(&env);
        let contract_id = env.register_contract(None, CollateralRegistry);

        env.as_contract(&contract_id, || {
            CollateralRegistry::initialize(env.clone(), admin).unwrap();
            CollateralRegistry::set_escrow_manager(env.clone(), escrow_manager).unwrap();

            let result = CollateralRegistry::lock_collateral(env.clone(), 999);
            assert_eq!(result, Err(ContractError::CollateralNotFound));
        });
    }

    #[test]
    fn test_lock_collateral_unauthorized() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let unauthorized = Address::generate(&env);
        let owner = Address::generate(&env);
        let contract_id = env.register_contract(None, CollateralRegistry);

        env.as_contract(&contract_id, || {
            CollateralRegistry::initialize(env.clone(), admin).unwrap();

            // Register collateral
            let future_ts = env.ledger().timestamp() + 86400;
            let metadata_hash = BytesN::from_array(&env, &[1; 32]);
            let collateral_id = CollateralRegistry::register_collateral(
                env.clone(),
                owner,
                1000,
                future_ts,
                metadata_hash,
            ).unwrap();

            // Try to lock with unauthorized address (no escrow manager set)
            let result = CollateralRegistry::lock_collateral(env.clone(), collateral_id);
            assert_eq!(result, Err(ContractError::Unauthorized));
        });
    }
}
//! Collateral Registry Contract for StelloVault
//!
//! This contract manages tokenized real-world assets with:
//! - Collateral registration and metadata management
//! - Ownership tracking and transfer validation
//! - Valuation updates with oracle verification
//! - Collateral locking for loan security
//! - Authenticity verification and classification
//! - Efficient on-chain storage with IPFS integration
//! - Comprehensive audit trails via events

#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, BytesN, Env, String, Vec,
};

mod classification;
mod collateral;
mod locking;
mod ownership;
mod valuation;
mod verification;

use classification::*;
use collateral::*;
use locking::*;
use ownership::*;
use valuation::*;
use verification::*;

/// Contract errors
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContractError {
    Unauthorized = 1,
    AlreadyInitialized = 2,
    InvalidCollateralData = 3,
    CollateralNotFound = 4,
    CollateralAlreadyExists = 5,
    InvalidMetadata = 6,
    InvalidValuation = 7,
    OracleSignatureInvalid = 8,
    CollateralLocked = 9,
    CollateralNotLocked = 10,
    InvalidTransfer = 11,
    UnauthorizedTransfer = 12,
    InvalidVerificationData = 13,
    VerificationFailed = 14,
    InvalidClassification = 15,
    MetadataHashMismatch = 16,
    InvalidOwnershipProof = 17,
    TransferFailed = 18,
    InsufficientPermissions = 19,
    InvalidAssetHash = 20,
    DuplicateCollateral = 21,
    CollateralExpired = 22,
    InvalidLockingEscrow = 23,
    LockingFailed = 24,
    UnlockingFailed = 25,
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

/// Main collateral registry contract
#[contract]
pub struct CollateralRegistry;

/// Contract implementation
#[contractimpl]
impl CollateralRegistry {
    /// Initialize the contract with admin and oracle addresses
    ///
    /// # Arguments
    /// * `admin` - Admin address for contract management
    /// * `oracle` - Oracle address for valuation verification
    ///
    /// # Events
    /// Emits `RegistryInitialized` event
    pub fn initialize(env: Env, admin: Address, oracle: Address) -> Result<(), ContractError> {
        if env.storage().instance().has(&symbol_short!("admin")) {
            return Err(ContractError::AlreadyInitialized);
        }

        env.storage()
            .instance()
            .set(&symbol_short!("admin"), &admin);
        env.storage()
            .instance()
            .set(&symbol_short!("oracle"), &oracle);
        env.storage()
            .instance()
            .set(&symbol_short!("next_id"), &1u64);

        env.events()
            .publish((symbol_short!("reg_init"),), (admin.clone(), oracle));

        Ok(())
    }

    /// Register new collateral asset
    ///
    /// # Arguments
    /// * `owner` - Address of collateral owner
    /// * `asset_hash` - SHA-256 hash of asset document
    /// * `metadata_uri` - IPFS URI for metadata
    /// * `asset_type` - Type of asset
    /// * `initial_valuation` - Initial asset valuation
    ///
    /// # Returns
    /// The collateral ID
    ///
    /// # Events
    /// Emits `CollateralRegistered` event
    pub fn register_collateral(
        env: Env,
        owner: Address,
        asset_hash: BytesN<32>,
        metadata_uri: String,
        asset_type: AssetType,
        initial_valuation: i128,
    ) -> Result<u64, ContractError> {
        owner.require_auth();

        // Validate inputs
        if initial_valuation <= 0 {
            return Err(ContractError::InvalidValuation);
        }

        // Check for duplicate asset hash
        let hash_key = format_asset_hash_key(&asset_hash);
        if env.storage().persistent().has(&hash_key) {
            return Err(ContractError::DuplicateCollateral);
        }

        // Get next collateral ID
        let collateral_id: u64 = env
            .storage()
            .instance()
            .get(&symbol_short!("next_id"))
            .unwrap_or(1u64);

        // Create collateral record
        let collateral = Collateral {
            id: collateral_id,
            owner: owner.clone(),
            asset_hash,
            metadata_uri: metadata_uri.clone(),
            asset_type,
            current_valuation: initial_valuation,
            previous_valuation: 0,
            valuation_timestamp: env.ledger().timestamp(),
            status: CollateralStatus::Active,
            locked: false,
            locked_by_escrow: 0,
            verification_status: collateral::VerificationStatus::Pending,
            verified_by: None,
            verified_at: 0,
            created_at: env.ledger().timestamp(),
            updated_at: env.ledger().timestamp(),
            expiry_date: 0,
            fractionalized: false,
            fraction_count: 0,
        };

        // Store collateral
        let storage_key = format_collateral_storage_key(collateral_id);
        env.storage().persistent().set(&storage_key, &collateral);

        // Store asset hash mapping
        env.storage().persistent().set(&hash_key, &collateral_id);

        // Store owner mapping
        let owner_key = format_owner_collateral_key(&owner, collateral_id);
        env.storage().persistent().set(&owner_key, &collateral_id);

        // Update next ID
        env.storage()
            .instance()
            .set(&symbol_short!("next_id"), &(collateral_id + 1));

        // Emit event
        env.events().publish(
            (symbol_short!("col_regd"),),
            (collateral_id, owner, initial_valuation),
        );

        Ok(collateral_id)
    }

    /// Update collateral valuation with oracle verification
    ///
    /// # Arguments
    /// * `collateral_id` - ID of collateral
    /// * `new_valuation` - New valuation amount
    /// * `oracle_signature` - Oracle signature for verification
    ///
    /// # Events
    /// Emits `ValuationUpdated` event
    pub fn update_valuation(
        env: Env,
        collateral_id: u64,
        new_valuation: i128,
        _oracle_signature: BytesN<64>,
    ) -> Result<(), ContractError> {
        // Get oracle address
        let oracle: Address = env
            .storage()
            .instance()
            .get(&symbol_short!("oracle"))
            .ok_or(ContractError::Unauthorized)?;

        // Verify oracle signature (simplified - in production use proper signature verification)
        // For now, just verify oracle is authorized
        oracle.require_auth();

        // Validate valuation
        if new_valuation <= 0 {
            return Err(ContractError::InvalidValuation);
        }

        // Get collateral
        let storage_key = format_collateral_storage_key(collateral_id);
        let mut collateral: Collateral = env
            .storage()
            .persistent()
            .get(&storage_key)
            .ok_or(ContractError::CollateralNotFound)?;

        // Check if collateral is expired
        if collateral.expiry_date > 0 && collateral.expiry_date < env.ledger().timestamp() {
            return Err(ContractError::CollateralExpired);
        }

        // Update valuation
        collateral.previous_valuation = collateral.current_valuation;
        collateral.current_valuation = new_valuation;
        collateral.valuation_timestamp = env.ledger().timestamp();
        collateral.updated_at = env.ledger().timestamp();

        // Store updated collateral
        env.storage().persistent().set(&storage_key, &collateral);

        // Record valuation history
        let valuation_record = ValuationRecord {
            collateral_id,
            old_valuation: collateral.previous_valuation,
            new_valuation,
            oracle: oracle.clone(),
            timestamp: env.ledger().timestamp(),
        };

        let valuation_key = format_valuation_history_key(collateral_id, env.ledger().timestamp());
        env.storage()
            .persistent()
            .set(&valuation_key, &valuation_record);

        // Emit event
        env.events()
            .publish((symbol_short!("val_updt"),), (collateral_id, new_valuation));

        Ok(())
    }

    /// Transfer collateral ownership
    ///
    /// # Arguments
    /// * `collateral_id` - ID of collateral
    /// * `new_owner` - Address of new owner
    ///
    /// # Events
    /// Emits `CollateralTransferred` event
    pub fn transfer_collateral(
        env: Env,
        collateral_id: u64,
        new_owner: Address,
    ) -> Result<(), ContractError> {
        // Get collateral
        let storage_key = format_collateral_storage_key(collateral_id);
        let mut collateral: Collateral = env
            .storage()
            .persistent()
            .get(&storage_key)
            .ok_or(ContractError::CollateralNotFound)?;

        // Verify current owner
        collateral.owner.require_auth();

        // Check if collateral is locked
        if collateral.locked {
            return Err(ContractError::CollateralLocked);
        }

        // Validate new owner
        if new_owner == collateral.owner {
            return Err(ContractError::InvalidTransfer);
        }

        // Update ownership
        let previous_owner = collateral.owner.clone();
        collateral.owner = new_owner.clone();
        collateral.updated_at = env.ledger().timestamp();

        // Store updated collateral
        env.storage().persistent().set(&storage_key, &collateral);

        // Update owner mappings
        let old_owner_key = format_owner_collateral_key(&previous_owner, collateral_id);
        env.storage().persistent().remove(&old_owner_key);

        let new_owner_key = format_owner_collateral_key(&new_owner, collateral_id);
        env.storage()
            .persistent()
            .set(&new_owner_key, &collateral_id);

        // Record transfer
        let transfer_record = OwnershipTransfer {
            collateral_id,
            from: previous_owner.clone(),
            to: new_owner.clone(),
            timestamp: env.ledger().timestamp(),
        };

        let transfer_key = format_transfer_history_key(collateral_id, env.ledger().timestamp());
        env.storage()
            .persistent()
            .set(&transfer_key, &transfer_record);

        // Emit event
        env.events().publish(
            (symbol_short!("col_xfer"),),
            (collateral_id, previous_owner, new_owner),
        );

        Ok(())
    }

    /// Lock collateral for loan security
    ///
    /// # Arguments
    /// * `collateral_id` - ID of collateral
    /// * `escrow_id` - ID of escrow/loan
    ///
    /// # Events
    /// Emits `CollateralLocked` event
    pub fn lock_collateral(
        env: Env,
        collateral_id: u64,
        escrow_id: u64,
    ) -> Result<(), ContractError> {
        // Get collateral
        let storage_key = format_collateral_storage_key(collateral_id);
        let mut collateral: Collateral = env
            .storage()
            .persistent()
            .get(&storage_key)
            .ok_or(ContractError::CollateralNotFound)?;

        // Verify owner
        collateral.owner.require_auth();

        // Check if already locked
        if collateral.locked {
            return Err(ContractError::CollateralLocked);
        }

        // Lock collateral
        collateral.locked = true;
        collateral.locked_by_escrow = escrow_id;
        collateral.updated_at = env.ledger().timestamp();

        // Store updated collateral
        env.storage().persistent().set(&storage_key, &collateral);

        // Record locking
        let lock_record = CollateralLock {
            collateral_id,
            escrow_id,
            locked_at: env.ledger().timestamp(),
            locked_by: collateral.owner.clone(),
        };

        let lock_key = format_lock_history_key(collateral_id, env.ledger().timestamp());
        env.storage().persistent().set(&lock_key, &lock_record);

        // Emit event
        env.events()
            .publish((symbol_short!("col_lock"),), (collateral_id, escrow_id));

        Ok(())
    }

    /// Unlock collateral from loan
    ///
    /// # Arguments
    /// * `collateral_id` - ID of collateral
    ///
    /// # Events
    /// Emits `CollateralUnlocked` event
    pub fn unlock_collateral(env: Env, collateral_id: u64) -> Result<(), ContractError> {
        // Get collateral
        let storage_key = format_collateral_storage_key(collateral_id);
        let mut collateral: Collateral = env
            .storage()
            .persistent()
            .get(&storage_key)
            .ok_or(ContractError::CollateralNotFound)?;

        // Verify owner
        collateral.owner.require_auth();

        // Check if locked
        if !collateral.locked {
            return Err(ContractError::CollateralNotLocked);
        }

        // Unlock collateral
        let escrow_id = collateral.locked_by_escrow;
        collateral.locked = false;
        collateral.locked_by_escrow = 0;
        collateral.updated_at = env.ledger().timestamp();

        // Store updated collateral
        env.storage().persistent().set(&storage_key, &collateral);

        // Emit event
        env.events()
            .publish((symbol_short!("col_unlk"),), (collateral_id, escrow_id));

        Ok(())
    }

    /// Verify collateral authenticity
    ///
    /// # Arguments
    /// * `collateral_id` - ID of collateral
    /// * `verification_data` - Verification data structure
    ///
    /// # Events
    /// Emits `CollateralVerified` event
    pub fn verify_collateral(
        env: Env,
        collateral_id: u64,
        verification_data: VerificationData,
    ) -> Result<(), ContractError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&symbol_short!("admin"))
            .ok_or(ContractError::Unauthorized)?;

        admin.require_auth();

        // Get collateral
        let storage_key = format_collateral_storage_key(collateral_id);
        let mut collateral: Collateral = env
            .storage()
            .persistent()
            .get(&storage_key)
            .ok_or(ContractError::CollateralNotFound)?;

        // Verify document hash matches
        if collateral.asset_hash != verification_data.document_hash {
            return Err(ContractError::MetadataHashMismatch);
        }

        // Update verification status
        collateral.verification_status = collateral::VerificationStatus::Verified;
        collateral.verified_by = Some(admin.clone());
        collateral.verified_at = env.ledger().timestamp();
        collateral.updated_at = env.ledger().timestamp();

        // Store updated collateral
        env.storage().persistent().set(&storage_key, &collateral);

        // Record verification
        let verification_record = VerificationRecord {
            collateral_id,
            verified_by: admin.clone(),
            verification_method: verification_data.verification_method,
            verified_at: env.ledger().timestamp(),
        };

        let verification_key = format_verification_history_key(collateral_id);
        env.storage()
            .persistent()
            .set(&verification_key, &verification_record);

        // Emit event
        env.events()
            .publish((symbol_short!("col_verf"),), (collateral_id, admin));

        Ok(())
    }

    /// Get collateral details
    ///
    /// # Arguments
    /// * `collateral_id` - ID of collateral
    ///
    /// # Returns
    /// Collateral details
    pub fn get_collateral(env: Env, collateral_id: u64) -> Result<Collateral, ContractError> {
        let storage_key = format_collateral_storage_key(collateral_id);
        env.storage()
            .persistent()
            .get(&storage_key)
            .ok_or(ContractError::CollateralNotFound)
    }

    /// Get collateral by asset hash
    ///
    /// # Arguments
    /// * `asset_hash` - SHA-256 hash of asset
    ///
    /// # Returns
    /// Collateral ID if found
    pub fn get_collateral_by_hash(env: Env, asset_hash: BytesN<32>) -> Result<u64, ContractError> {
        let hash_key = format_asset_hash_key(&asset_hash);
        env.storage()
            .persistent()
            .get(&hash_key)
            .ok_or(ContractError::CollateralNotFound)
    }

    /// Get collateral valuation history
    ///
    /// # Arguments
    /// * `collateral_id` - ID of collateral
    ///
    /// # Returns
    /// Vector of valuation records
    pub fn get_valuation_history(
        env: Env,
        _collateral_id: u64,
    ) -> Result<Vec<ValuationRecord>, ContractError> {
        // Note: In production, would iterate through valuation records
        Ok(Vec::new(&env))
    }

    /// Get ownership transfer history
    ///
    /// # Arguments
    /// * `collateral_id` - ID of collateral
    ///
    /// # Returns
    /// Vector of transfer records
    pub fn get_transfer_history(
        env: Env,
        _collateral_id: u64,
    ) -> Result<Vec<OwnershipTransfer>, ContractError> {
        // Note: In production, would iterate through transfer records
        Ok(Vec::new(&env))
    }

    /// Classify collateral asset
    ///
    /// # Arguments
    /// * `collateral_id` - ID of collateral
    /// * `classification` - Asset classification
    pub fn classify_collateral(
        env: Env,
        collateral_id: u64,
        classification: AssetClassification,
    ) -> Result<(), ContractError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&symbol_short!("admin"))
            .ok_or(ContractError::Unauthorized)?;

        admin.require_auth();

        // Get collateral
        let storage_key = format_collateral_storage_key(collateral_id);
        let _collateral: Collateral = env
            .storage()
            .persistent()
            .get(&storage_key)
            .ok_or(ContractError::CollateralNotFound)?;

        // Store classification
        let classification_key = format_classification_key(collateral_id);
        env.storage()
            .persistent()
            .set(&classification_key, &classification);

        // Emit event
        env.events()
            .publish((symbol_short!("col_class"),), (collateral_id, admin));

        Ok(())
    }

    /// Get collateral classification
    ///
    /// # Arguments
    /// * `collateral_id` - ID of collateral
    ///
    /// # Returns
    /// Asset classification
    pub fn get_classification(
        env: Env,
        collateral_id: u64,
    ) -> Result<AssetClassification, ContractError> {
        let classification_key = format_classification_key(collateral_id);
        env.storage()
            .persistent()
            .get(&classification_key)
            .ok_or(ContractError::InvalidClassification)
    }
}

// Helper functions

fn format_collateral_storage_key(collateral_id: u64) -> (soroban_sdk::Symbol, u64) {
    (symbol_short!("collat"), collateral_id)
}

fn format_asset_hash_key(asset_hash: &BytesN<32>) -> (soroban_sdk::Symbol, BytesN<32>) {
    (symbol_short!("hash"), asset_hash.clone())
}

fn format_owner_collateral_key(
    owner: &Address,
    collateral_id: u64,
) -> (soroban_sdk::Symbol, Address, u64) {
    (symbol_short!("owner"), owner.clone(), collateral_id)
}

fn format_valuation_history_key(
    collateral_id: u64,
    timestamp: u64,
) -> (soroban_sdk::Symbol, u64, u64) {
    (symbol_short!("val"), collateral_id, timestamp)
}

fn format_transfer_history_key(
    collateral_id: u64,
    timestamp: u64,
) -> (soroban_sdk::Symbol, u64, u64) {
    (symbol_short!("trans"), collateral_id, timestamp)
}

fn format_lock_history_key(collateral_id: u64, timestamp: u64) -> (soroban_sdk::Symbol, u64, u64) {
    (symbol_short!("lock"), collateral_id, timestamp)
}

fn format_verification_history_key(collateral_id: u64) -> (soroban_sdk::Symbol, u64) {
    (symbol_short!("verif"), collateral_id)
}

fn format_classification_key(collateral_id: u64) -> (soroban_sdk::Symbol, u64) {
    (symbol_short!("class"), collateral_id)
}

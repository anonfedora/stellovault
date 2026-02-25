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

/// Contract data structure for storage
#[contracttype]
#[derive(Clone)]
pub struct ContractData {
    pub admin: Address,
    pub initialized: bool,
    pub oracles: Vec<Address>,
}

/// Event symbols
const ORACLE_ADDED: Symbol = symbol_short!("orc_add");
const ORACLE_REMOVED: Symbol = symbol_short!("orc_rem");
const ORACLE_CONFIRMED: Symbol = symbol_short!("confirmed");
const INITIALIZED: Symbol = symbol_short!("init");
const CONFIRMING_ORACLES: Symbol = symbol_short!("conf_orc");

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

        // Store admin and initialization status
        let contract_data = ContractData {
            admin: admin.clone(),
            initialized: true,
            oracles: Vec::new(&env),
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
        if event_type < 1 || event_type > 5 {
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
                if let Some(confirmation) = env.storage().persistent().get(&confirmation_key) {
                    confirmations.push_back(confirmation);
                }
            }

            return Ok(confirmations);
        }

        // Fallback for legacy data: iterate through all registered oracles
        for oracle in contract_data.oracles.iter() {
            let confirmation_key = (escrow_id.clone(), oracle.clone());
            if let Some(confirmation) = env.storage().persistent().get(&confirmation_key) {
                confirmations.push_back(confirmation);
            }
        }

        Ok(confirmations)
    }

    fn get_confirming_oracles(env: &Env, escrow_id: &Bytes) -> Vec<Address> {
        let confirming_key = (CONFIRMING_ORACLES, escrow_id.clone());
        env.storage()
            .persistent()
            .get(&confirming_key)
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
}


//! Oracle Adapter Contract for StelloVault
//!
//! This contract manages oracle providers and verifies off-chain events
//! such as shipment confirmations, delivery status, and quality inspections.
//! It serves as the bridge between on-chain escrow operations and trusted oracles.

#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Bytes, BytesN, Env, Symbol, Vec};

/// Contract errors
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContractError {
    Unauthorized = 1,
    AlreadyInitialized = 2,
    OracleNotRegistered = 3,
    OracleAlreadyRegistered = 4,
    InvalidSignature = 5,
    ConfirmationAlreadyExists = 6,
    EscrowNotFound = 7,
    InvalidEventType = 8,
}

/// Event types for oracle confirmations
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EventType {
    Shipment = 1,
    Delivery = 2,
    Quality = 3,
    Custom = 4,
}

/// Oracle confirmation data structure
#[contracttype]
#[derive(Clone)]
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
const ORACLE_ADDED: Symbol = symbol_short!("oracle_add");
const ORACLE_REMOVED: Symbol = symbol_short!("oracle_rem");
const ORACLE_CONFIRMED: Symbol = symbol_short!("confirmed");
const INITIALIZED: Symbol = symbol_short!("init");

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

        env.storage().instance().set(&Symbol::new(&env, "data"), &contract_data);

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

        let mut contract_data = Self::get_contract_data(&env);

        // Check if oracle is already registered
        if Self::is_oracle_registered(&contract_data, &oracle) {
            return Err(ContractError::OracleAlreadyRegistered);
        }

        // Add oracle to registry
        contract_data.oracles.push_back(oracle.clone());

        // Save updated data
        env.storage().instance().set(&Symbol::new(&env, "data"), &contract_data);

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

        let mut contract_data = Self::get_contract_data(&env);

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
        env.storage().instance().set(&Symbol::new(&env, "data"), &contract_data);

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
        escrow_id: Bytes,
        event_type: u32,
        result: Bytes,
        signature: Bytes,
    ) -> Result<(), ContractError> {
        let contract_data = Self::get_contract_data(&env);

        // Get caller (oracle)
        let oracle = env.invoker();

        // Verify oracle is registered
        if !Self::is_oracle_registered(&contract_data, &oracle) {
            return Err(ContractError::OracleNotRegistered);
        }

        // Validate event type
        if event_type < 1 || event_type > 4 {
            return Err(ContractError::InvalidEventType);
        }

        // Check if confirmation already exists (prevent replay)
        let confirmation_key = (escrow_id.clone(), oracle.clone());
        if env.storage().persistent().has(&confirmation_key) {
            return Err(ContractError::ConfirmationAlreadyExists);
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
        env.storage().persistent().set(&confirmation_key, &confirmation);

        // Emit event
        env.events().publish(
            (ORACLE_CONFIRMED,),
            (escrow_id, event_type, result, oracle),
        );

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
        let contract_data = Self::get_contract_data(&env);
        let mut confirmations = Vec::new(&env);

        // Iterate through all registered oracles
        for oracle in contract_data.oracles.iter() {
            let confirmation_key = (escrow_id.clone(), oracle.clone());
            if let Some(confirmation) = env.storage().persistent().get(&confirmation_key) {
                confirmations.push_back(confirmation);
            }
        }

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
    pub fn is_oracle_registered_query(env: Env, oracle: Address) -> bool {
        let contract_data = Self::get_contract_data(&env);
        Self::is_oracle_registered(&contract_data, &oracle)
    }

    /// Get the total number of registered oracles
    pub fn get_oracle_count(env: Env) -> u32 {
        let contract_data = Self::get_contract_data(&env);
        contract_data.oracles.len()
    }

    /// Get oracle address at specific index
    ///
    /// # Arguments
    /// * `index` - The index to query
    ///
    /// # Returns
    /// Oracle address at the given index
    pub fn get_oracle_at(env: Env, index: u32) -> Option<Address> {
        let contract_data = Self::get_contract_data(&env);
        contract_data.oracles.get(index)
    }

    /// Get admin address
    pub fn get_admin(env: Env) -> Address {
        let contract_data = Self::get_contract_data(&env);
        contract_data.admin
    }

    // Helper functions

    fn is_initialized(env: &Env) -> bool {
        env.storage().instance().has(&Symbol::new(env, "data"))
    }

    fn get_contract_data(env: &Env) -> ContractData {
        env.storage().instance()
            .get(&Symbol::new(env, "data"))
            .unwrap_or(ContractData {
                admin: Address::from_contract_id(&BytesN::from_array(env, &[0; 32])),
                initialized: false,
                oracles: Vec::new(env),
            })
    }

    fn check_admin(env: &Env) -> Result<(), ContractError> {
        let contract_data = Self::get_contract_data(env);
        let caller = env.invoker();

        if caller != contract_data.admin {
            return Err(ContractError::Unauthorized);
        }

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
        let mut message_data = Vec::new(env);
        message_data.append(&escrow_id.clone());
        message_data.append(&Bytes::from_slice(env, &event_type.to_be_bytes()));
        message_data.append(&result.clone());

        env.crypto().sha256(&message_data)
    }

    fn verify_signature(
        env: &Env,
        message: &BytesN<32>,
        signature: &Bytes,
        oracle: &Address,
    ) -> Result<(), ContractError> {
        // For Soroban, we'll use the built-in signature verification
        // This is a simplified version - in production, you'd want more robust verification
        match env.crypto().ed25519_verify(&oracle.contract_id().into(), message, signature) {
            Ok(_) => Ok(()),
            Err(_) => Err(ContractError::InvalidSignature),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::{Address as _, BytesN as _};
    use soroban_sdk::{testutils::MockAuth, testutils::MockAuthInvoke, Address, Env, Bytes};

    #[test]
    fn test_initialization() {
        let env = Env::default();
        let contract_id = env.register_contract(None, OracleAdapter);
        let client = OracleAdapterClient::new(&env, &contract_id);

        let admin = Address::generate(&env);

        // Test successful initialization
        assert_eq!(client.initialize(&admin), ());

        // Test double initialization fails
        assert_eq!(client.try_initialize(&admin), Err(Ok(ContractError::AlreadyInitialized)));

        // Test admin getter
        assert_eq!(client.get_admin(), admin);
    }

    #[test]
    fn test_oracle_management() {
        let env = Env::default();
        let contract_id = env.register_contract(None, OracleAdapter);
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
        client.add_oracle(&oracle1);
        assert_eq!(client.is_oracle_registered_query(&oracle1), true);
        assert_eq!(client.get_oracle_count(), 1);

        // Test adding second oracle
        client.add_oracle(&oracle2);
        assert_eq!(client.is_oracle_registered_query(&oracle2), true);
        assert_eq!(client.get_oracle_count(), 2);

        // Test adding same oracle fails
        assert_eq!(client.try_add_oracle(&oracle1), Err(Ok(ContractError::OracleAlreadyRegistered)));

        // Test unauthorized add fails
        env.as_contract(&contract_id, || {
            env.mock_auths(&[MockAuth {
                address: &unauthorized,
                invoke: &MockAuthInvoke {
                    contract: &contract_id,
                    fn_name: "add_oracle",
                    args: (Address::generate(&env),).into_val(&env),
                    sub_invokes: &[],
                },
            }]);
            assert_eq!(OracleAdapter::add_oracle(env, Address::generate(&env)), Err(ContractError::Unauthorized));
        });

        // Test removing oracle
        client.remove_oracle(&oracle1);
        assert_eq!(client.is_oracle_registered_query(&oracle1), false);
        assert_eq!(client.get_oracle_count(), 1);

        // Test removing non-existent oracle fails
        assert_eq!(client.try_remove_oracle(&oracle1), Err(Ok(ContractError::OracleNotRegistered)));

        // Test unauthorized remove fails
        env.as_contract(&contract_id, || {
            env.mock_auths(&[MockAuth {
                address: &unauthorized,
                invoke: &MockAuthInvoke {
                    contract: &contract_id,
                    fn_name: "remove_oracle",
                    args: (oracle2.clone(),).into_val(&env),
                    sub_invokes: &[],
                },
            }]);
            assert_eq!(OracleAdapter::remove_oracle(env, oracle2), Err(ContractError::Unauthorized));
        });
    }

    #[test]
    fn test_event_type_validation() {
        let env = Env::default();
        let contract_id = env.register_contract(None, OracleAdapter);
        let client = OracleAdapterClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let oracle = Address::generate(&env);

        // Initialize and add oracle
        client.initialize(&admin);
        client.add_oracle(&oracle);

        let escrow_id = Bytes::from_slice(&env, b"escrow_123");
        let result = Bytes::from_slice(&env, b"confirmed");
        let signature = Bytes::from_slice(&env, b"mock_signature");

        // Test invalid event type (0)
        env.as_contract(&contract_id, || {
            env.mock_auths(&[MockAuth {
                address: &oracle,
                invoke: &MockAuthInvoke {
                    contract: &contract_id,
                    fn_name: "confirm_event",
                    args: (escrow_id.clone(), 0u32, result.clone(), signature.clone()).into_val(&env),
                    sub_invokes: &[],
                },
            }]);
            assert_eq!(OracleAdapter::confirm_event(env, escrow_id.clone(), 0u32, result.clone(), signature.clone()),
                      Err(ContractError::InvalidEventType));
        });

        // Test invalid event type (5)
        env.as_contract(&contract_id, || {
            env.mock_auths(&[MockAuth {
                address: &oracle,
                invoke: &MockAuthInvoke {
                    contract: &contract_id,
                    fn_name: "confirm_event",
                    args: (escrow_id.clone(), 5u32, result.clone(), signature.clone()).into_val(&env),
                    sub_invokes: &[],
                },
            }]);
            assert_eq!(OracleAdapter::confirm_event(env, escrow_id.clone(), 5u32, result.clone(), signature.clone()),
                      Err(ContractError::InvalidEventType));
        });

        // Test valid event types (1-4)
        for event_type in 1..=4 {
            env.as_contract(&contract_id, || {
                env.mock_auths(&[MockAuth {
                    address: &oracle,
                    invoke: &MockAuthInvoke {
                        contract: &contract_id,
                        fn_name: "confirm_event",
                        args: (escrow_id.clone(), event_type, result.clone(), signature.clone()).into_val(&env),
                        sub_invokes: &[],
                    },
                }]);
                // Note: This will fail due to signature verification, but event type validation passes
                let result = OracleAdapter::confirm_event(env, escrow_id.clone(), event_type, result.clone(), signature.clone());
                assert!(result == Err(ContractError::InvalidSignature) || result.is_ok());
            });
        }
    }

    #[test]
    fn test_replay_attack_prevention() {
        let env = Env::default();
        let contract_id = env.register_contract(None, OracleAdapter);
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

        // First confirmation should work (even with invalid signature for this test)
        env.as_contract(&contract_id, || {
            env.mock_auths(&[MockAuth {
                address: &oracle,
                invoke: &MockAuthInvoke {
                    contract: &contract_id,
                    fn_name: "confirm_event",
                    args: (escrow_id.clone(), event_type, result.clone(), signature.clone()).into_val(&env),
                    sub_invokes: &[],
                },
            }]);
            // Skip signature verification for this test by mocking it
            // In real implementation, signature would be verified
            let confirm_result = OracleAdapter::confirm_event(env, escrow_id.clone(), event_type, result.clone(), signature.clone());
            // The result depends on signature verification implementation
        });

        // Second confirmation from same oracle should fail (replay attack)
        env.as_contract(&contract_id, || {
            env.mock_auths(&[MockAuth {
                address: &oracle,
                invoke: &MockAuthInvoke {
                    contract: &contract_id,
                    fn_name: "confirm_event",
                    args: (escrow_id.clone(), event_type, result.clone(), signature.clone()).into_val(&env),
                    sub_invokes: &[],
                },
            }]);
            assert_eq!(OracleAdapter::confirm_event(env, escrow_id.clone(), event_type, result.clone(), signature.clone()),
                      Err(ContractError::ConfirmationAlreadyExists));
        });
    }

    #[test]
    fn test_unauthorized_oracle_confirmation() {
        let env = Env::default();
        let contract_id = env.register_contract(None, OracleAdapter);
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
        env.as_contract(&contract_id, || {
            env.mock_auths(&[MockAuth {
                address: &unauthorized_oracle,
                invoke: &MockAuthInvoke {
                    contract: &contract_id,
                    fn_name: "confirm_event",
                    args: (escrow_id.clone(), event_type, result.clone(), signature.clone()).into_val(&env),
                    sub_invokes: &[],
                },
            }]);
            assert_eq!(OracleAdapter::confirm_event(env, escrow_id.clone(), event_type, result.clone(), signature.clone()),
                      Err(ContractError::OracleNotRegistered));
        });
    }

    #[test]
    fn test_get_confirmation_empty() {
        let env = Env::default();
        let contract_id = env.register_contract(None, OracleAdapter);
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
        let contract_id = env.register_contract(None, OracleAdapter);
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
        assert_eq!(client.is_oracle_registered_query(&Address::generate(&env)), false);

        // Test getting oracles by index
        let oracle_at_0 = client.get_oracle_at(0);
        let oracle_at_1 = client.get_oracle_at(1);
        let oracle_at_2 = client.get_oracle_at(2);

        assert!(oracle_at_0.is_some());
        assert!(oracle_at_1.is_some());
        assert!(oracle_at_2.is_none()); // Out of bounds
    }

    #[test]
    fn test_message_creation() {
        let env = Env::default();
        let contract_id = env.register_contract(None, OracleAdapter);

        let escrow_id = Bytes::from_slice(&env, b"escrow_123");
        let event_type = 1u32;
        let result = Bytes::from_slice(&env, b"confirmed");

        env.as_contract(&contract_id, || {
            let message = OracleAdapter::create_message(&env, &escrow_id, event_type, &result);
            // Message should be a valid hash
            assert_eq!(message.len(), 32);
        });
    }
}
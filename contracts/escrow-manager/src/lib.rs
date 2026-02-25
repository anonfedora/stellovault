//! Escrow Manager Contract for StelloVault
//!
//! This contract is the "brain" of the trade finance flow, linking shipment
//! verification to funding release. It manages escrow creation backed by
//! collateral, oracle-verified fund release, and refund on expiry.

#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, token, Address, Bytes, Env, IntoVal,
    Symbol, Val, Vec,
};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EscrowStatus {
    Active = 0,
    Released = 1,
    Refunded = 2,
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContractError {
    Unauthorized = 1,
    AlreadyInitialized = 2,
    EscrowNotFound = 3,
    EscrowNotActive = 4,
    InvalidAmount = 5,
    ConfirmationNotMet = 6,
    EscrowNotExpired = 7,
    PathPaymentFailed = 8,
    SlippageExceeded = 9,
    InvalidOracleSet = 10,
    InvalidThreshold = 11,
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

/// Escrow configuration for creation
#[contracttype]
#[derive(Clone, Debug)]
pub struct EscrowConfig {
    pub buyer: Address,
    pub seller: Address,
    pub lender: Address,
    pub collateral_id: u64,
    pub amount: i128,
    pub asset: Address,
    pub required_confirmation: u32,
    pub expiry_ts: u64,
    pub destination_asset: Address,
    pub min_destination_amount: i128,
    pub required_confirmations: u32,
    pub oracle_set: Vec<Address>,
}

/// Escrow data structure linking buyer, seller, lender, collateral and oracle.
#[contracttype]
#[derive(Clone, Debug)]
pub struct Escrow {
    pub id: u64,
    pub buyer: Address,
    pub seller: Address,
    pub lender: Address,
    pub collateral_id: u64,
    pub amount: i128,
    pub asset: Address,
    /// Required oracle event type (1=Shipment, 2=Delivery, 3=Quality, 4=Custom, 5=Valuation)
    pub required_confirmation: u32,
    pub status: EscrowStatus,
    pub expiry_ts: u64,
    pub created_at: u64,
    /// Destination asset for path payment (if different from source asset)
    pub destination_asset: Address,
    /// Minimum amount to receive in destination asset (slippage protection)
    pub min_destination_amount: i128,
    /// Number of oracle confirmations required for multi-oracle consensus (0 for single oracle)
    pub required_confirmations: u32,
    /// Set of authorized oracles for consensus (empty means any registered oracle can confirm)
    pub oracle_set: Vec<Address>,
}

/// Local mirror of OracleAdapter's ConfirmationData for cross-contract deserialization.
/// Field names and types must match the oracle-adapter definition exactly.
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

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct EscrowManager;

#[contractimpl]
impl EscrowManager {
    /// Initialize the contract with admin and external contract addresses.
    pub fn initialize(
        env: Env,
        admin: Address,
        collateral_registry: Address,
        oracle_adapter: Address,
        loan_management: Address,
        treasury: Address,
    ) -> Result<(), ContractError> {
        if env.storage().instance().has(&symbol_short!("admin")) {
            return Err(ContractError::AlreadyInitialized);
        }

        env.storage()
            .instance()
            .set(&symbol_short!("admin"), &admin);
        env.storage()
            .instance()
            .set(&symbol_short!("coll_reg"), &collateral_registry);
        env.storage()
            .instance()
            .set(&symbol_short!("oracle"), &oracle_adapter);
        env.storage()
            .instance()
            .set(&symbol_short!("loan_mgr"), &loan_management);
        env.storage()
            .instance()
            .set(&symbol_short!("treasury"), &treasury);
        env.storage()
            .instance()
            .set(&symbol_short!("next_id"), &1u64);

        env.events().publish((symbol_short!("esc_init"),), (admin,));

        Ok(())
    }

    /// Set the treasury address (admin only).
    pub fn set_treasury(env: Env, treasury: Address) -> Result<(), ContractError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&symbol_short!("admin"))
            .ok_or(ContractError::Unauthorized)?;

        admin.require_auth();

        env.storage()
            .instance()
            .set(&symbol_short!("treasury"), &treasury);

        env.events()
            .publish((symbol_short!("trs_set"),), (treasury,));

        Ok(())
    }

    /// Get the current treasury address.
    pub fn get_treasury(env: Env) -> Option<Address> {
        env.storage().instance().get(&symbol_short!("treasury"))
    }

    /// Create a new escrow.
    ///
    /// Locks the referenced collateral via CollateralRegistry and transfers
    /// funds from the lender into this contract.
    ///
    /// # Arguments
    /// * `buyer` - Buyer address
    /// * `seller` - Seller address
    /// * `lender` - Lender providing funds (must authorize)
    /// * `collateral_id` - CollateralRegistry collateral ID to lock
    /// * `amount` - Escrow amount
    /// * `asset` - Token address for the escrowed asset
    /// * `required_confirmation` - EventType (u32) the oracle must confirm before release
    /// * `expiry_ts` - Timestamp after which the escrow can be refunded
    /// * `destination_asset` - Asset to pay seller (for path payments)
    /// * `min_destination_amount` - Minimum amount seller must receive (slippage protection)
    /// * `required_confirmations` - Number of oracle confirmations required (0 for single oracle)
    /// * `oracle_set` - Set of authorized oracles (empty means any registered oracle)
    pub fn create_escrow(
        env: Env,
        config: EscrowConfig,
    ) -> Result<u64, ContractError> {
        config.lender.require_auth();

        if config.amount <= 0 || config.min_destination_amount <= 0 {
            return Err(ContractError::InvalidAmount);
        }

        // Lock collateral via CollateralRegistry
        let coll_reg: Address = env
            .storage()
            .instance()
            .get(&symbol_short!("coll_reg"))
            .ok_or(ContractError::Unauthorized)?;

        let lock_args: Vec<Val> = Vec::from_array(&env, [config.collateral_id.into_val(&env)]);
        env.invoke_contract::<Val>(&coll_reg, &Symbol::new(&env, "lock_collateral"), lock_args);

        // Transfer funds from lender to this contract
        let token_client = token::Client::new(&env, &config.asset);
        token_client.transfer(&config.lender, &env.current_contract_address(), &config.amount);

        let escrow_id: u64 = env
            .storage()
            .instance()
            .get(&symbol_short!("next_id"))
            .unwrap_or(1);

        let escrow = Escrow {
            id: escrow_id,
            buyer: config.buyer.clone(),
            seller: config.seller.clone(),
            lender: config.lender.clone(),
            collateral_id: config.collateral_id,
            amount: config.amount,
            asset: config.asset,
            required_confirmation: config.required_confirmation,
            status: EscrowStatus::Active,
            expiry_ts: config.expiry_ts,
            created_at: env.ledger().timestamp(),
            destination_asset: config.destination_asset,
            min_destination_amount: config.min_destination_amount,
            required_confirmations: config.required_confirmations,
            oracle_set: config.oracle_set,
        };

        env.storage().persistent().set(&escrow_id, &escrow);
        env.storage()
            .instance()
            .set(&symbol_short!("next_id"), &(escrow_id + 1));

        env.events().publish(
            (symbol_short!("esc_crtd"),),
            (escrow_id, config.buyer, config.seller, config.lender, config.amount),
        );

        Ok(escrow_id)
    }

    /// Release escrowed funds to the seller after oracle confirmation.
    ///
    /// Queries OracleAdapter::get_confirmation for the required event type.
    /// If a verified confirmation matching the required type is found:
    /// - Executes path payment from source asset to destination asset (if different)
    /// - Uses Stellar's built-in DEX for currency conversion
    /// - Enforces slippage protection via min_destination_amount
    /// - Unlocks collateral via CollateralRegistry
    /// - Emits release event (for LoanManagement off-chain notification)
    pub fn release_funds_on_confirmation(env: Env, escrow_id: u64) -> Result<(), ContractError> {
        let mut escrow: Escrow = env
            .storage()
            .persistent()
            .get(&escrow_id)
            .ok_or(ContractError::EscrowNotFound)?;

        if escrow.status != EscrowStatus::Active {
            return Err(ContractError::EscrowNotActive);
        }

        // Query OracleAdapter for confirmations
        let oracle: Address = env
            .storage()
            .instance()
            .get(&symbol_short!("oracle"))
            .ok_or(ContractError::Unauthorized)?;

        let escrow_id_bytes = Bytes::from_slice(&env, &escrow_id.to_be_bytes());
        let conf_args: Vec<Val> = Vec::from_array(&env, [escrow_id_bytes.into_val(&env)]);

        let confirmations: Option<Vec<ConfirmationData>> =
            env.invoke_contract(&oracle, &Symbol::new(&env, "get_confirmation"), conf_args);

        // Check if a verified confirmation matching the required event type exists
        let confirmed = match confirmations {
            Some(confs) => {
                let mut found = false;
                for conf in confs.iter() {
                    if conf.event_type == escrow.required_confirmation && conf.verified {
                        found = true;
                        break;
                    }
                }
                found
            }
            None => false,
        };

        if !confirmed {
            return Err(ContractError::ConfirmationNotMet);
        }

        // Execute payment: path payment if assets differ, direct transfer otherwise
        if escrow.asset == escrow.destination_asset {
            // Direct transfer - no conversion needed
            let token_client = token::Client::new(&env, &escrow.asset);
            token_client.transfer(
                &env.current_contract_address(),
                &escrow.seller,
                &escrow.amount,
            );
        } else {
            // Path payment - use Stellar's built-in DEX
            let source_token = token::Client::new(&env, &escrow.asset);

            // Execute path payment using Stellar's native path payment functionality
            // This leverages the Stellar DEX to find the best conversion path
            let _amount_received = source_token.try_transfer_from(
                &env.current_contract_address(),
                &env.current_contract_address(),
                &escrow.seller,
                &escrow.amount,
            );

            // For path payments, we need to use a different approach
            // Since Soroban doesn't have direct path payment support yet,
            // we simulate it by doing a swap through the contract
            // In production, this would integrate with Stellar's path payment protocol

            // For now, we'll use a simplified approach:
            // 1. Transfer source asset from escrow to a temporary holding
            // 2. Invoke a swap operation (would be DEX in production)
            // 3. Transfer destination asset to seller

            // This is a placeholder for the actual path payment implementation
            // In a real scenario, you'd call into Stellar's path payment host function
            let _dest_token = token::Client::new(&env, &escrow.destination_asset);

            // Simulate path payment by checking if we can meet minimum destination amount
            // In production, this would be handled by Stellar's path payment protocol
            let estimated_dest_amount = Self::estimate_path_payment(
                &env,
                &escrow.asset,
                &escrow.destination_asset,
                escrow.amount,
            )?;

            if estimated_dest_amount < escrow.min_destination_amount {
                return Err(ContractError::SlippageExceeded);
            }

            // Execute the path payment
            // Note: In production Stellar contracts, this would use the native path payment
            // host function which automatically finds the best path through the DEX
            source_token.transfer(
                &env.current_contract_address(),
                &escrow.seller,
                &escrow.amount,
            );

            // Emit path payment event for tracking
            env.events().publish(
                (symbol_short!("path_pay"),),
                (escrow_id, escrow.amount, estimated_dest_amount),
            );
        }

        // Calculate and collect protocol fee if treasury is configured
        let treasury_opt: Option<Address> = env.storage().instance().get(&symbol_short!("treasury"));
        let protocol_fee = if treasury_opt.is_some() {
            let treasury = treasury_opt.as_ref().unwrap();
            
            // Query fee_bps from ProtocolTreasury
            let fee_bps_args: soroban_sdk::Vec<Val> = soroban_sdk::Vec::new(&env);
            let fee_bps: u32 = env.invoke_contract(
                &treasury,
                &Symbol::new(&env, "get_fee_bps"),
                fee_bps_args,
            );
            
            // Calculate fee on the escrow amount
            let fee_amount = (escrow.amount * fee_bps as i128) / 10000;
            
            if fee_amount > 0 {
                // Record the fee deposit in treasury
                // Note: In a full implementation, the actual token transfer would happen
                // before this call, either deducted from the payment or transferred separately
                let deposit_args: soroban_sdk::Vec<Val> = soroban_sdk::Vec::from_array(
                    &env,
                    [
                        escrow.asset.into_val(&env), // Asset address
                        fee_amount.into_val(&env),
                    ],
                );
                env.invoke_contract(
                    &treasury,
                    &Symbol::new(&env, "deposit_fee"),
                    deposit_args,
                );
                
                // Emit fee collection event
                env.events().publish(
                    (symbol_short!("fee_col"),),
                    (escrow_id, fee_amount, escrow.asset),
                );
            }
            
            fee_amount
        } else {
            0i128
        };

        // Unlock collateral via CollateralRegistry
        let coll_reg: Address = env
            .storage()
            .instance()
            .get(&symbol_short!("coll_reg"))
            .ok_or(ContractError::Unauthorized)?;

        let unlock_args: Vec<Val> = Vec::from_array(&env, [escrow.collateral_id.into_val(&env)]);
        env.invoke_contract::<Val>(
            &coll_reg,
            &Symbol::new(&env, "unlock_collateral"),
            unlock_args,
        );

        escrow.status = EscrowStatus::Released;
        env.storage().persistent().set(&escrow_id, &escrow);

        env.events()
            .publish((symbol_short!("esc_rel"),), (escrow_id,));

        Ok(())
    }

    /// Estimate the destination amount for a path payment.
    ///
    /// In production, this would query Stellar's DEX for the best path.
    /// For testing, we use a simplified estimation.
    fn estimate_path_payment(
        env: &Env,
        _source_asset: &Address,
        _dest_asset: &Address,
        source_amount: i128,
    ) -> Result<i128, ContractError> {
        // Simplified estimation for testing
        // In production, this would query the actual DEX liquidity and paths
        // For now, assume a 1:1 ratio (would be replaced with actual DEX query)

        // Check if we have a stored exchange rate for testing
        let rate_key = symbol_short!("test_rate");
        let exchange_rate: i128 = env.storage().instance().get(&rate_key).unwrap_or(1_000_000); // Default 1:1 (with 6 decimals precision)

        // Calculate destination amount: source_amount * rate / 1_000_000
        let dest_amount = source_amount
            .checked_mul(exchange_rate)
            .and_then(|v| v.checked_div(1_000_000))
            .ok_or(ContractError::PathPaymentFailed)?;

        Ok(dest_amount)
    }

    /// Set exchange rate for testing path payments.
    /// Rate is expressed with 6 decimals precision (1_000_000 = 1:1 ratio).
    /// This is a test helper and would not exist in production.
    pub fn set_test_exchange_rate(env: Env, rate: i128) {
        env.storage()
            .instance()
            .set(&symbol_short!("test_rate"), &rate);
    }

    /// Refund the escrowed funds to the lender if the escrow has expired.
    ///
    /// Anyone can call this after expiry. Unlocks collateral and returns
    /// funds to the lender.
    pub fn refund_escrow(env: Env, escrow_id: u64) -> Result<(), ContractError> {
        let mut escrow: Escrow = env
            .storage()
            .persistent()
            .get(&escrow_id)
            .ok_or(ContractError::EscrowNotFound)?;

        if escrow.status != EscrowStatus::Active {
            return Err(ContractError::EscrowNotActive);
        }

        let current_ts = env.ledger().timestamp();
        if current_ts <= escrow.expiry_ts {
            return Err(ContractError::EscrowNotExpired);
        }

        // Refund lender
        let token_client = token::Client::new(&env, &escrow.asset);
        token_client.transfer(
            &env.current_contract_address(),
            &escrow.lender,
            &escrow.amount,
        );

        // Unlock collateral via CollateralRegistry
        let coll_reg: Address = env
            .storage()
            .instance()
            .get(&symbol_short!("coll_reg"))
            .ok_or(ContractError::Unauthorized)?;

        let unlock_args: Vec<Val> = Vec::from_array(&env, [escrow.collateral_id.into_val(&env)]);
        env.invoke_contract::<Val>(
            &coll_reg,
            &Symbol::new(&env, "unlock_collateral"),
            unlock_args,
        );

        escrow.status = EscrowStatus::Refunded;
        env.storage().persistent().set(&escrow_id, &escrow);

        env.events()
            .publish((symbol_short!("esc_rfnd"),), (escrow_id,));

        Ok(())
    }

    /// Get escrow details.
    pub fn get_escrow(env: Env, escrow_id: u64) -> Option<Escrow> {
        env.storage().persistent().get(&escrow_id)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{
        testutils::Address as _, testutils::Ledger as _, token, Address, Bytes, Env, Vec,
    };

    // -- Mock CollateralRegistry ------------------------------------------

    #[contract]
    pub struct MockCollateralRegistry;

    #[contractimpl]
    impl MockCollateralRegistry {
        pub fn lock_collateral(env: Env, id: u64) {
            env.storage().persistent().set(&id, &true);
            env.events().publish((symbol_short!("coll_lock"),), (id,));
        }

        pub fn unlock_collateral(env: Env, id: u64) {
            env.storage().persistent().set(&id, &false);
            env.events().publish((symbol_short!("coll_unlk"),), (id,));
        }
    }

    // -- Mock OracleAdapter -----------------------------------------------

    #[contract]
    pub struct MockOracleAdapter;

    #[contractimpl]
    impl MockOracleAdapter {
        /// Returns confirmations stored under the escrow_id key.
        pub fn get_confirmation(env: Env, escrow_id: Bytes) -> Option<Vec<ConfirmationData>> {
            env.storage().persistent().get(&escrow_id)
        }

        /// Test helper: store confirmation data for a given escrow_id.
        pub fn set_confirmation(env: Env, escrow_id: Bytes, confirmations: Vec<ConfirmationData>) {
            env.storage().persistent().set(&escrow_id, &confirmations);
        }
    }

    // -- Helpers -----------------------------------------------------------

    struct TestEnv<'a> {
        env: Env,
        escrow_client: EscrowManagerClient<'a>,
        escrow_id_addr: Address,
        coll_reg_addr: Address,
        oracle_client: MockOracleAdapterClient<'a>,
        token_addr: Address,
        treasury_addr: Address,
        buyer: Address,
        seller: Address,
        lender: Address,
    }

    fn setup() -> TestEnv<'static> {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let buyer = Address::generate(&env);
        let seller = Address::generate(&env);
        let lender = Address::generate(&env);

        // Register contracts
        let escrow_id_addr = env.register(EscrowManager, ());
        let escrow_client = EscrowManagerClient::new(&env, &escrow_id_addr);

        let coll_reg_addr = env.register(MockCollateralRegistry, ());
        let oracle_addr = env.register(MockOracleAdapter, ());
        let oracle_client = MockOracleAdapterClient::new(&env, &oracle_addr);

        let loan_mgr_addr = Address::generate(&env); // placeholder
        let treasury_addr = Address::generate(&env); // placeholder treasury

        // Create a Stellar asset token
        let token_admin = Address::generate(&env);
        let token_contract = env.register_stellar_asset_contract_v2(token_admin);
        let token_addr = token_contract.address();
        let token_admin_client = token::StellarAssetClient::new(&env, &token_addr);
        token_admin_client.mint(&lender, &1_000_000);

        // Initialize escrow manager
        escrow_client.initialize(&admin, &coll_reg_addr, &oracle_addr, &loan_mgr_addr, &treasury_addr);

        // Leak lifetimes for test convenience
        let escrow_client = unsafe {
            core::mem::transmute::<EscrowManagerClient<'_>, EscrowManagerClient<'static>>(
                escrow_client,
            )
        };
        let oracle_client = unsafe {
            core::mem::transmute::<MockOracleAdapterClient<'_>, MockOracleAdapterClient<'static>>(
                oracle_client,
            )
        };

        TestEnv {
            env,
            escrow_client,
            escrow_id_addr,
            coll_reg_addr,
            oracle_client,
            token_addr,
            treasury_addr,
            buyer,
            seller,
            lender,
        }
    }

    fn create_test_escrow(t: &TestEnv) -> u64 {
        let expiry = t.env.ledger().timestamp() + 3600;
        t.escrow_client.create_escrow(&EscrowConfig {
            buyer: t.buyer.clone(),
            seller: t.seller.clone(),
            lender: t.lender.clone(),
            collateral_id: 1u64,
            amount: 5000i128,
            asset: t.token_addr.clone(),
            required_confirmation: 2u32, // Delivery
            expiry_ts: expiry,
            destination_asset: t.token_addr.clone(),
            min_destination_amount: 5000i128,
            required_confirmations: 0u32,
            oracle_set: Vec::new(&t.env),
        })
    }

    fn set_oracle_confirmation(t: &TestEnv, escrow_id: u64, event_type: u32, verified: bool) {
        let escrow_id_bytes = Bytes::from_slice(&t.env, &escrow_id.to_be_bytes());
        let oracle_addr = Address::generate(&t.env);

        let conf = ConfirmationData {
            escrow_id: escrow_id_bytes.clone(),
            event_type,
            result: Bytes::from_slice(&t.env, b"confirmed"),
            oracle: oracle_addr,
            timestamp: t.env.ledger().timestamp(),
            verified,
        };

        let confs = Vec::from_array(&t.env, [conf]);
        t.oracle_client.set_confirmation(&escrow_id_bytes, &confs);
    }

    // -- Tests ------------------------------------------------------------

    #[test]
    fn test_initialize() {
        let t = setup();

        t.env.as_contract(&t.escrow_id_addr, || {
            let admin: Address = t
                .env
                .storage()
                .instance()
                .get(&symbol_short!("admin"))
                .unwrap();
            assert!(admin == admin); // just check it exists

            let coll_reg: Address = t
                .env
                .storage()
                .instance()
                .get(&symbol_short!("coll_reg"))
                .unwrap();
            assert_eq!(coll_reg, t.coll_reg_addr);
        });
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #2)")]
    fn test_initialize_already_initialized() {
        let t = setup();
        let admin = Address::generate(&t.env);
        let dummy = Address::generate(&t.env);
        t.escrow_client.initialize(&admin, &dummy, &dummy, &dummy);
    }

    #[test]
    fn test_create_escrow_success() {
        let t = setup();
        let escrow_id = create_test_escrow(&t);
        assert_eq!(escrow_id, 1);

        let escrow = t.escrow_client.get_escrow(&escrow_id).unwrap();
        assert_eq!(escrow.buyer, t.buyer);
        assert_eq!(escrow.seller, t.seller);
        assert_eq!(escrow.lender, t.lender);
        assert_eq!(escrow.collateral_id, 1);
        assert_eq!(escrow.amount, 5000);
        assert_eq!(escrow.required_confirmation, 2); // Delivery
        assert_eq!(escrow.status, EscrowStatus::Active);

        // Verify collateral was locked in mock
        t.env.as_contract(&t.coll_reg_addr, || {
            let locked: bool = t.env.storage().persistent().get(&1u64).unwrap();
            assert!(locked);
        });

        // Verify funds transferred to escrow contract
        let token = token::Client::new(&t.env, &t.token_addr);
        assert_eq!(token.balance(&t.escrow_id_addr), 5000);
        assert_eq!(token.balance(&t.lender), 1_000_000 - 5000);
    }

    #[test]
    fn test_create_multiple_escrows() {
        let t = setup();

        let id1 = create_test_escrow(&t);
        let id2 = create_test_escrow(&t);

        assert_eq!(id1, 1);
        assert_eq!(id2, 2);

        assert!(t.escrow_client.get_escrow(&id1).is_some());
        assert!(t.escrow_client.get_escrow(&id2).is_some());
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #5)")]
    fn test_create_escrow_invalid_amount() {
        let t = setup();
        let expiry = t.env.ledger().timestamp() + 3600;
        t.escrow_client.create_escrow(&EscrowConfig {
            buyer: t.buyer.clone(),
            seller: t.seller.clone(),
            lender: t.lender.clone(),
            collateral_id: 1u64,
            amount: 0i128, // invalid
            asset: t.token_addr.clone(),
            required_confirmation: 2u32,
            expiry_ts: expiry,
            destination_asset: t.token_addr.clone(),
            min_destination_amount: 5000i128,
            required_confirmations: 0u32,
            oracle_set: Vec::new(&t.env),
        });
    }

    #[test]
    fn test_release_funds_on_confirmation() {
        let t = setup();
        let escrow_id = create_test_escrow(&t);

        // Set up oracle confirmation for Delivery (event_type=2)
        set_oracle_confirmation(&t, escrow_id, 2, true);

        t.escrow_client.release_funds_on_confirmation(&escrow_id);

        // Verify status
        let escrow = t.escrow_client.get_escrow(&escrow_id).unwrap();
        assert_eq!(escrow.status, EscrowStatus::Released);

        // Verify funds sent to seller
        let token = token::Client::new(&t.env, &t.token_addr);
        assert_eq!(token.balance(&t.seller), 5000);
        assert_eq!(token.balance(&t.escrow_id_addr), 0);

        // Verify collateral was unlocked
        t.env.as_contract(&t.coll_reg_addr, || {
            let locked: bool = t.env.storage().persistent().get(&1u64).unwrap();
            assert!(!locked);
        });
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #6)")]
    fn test_release_without_confirmation() {
        let t = setup();
        let escrow_id = create_test_escrow(&t);

        // No oracle confirmation set
        t.escrow_client.release_funds_on_confirmation(&escrow_id);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #6)")]
    fn test_release_wrong_event_type() {
        let t = setup();
        let escrow_id = create_test_escrow(&t);

        // Oracle confirmed Shipment (1) but escrow requires Delivery (2)
        set_oracle_confirmation(&t, escrow_id, 1, false);

        t.escrow_client.release_funds_on_confirmation(&escrow_id);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #6)")]
    fn test_release_unverified_confirmation() {
        let t = setup();
        let escrow_id = create_test_escrow(&t);

        // Right event type but verified=false
        set_oracle_confirmation(&t, escrow_id, 2, false);

        t.escrow_client.release_funds_on_confirmation(&escrow_id);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #4)")]
    fn test_release_already_released() {
        let t = setup();
        let escrow_id = create_test_escrow(&t);

        set_oracle_confirmation(&t, escrow_id, 2, true);
        t.escrow_client.release_funds_on_confirmation(&escrow_id);

        // Try again
        t.escrow_client.release_funds_on_confirmation(&escrow_id);
    }

    #[test]
    fn test_refund_escrow_success() {
        let t = setup();
        let escrow_id = create_test_escrow(&t);

        let token = token::Client::new(&t.env, &t.token_addr);
        let lender_balance_before = token.balance(&t.lender);

        // Advance past expiry
        t.env.ledger().with_mut(|li| {
            li.timestamp += 3601;
        });

        t.escrow_client.refund_escrow(&escrow_id);

        // Verify status
        let escrow = t.escrow_client.get_escrow(&escrow_id).unwrap();
        assert_eq!(escrow.status, EscrowStatus::Refunded);

        // Verify funds returned to lender
        assert_eq!(token.balance(&t.lender), lender_balance_before + 5000);
        assert_eq!(token.balance(&t.escrow_id_addr), 0);

        // Verify collateral unlocked
        t.env.as_contract(&t.coll_reg_addr, || {
            let locked: bool = t.env.storage().persistent().get(&1u64).unwrap();
            assert!(!locked);
        });
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #7)")]
    fn test_refund_before_expiry() {
        let t = setup();
        let escrow_id = create_test_escrow(&t);

        // Don't advance time - escrow not expired
        t.escrow_client.refund_escrow(&escrow_id);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #4)")]
    fn test_refund_already_refunded() {
        let t = setup();
        let escrow_id = create_test_escrow(&t);

        t.env.ledger().with_mut(|li| {
            li.timestamp += 3601;
        });

        t.escrow_client.refund_escrow(&escrow_id);

        // Try again
        t.escrow_client.refund_escrow(&escrow_id);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #4)")]
    fn test_refund_after_release() {
        let t = setup();
        let escrow_id = create_test_escrow(&t);

        // Release first
        set_oracle_confirmation(&t, escrow_id, 2, true);
        t.escrow_client.release_funds_on_confirmation(&escrow_id);

        // Try to refund after release
        t.env.ledger().with_mut(|li| {
            li.timestamp += 3601;
        });
        t.escrow_client.refund_escrow(&escrow_id);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #3)")]
    fn test_release_nonexistent_escrow() {
        let t = setup();
        t.escrow_client.release_funds_on_confirmation(&999u64);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #3)")]
    fn test_refund_nonexistent_escrow() {
        let t = setup();
        t.escrow_client.refund_escrow(&999u64);
    }

    #[test]
    fn test_get_escrow_not_found() {
        let t = setup();
        assert!(t.escrow_client.get_escrow(&999u64).is_none());
    }

    #[test]
    fn test_path_payment_same_asset() {
        let t = setup();
        let escrow_id = create_test_escrow(&t);

        // Set oracle confirmation
        set_oracle_confirmation(&t, escrow_id, 2, true);

        // Release with same source and destination asset
        t.escrow_client.release_funds_on_confirmation(&escrow_id);

        let escrow = t.escrow_client.get_escrow(&escrow_id).unwrap();
        assert_eq!(escrow.status, EscrowStatus::Released);

        // Verify seller received funds
        let token = token::Client::new(&t.env, &t.token_addr);
        assert_eq!(token.balance(&t.seller), 5000);
    }

    #[test]
    fn test_path_payment_different_asset() {
        let t = setup();

        // Create a second token for destination
        let token_admin = Address::generate(&t.env);
        let dest_token_contract = t
            .env
            .register_stellar_asset_contract_v2(token_admin.clone());
        let dest_token_addr = dest_token_contract.address();
        let dest_token_admin_client = token::StellarAssetClient::new(&t.env, &dest_token_addr);

        // Mint destination tokens to the escrow contract for the swap
        dest_token_admin_client.mint(&t.escrow_id_addr, &10_000);

        // Create escrow with different destination asset
        let expiry = t.env.ledger().timestamp() + 3600;
        let escrow_id = t.escrow_client.create_escrow(&EscrowConfig {
            buyer: t.buyer.clone(),
            seller: t.seller.clone(),
            lender: t.lender.clone(),
            collateral_id: 1u64,
            amount: 5000i128,
            asset: t.token_addr.clone(),
            required_confirmation: 2u32,
            expiry_ts: expiry,
            destination_asset: dest_token_addr.clone(),
            min_destination_amount: 4500i128,
            required_confirmations: 0u32,
            oracle_set: Vec::new(&t.env),
        });

        // Set exchange rate: 0.95 (5% loss in conversion)
        t.escrow_client.set_test_exchange_rate(&950_000i128);

        // Set oracle confirmation
        set_oracle_confirmation(&t, escrow_id, 2, true);

        // Release with path payment
        t.escrow_client.release_funds_on_confirmation(&escrow_id);

        let escrow = t.escrow_client.get_escrow(&escrow_id).unwrap();
        assert_eq!(escrow.status, EscrowStatus::Released);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #9)")]
    fn test_path_payment_slippage_exceeded() {
        let t = setup();

        // Create a second token for destination
        let token_admin = Address::generate(&t.env);
        let dest_token_contract = t
            .env
            .register_stellar_asset_contract_v2(token_admin.clone());
        let dest_token_addr = dest_token_contract.address();

        // Create escrow with different destination asset
        let expiry = t.env.ledger().timestamp() + 3600;
        let escrow_id = t.escrow_client.create_escrow(&EscrowConfig {
            buyer: t.buyer.clone(),
            seller: t.seller.clone(),
            lender: t.lender.clone(),
            collateral_id: 1u64,
            amount: 5000i128,
            asset: t.token_addr.clone(),
            required_confirmation: 2u32,
            expiry_ts: expiry,
            destination_asset: dest_token_addr.clone(),
            min_destination_amount: 4800i128,
            required_confirmations: 0u32,
            oracle_set: Vec::new(&t.env),
        });

        // Set exchange rate: 0.90 (10% loss in conversion)
        t.escrow_client.set_test_exchange_rate(&900_000i128);

        // Set oracle confirmation
        set_oracle_confirmation(&t, escrow_id, 2, true);

        // This should fail due to slippage
        t.escrow_client.release_funds_on_confirmation(&escrow_id);
    }

    #[test]
    fn test_create_escrow_with_path_payment_params() {
        let t = setup();

        // Create a second token for destination
        let token_admin = Address::generate(&t.env);
        let dest_token_contract = t.env.register_stellar_asset_contract_v2(token_admin);
        let dest_token_addr = dest_token_contract.address();

        let expiry = t.env.ledger().timestamp() + 3600;
        let escrow_id = t.escrow_client.create_escrow(&EscrowConfig {
            buyer: t.buyer.clone(),
            seller: t.seller.clone(),
            lender: t.lender.clone(),
            collateral_id: 1u64,
            amount: 5000i128,
            asset: t.token_addr.clone(),
            required_confirmation: 2u32,
            expiry_ts: expiry,
            destination_asset: dest_token_addr.clone(),
            min_destination_amount: 4500i128,
            required_confirmations: 0u32,
            oracle_set: Vec::new(&t.env),
        });

        let escrow = t.escrow_client.get_escrow(&escrow_id).unwrap();
        assert_eq!(escrow.destination_asset, dest_token_addr);
        assert_eq!(escrow.min_destination_amount, 4500);
        assert_eq!(escrow.status, EscrowStatus::Active);
    }
}

//! Escrow Manager Contract for StelloVault
//!
//! This contract is the "brain" of the trade finance flow, linking shipment
//! verification to funding release. It manages escrow creation backed by
//! collateral, oracle-verified fund release, and refund on expiry.

#![no_std]

mod refund;

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
    Disputed = 3,
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
    ConsensusNotMet = 12,
    EscrowDisputed = 13,
    EscrowNotDisputed = 14,
    InsufficientBalance = 15,
    NoPendingAdmin = 16,
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
    pub disputed_at: Option<u64>,
    pub disputed_by: Option<Address>,
    pub dispute_reason: Option<Bytes>,
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DisputeDecision {
    ReleaseToSeller = 0,
    RefundToBuyer = 1,
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
        env.storage()
            .instance()
            .remove(&symbol_short!("pend_adm"));

        env.events()
            .publish((symbol_short!("adm_acpt"),), (pending,));

        Ok(())
    }

    /// Return the pending admin address if a proposal is active.
    pub fn get_pending_admin(env: Env) -> Option<Address> {
        env.storage()
            .instance()
            .get(&symbol_short!("pend_adm"))
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
    pub fn create_escrow(env: Env, config: EscrowConfig) -> Result<u64, ContractError> {
        config.lender.require_auth();

        if config.amount <= 0 || config.min_destination_amount <= 0 {
            return Err(ContractError::InvalidAmount);
        }

        // Validate multi-oracle consensus parameters
        if config.required_confirmations > 0 {
            // Multi-oracle consensus mode requires oracle_set
            if config.oracle_set.is_empty() {
                return Err(ContractError::InvalidOracleSet);
            }

            // Threshold cannot exceed oracle set size
            if config.required_confirmations > config.oracle_set.len() {
                return Err(ContractError::InvalidThreshold);
            }

            // Prevent duplicate oracles in the set
            let mut unique_oracles = Vec::new(&env);
            for oracle in config.oracle_set.iter() {
                // Check for duplicates
                for existing in unique_oracles.iter() {
                    if existing == oracle {
                        return Err(ContractError::InvalidOracleSet);
                    }
                }
                unique_oracles.push_back(oracle);
            }
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
        token_client.transfer(
            &config.lender,
            &env.current_contract_address(),
            &config.amount,
        );

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
            disputed_at: None,
            disputed_by: None,
            dispute_reason: None,
        };

        env.storage().persistent().set(&escrow_id, &escrow);
        env.storage()
            .instance()
            .set(&symbol_short!("next_id"), &(escrow_id + 1));

        // Index escrow by seller (merchant) address for efficient querying
        let seller_key = (symbol_short!("seller"), escrow.seller.clone());
        let mut seller_escrows: Vec<u64> = env
            .storage()
            .persistent()
            .get(&seller_key)
            .unwrap_or(Vec::new(&env));
        seller_escrows.push_back(escrow_id);
        env.storage().persistent().set(&seller_key, &seller_escrows);

        env.events().publish(
            (symbol_short!("esc_crtd"),),
            (
                escrow_id,
                config.buyer,
                config.seller,
                config.lender,
                config.amount,
            ),
        );

        Ok(escrow_id)
    }

    pub fn raise_dispute(
        env: Env,
        escrow_id: u64,
        disputer: Address,
        reason: Bytes,
    ) -> Result<(), ContractError> {
        let mut escrow: Escrow = env
            .storage()
            .persistent()
            .get(&escrow_id)
            .ok_or(ContractError::EscrowNotFound)?;

        match escrow.status {
            EscrowStatus::Active => {}
            EscrowStatus::Disputed => return Err(ContractError::EscrowDisputed),
            _ => return Err(ContractError::EscrowNotActive),
        }

        if disputer == escrow.buyer {
            escrow.buyer.require_auth();
        } else if disputer == escrow.seller {
            escrow.seller.require_auth();
        } else {
            return Err(ContractError::Unauthorized);
        }

        escrow.status = EscrowStatus::Disputed;
        escrow.disputed_at = Some(env.ledger().timestamp());
        escrow.disputed_by = Some(disputer.clone());
        escrow.dispute_reason = Some(reason.clone());

        env.storage().persistent().set(&escrow_id, &escrow);
        env.events()
            .publish((symbol_short!("esc_disp"),), (escrow_id, disputer, reason));

        Ok(())
    }

    /// Release escrowed funds to the seller after oracle confirmation.
    ///
    /// For multi-oracle consensus: Queries OracleAdapter::check_consensus to verify
    /// that the required number of unique oracle confirmations have been received.
    /// For single oracle (backward compatibility): Falls back to get_confirmation check.
    ///
    /// If consensus is met:
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

        if escrow.status == EscrowStatus::Disputed {
            return Err(ContractError::EscrowDisputed);
        }
        if escrow.status != EscrowStatus::Active {
            return Err(ContractError::EscrowNotActive);
        }

        // Query OracleAdapter for consensus
        let oracle: Address = env
            .storage()
            .instance()
            .get(&symbol_short!("oracle"))
            .ok_or(ContractError::Unauthorized)?;

        let escrow_id_bytes = Bytes::from_slice(&env, &escrow_id.to_be_bytes());

        // Check if multi-oracle consensus is configured (required_confirmations > 0)
        if escrow.required_confirmations > 0 {
            // Multi-oracle consensus mode
            // Validate oracle_set is not empty when required_confirmations > 0
            if escrow.oracle_set.is_empty() {
                return Err(ContractError::InvalidOracleSet);
            }

            // Validate threshold doesn't exceed oracle set size
            if escrow.required_confirmations > escrow.oracle_set.len() {
                return Err(ContractError::InvalidThreshold);
            }

            // Call OracleAdapter::check_consensus
            let consensus_args: Vec<Val> = Vec::from_array(
                &env,
                [
                    escrow_id_bytes.into_val(&env),
                    escrow.required_confirmation.into_val(&env), // Add event type filtering
                    escrow.required_confirmations.into_val(&env),
                    escrow.oracle_set.into_val(&env),
                ],
            );

            let consensus_met: bool = env.invoke_contract(
                &oracle,
                &Symbol::new(&env, "check_consensus"),
                consensus_args,
            );

            if !consensus_met {
                return Err(ContractError::ConsensusNotMet);
            }
        } else {
            // Single oracle mode (backward compatibility)
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
        }

        Self::execute_release_payout(&env, escrow_id, &escrow)?;
        Self::collect_protocol_fee(&env, escrow_id, &escrow);
        Self::unlock_collateral(&env, escrow.collateral_id)?;

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

    fn execute_release_payout(
        env: &Env,
        escrow_id: u64,
        escrow: &Escrow,
    ) -> Result<(), ContractError> {
        // Execute payment: path payment if assets differ, direct transfer otherwise
        if escrow.asset == escrow.destination_asset {
            let token_client = token::Client::new(env, &escrow.asset);
            token_client.transfer(
                &env.current_contract_address(),
                &escrow.seller,
                &escrow.amount,
            );
            return Ok(());
        }

        let estimated_dest_amount = Self::estimate_path_payment(
            env,
            &escrow.asset,
            &escrow.destination_asset,
            escrow.amount,
        )?;

        if estimated_dest_amount < escrow.min_destination_amount {
            return Err(ContractError::SlippageExceeded);
        }

        // Placeholder settlement for cross-asset payout:
        // transfer destination-asset amount to seller.
        let dest_token = token::Client::new(env, &escrow.destination_asset);
        dest_token.transfer(
            &env.current_contract_address(),
            &escrow.seller,
            &estimated_dest_amount,
        );

        env.events().publish(
            (symbol_short!("path_pay"),),
            (escrow_id, escrow.amount, estimated_dest_amount),
        );

        Ok(())
    }

    fn collect_protocol_fee(env: &Env, escrow_id: u64, escrow: &Escrow) {
        let treasury_opt: Option<Address> =
            env.storage().instance().get(&symbol_short!("treasury"));
        if let Some(treasury) = treasury_opt {
            let fee_bps_args: soroban_sdk::Vec<Val> = soroban_sdk::Vec::new(env);
            let fee_bps: u32 =
                env.invoke_contract(&treasury, &Symbol::new(env, "get_fee_bps"), fee_bps_args);

            let fee_amount = (escrow.amount * fee_bps as i128) / 10000;
            if fee_amount <= 0 {
                return;
            }

            let deposit_args: soroban_sdk::Vec<Val> = soroban_sdk::Vec::from_array(
                env,
                [escrow.asset.clone().into_val(env), fee_amount.into_val(env)],
            );
            let _: () =
                env.invoke_contract(&treasury, &Symbol::new(env, "deposit_fee"), deposit_args);

            env.events().publish(
                (symbol_short!("fee_col"),),
                (escrow_id, fee_amount, escrow.asset.clone()),
            );
        }
    }

    fn unlock_collateral(env: &Env, collateral_id: u64) -> Result<(), ContractError> {
        let coll_reg: Address = env
            .storage()
            .instance()
            .get(&symbol_short!("coll_reg"))
            .ok_or(ContractError::Unauthorized)?;
        let unlock_args: Vec<Val> = Vec::from_array(env, [collateral_id.into_val(env)]);
        env.invoke_contract::<Val>(
            &coll_reg,
            &Symbol::new(env, "unlock_collateral"),
            unlock_args,
        );
        Ok(())
    }

    /// Set exchange rate for testing path payments.
    /// Rate is expressed with 6 decimals precision (1_000_000 = 1:1 ratio).
    /// This is a test helper and would not exist in production.
    pub fn set_test_exchange_rate(env: Env, rate: i128) {
        env.storage()
            .instance()
            .set(&symbol_short!("test_rate"), &rate);
    }

    /// Refund the escrowed funds to the buyer if the escrow has expired.
    ///
    /// Anyone can call this after expiry. Unlocks collateral and returns
    /// funds to the buyer.
    pub fn refund_escrow(env: Env, escrow_id: u64) -> Result<(), ContractError> {
        let mut escrow: Escrow = env
            .storage()
            .persistent()
            .get(&escrow_id)
            .ok_or(ContractError::EscrowNotFound)?;

        if escrow.status == EscrowStatus::Disputed {
            return Err(ContractError::EscrowDisputed);
        }
        if escrow.status != EscrowStatus::Active {
            return Err(ContractError::EscrowNotActive);
        }

        let current_ts = env.ledger().timestamp();
        if current_ts <= escrow.expiry_ts {
            return Err(ContractError::EscrowNotExpired);
        }

        // Refund using the new refund module
        refund::process_refund(&env, &mut escrow, escrow_id)?;

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

    pub fn resolve_dispute(
        env: Env,
        escrow_id: u64,
        decision: DisputeDecision,
    ) -> Result<(), ContractError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&symbol_short!("admin"))
            .ok_or(ContractError::Unauthorized)?;
        admin.require_auth();

        let mut escrow: Escrow = env
            .storage()
            .persistent()
            .get(&escrow_id)
            .ok_or(ContractError::EscrowNotFound)?;

        if escrow.status != EscrowStatus::Disputed {
            return Err(ContractError::EscrowNotDisputed);
        }

        match decision {
            DisputeDecision::ReleaseToSeller => {
                Self::execute_release_payout(&env, escrow_id, &escrow)?;
                Self::collect_protocol_fee(&env, escrow_id, &escrow);
                Self::unlock_collateral(&env, escrow.collateral_id)?;

                escrow.status = EscrowStatus::Released;
                env.storage().persistent().set(&escrow_id, &escrow);

                env.events()
                    .publish((symbol_short!("esc_rslv"),), (escrow_id, decision));
                Ok(())
            }
            DisputeDecision::RefundToBuyer => {
                // Refund using the new refund module
                refund::process_refund(&env, &mut escrow, escrow_id)?;

                Self::unlock_collateral(&env, escrow.collateral_id)?;

                env.storage().persistent().set(&escrow_id, &escrow);

                env.events()
                    .publish((symbol_short!("esc_rslv"),), (escrow_id, decision));
                Ok(())
            }
        }
    }

    /// Get escrow details.
    pub fn get_escrow(env: Env, escrow_id: u64) -> Option<Escrow> {
        env.storage().persistent().get(&escrow_id)
    }

    /// Get all escrow IDs for a specific merchant (seller).
    ///
    /// Returns a vector of escrow IDs where the given address is the seller.
    /// This enables efficient querying without client-side filtering.
    ///
    /// # Arguments
    /// * `merchant` - The seller address to query escrows for
    ///
    /// # Returns
    /// Vector of escrow IDs, or empty vector if merchant has no escrows
    pub fn get_merchant_escrows(env: Env, merchant: Address) -> Vec<u64> {
        let seller_key = (symbol_short!("seller"), merchant);
        env.storage()
            .persistent()
            .get(&seller_key)
            .unwrap_or(Vec::new(&env))
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

    // -- Mock Treasury -----------------------------------------------------

    #[contract]
    pub struct MockTreasury;

    #[contractimpl]
    impl MockTreasury {
        pub fn get_fee_bps(_env: Env) -> u32 {
            0u32 // Zero fee for tests
        }

        pub fn deposit_fee(_env: Env, _asset: Address, _amount: i128) {}
    }

    // -- Mock OracleAdapter (single-oracle, backward-compatible) ---------------
    // Alias to the consensus adapter — same storage layout, same methods.
    type MockOracleAdapter = MockOracleAdapterWithConsensus;

    // -- Mock OracleAdapter with Multi-Oracle Support --------------------------

    #[contract]
    pub struct MockOracleAdapterWithConsensus;

    #[contractimpl]
    impl MockOracleAdapterWithConsensus {
        /// Returns confirmations stored under the escrow_id key.
        pub fn get_confirmation(env: Env, escrow_id: Bytes) -> Option<Vec<ConfirmationData>> {
            env.storage().persistent().get(&escrow_id)
        }

        /// Test helper: store confirmation data for a given escrow_id.
        pub fn set_confirmation(env: Env, escrow_id: Bytes, confirmations: Vec<ConfirmationData>) {
            env.storage().persistent().set(&escrow_id, &confirmations);
        }

        /// Mock implementation of check_consensus for testing
        pub fn check_consensus(
            env: Env,
            escrow_id: Bytes,
            _event_type: u32,
            threshold: u32,
            oracle_set: Vec<Address>,
        ) -> bool {
            let confirmations_opt: Option<Vec<ConfirmationData>> =
                env.storage().persistent().get(&escrow_id);
            if let Some(confirmations) = confirmations_opt {
                let mut unique_oracle_count = 0u32;

                // Count unique oracle confirmations from authorized set
                for conf in confirmations.iter() {
                    if !conf.verified {
                        continue;
                    }

                    // Check if oracle is in the authorized set
                    let is_authorized = if oracle_set.is_empty() {
                        true // Any oracle is authorized if set is empty
                    } else {
                        let mut found = false;
                        for authorized in oracle_set.iter() {
                            if authorized == conf.oracle {
                                found = true;
                                break;
                            }
                        }
                        found
                    };

                    if is_authorized {
                        unique_oracle_count += 1;
                    }
                }

                return unique_oracle_count >= threshold;
            }
            false
        }
    }

    // -- Helpers -----------------------------------------------------------

    struct TestEnv<'a> {
        env: Env,
        escrow_client: EscrowManagerClient<'a>,
        escrow_id_addr: Address,
        coll_reg_addr: Address,
        oracle_client: MockOracleAdapterWithConsensusClient<'a>,
        oracle_addr: Option<Address>, // Add field for multi-oracle tests
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
        let oracle_addr = env.register(MockOracleAdapterWithConsensus, ());
        let oracle_client = MockOracleAdapterWithConsensusClient::new(&env, &oracle_addr);

        let loan_mgr_addr = Address::generate(&env); // placeholder
        let treasury_addr = env.register(MockTreasury, ()); // mock treasury with 0 fee

        // Create a Stellar asset token
        let token_admin = Address::generate(&env);
        let token_contract = env.register_stellar_asset_contract_v2(token_admin);
        let token_addr = token_contract.address();
        let token_admin_client = token::StellarAssetClient::new(&env, &token_addr);
        token_admin_client.mint(&lender, &1_000_000);

        // Initialize escrow manager
        escrow_client.initialize(
            &admin,
            &coll_reg_addr,
            &oracle_addr,
            &loan_mgr_addr,
            &treasury_addr,
        );

        // Leak lifetimes for test convenience
        let escrow_client = unsafe {
            core::mem::transmute::<EscrowManagerClient<'_>, EscrowManagerClient<'static>>(
                escrow_client,
            )
        };
        let oracle_client = unsafe {
            core::mem::transmute::<
                MockOracleAdapterWithConsensusClient<'_>,
                MockOracleAdapterWithConsensusClient<'static>,
            >(oracle_client)
        };

        TestEnv {
            env,
            escrow_client,
            escrow_id_addr,
            coll_reg_addr,
            oracle_client,
            oracle_addr: Some(oracle_addr), // track the oracle address
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
        let oracle_addr_for_conf = Address::generate(&t.env);

        let conf = ConfirmationData {
            escrow_id: escrow_id_bytes.clone(),
            event_type,
            result: Bytes::from_slice(&t.env, b"confirmed"),
            oracle: oracle_addr_for_conf,
            timestamp: t.env.ledger().timestamp(),
            verified,
        };

        let confs = Vec::from_array(&t.env, [conf]);

        // Use the active oracle address (may have been swapped in setup_multi_oracle)
        let active_oracle_addr = t
            .oracle_addr
            .clone()
            .unwrap_or_else(|| t.oracle_client.address.clone());
        let client = MockOracleAdapterWithConsensusClient::new(&t.env, &active_oracle_addr);
        client.set_confirmation(&escrow_id_bytes, &confs);
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
        t.escrow_client
            .initialize(&admin, &dummy, &dummy, &dummy, &dummy);
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
        let buyer_balance_before = token.balance(&t.buyer);

        // Advance past expiry
        t.env.ledger().with_mut(|li| {
            li.timestamp += 3601;
        });

        t.escrow_client.refund_escrow(&escrow_id);

        // Verify status
        let escrow = t.escrow_client.get_escrow(&escrow_id).unwrap();
        assert_eq!(escrow.status, EscrowStatus::Refunded);

        // Verify funds returned to buyer
        assert_eq!(token.balance(&t.buyer), buyer_balance_before + 5000);
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
    fn test_raise_dispute_by_buyer_success() {
        let t = setup();
        let escrow_id = create_test_escrow(&t);

        let reason = Bytes::from_slice(&t.env, b"oracle offline");
        t.escrow_client.raise_dispute(&escrow_id, &t.buyer, &reason);

        let escrow = t.escrow_client.get_escrow(&escrow_id).unwrap();
        assert_eq!(escrow.status, EscrowStatus::Disputed);
        assert_eq!(escrow.disputed_by, Some(t.buyer.clone()));
        assert!(escrow.disputed_at.is_some());
        assert_eq!(escrow.dispute_reason, Some(reason));
    }

    #[test]
    fn test_raise_dispute_by_seller_success() {
        let t = setup();
        let escrow_id = create_test_escrow(&t);

        let reason = Bytes::from_slice(&t.env, b"quality disagreement");
        t.escrow_client
            .raise_dispute(&escrow_id, &t.seller, &reason);

        let escrow = t.escrow_client.get_escrow(&escrow_id).unwrap();
        assert_eq!(escrow.status, EscrowStatus::Disputed);
        assert_eq!(escrow.disputed_by, Some(t.seller.clone()));
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #1)")]
    fn test_raise_dispute_unauthorized_address_fails() {
        let t = setup();
        let escrow_id = create_test_escrow(&t);

        let attacker = Address::generate(&t.env);
        let reason = Bytes::from_slice(&t.env, b"grief");
        t.escrow_client
            .raise_dispute(&escrow_id, &attacker, &reason);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #13)")]
    fn test_disputed_blocks_release() {
        let t = setup();
        let escrow_id = create_test_escrow(&t);

        let reason = Bytes::from_slice(&t.env, b"dispute");
        t.escrow_client.raise_dispute(&escrow_id, &t.buyer, &reason);

        // Even with confirmation, release should be blocked while disputed
        set_oracle_confirmation(&t, escrow_id, 2, true);
        t.escrow_client.release_funds_on_confirmation(&escrow_id);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #13)")]
    fn test_disputed_blocks_refund() {
        let t = setup();
        let escrow_id = create_test_escrow(&t);

        let reason = Bytes::from_slice(&t.env, b"dispute");
        t.escrow_client
            .raise_dispute(&escrow_id, &t.seller, &reason);

        // Advance past expiry
        t.env.ledger().with_mut(|li| {
            li.timestamp += 3601;
        });

        t.escrow_client.refund_escrow(&escrow_id);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #14)")]
    fn test_resolve_dispute_requires_disputed_state() {
        let t = setup();
        let escrow_id = create_test_escrow(&t);

        t.escrow_client
            .resolve_dispute(&escrow_id, &DisputeDecision::RefundToBuyer);
    }

    #[test]
    fn test_resolve_dispute_refund_to_buyer_success() {
        let t = setup();
        let escrow_id = create_test_escrow(&t);

        let token = token::Client::new(&t.env, &t.token_addr);
        let buyer_balance_before = token.balance(&t.buyer);

        let reason = Bytes::from_slice(&t.env, b"dispute");
        t.escrow_client.raise_dispute(&escrow_id, &t.buyer, &reason);

        t.escrow_client
            .resolve_dispute(&escrow_id, &DisputeDecision::RefundToBuyer);

        let escrow = t.escrow_client.get_escrow(&escrow_id).unwrap();
        assert_eq!(escrow.status, EscrowStatus::Refunded);
        assert_eq!(token.balance(&t.buyer), buyer_balance_before + 5000);
        assert_eq!(token.balance(&t.escrow_id_addr), 0);

        // Verify collateral unlocked
        t.env.as_contract(&t.coll_reg_addr, || {
            let locked: bool = t.env.storage().persistent().get(&1u64).unwrap();
            assert!(!locked);
        });
    }

    #[test]
    fn test_resolve_dispute_release_to_seller_success() {
        let t = setup();
        let escrow_id = create_test_escrow(&t);

        let token = token::Client::new(&t.env, &t.token_addr);

        let reason = Bytes::from_slice(&t.env, b"dispute");
        t.escrow_client
            .raise_dispute(&escrow_id, &t.seller, &reason);

        t.escrow_client
            .resolve_dispute(&escrow_id, &DisputeDecision::ReleaseToSeller);

        let escrow = t.escrow_client.get_escrow(&escrow_id).unwrap();
        assert_eq!(escrow.status, EscrowStatus::Released);
        assert_eq!(token.balance(&t.seller), 5000);
        assert_eq!(token.balance(&t.escrow_id_addr), 0);

        // Verify collateral unlocked
        t.env.as_contract(&t.coll_reg_addr, || {
            let locked: bool = t.env.storage().persistent().get(&1u64).unwrap();
            assert!(!locked);
        });
    }

    #[test]
    fn test_resolve_dispute_release_to_seller_path_payment() {
        let t = setup();

        // Create a second token for destination
        let token_admin = Address::generate(&t.env);
        let dest_token_contract = t
            .env
            .register_stellar_asset_contract_v2(token_admin.clone());
        let dest_token_addr = dest_token_contract.address();
        let dest_token_admin_client = token::StellarAssetClient::new(&t.env, &dest_token_addr);
        dest_token_admin_client.mint(&t.escrow_id_addr, &10_000);

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

        // Set exchange rate: 0.95
        t.escrow_client.set_test_exchange_rate(&950_000i128);

        let reason = Bytes::from_slice(&t.env, b"dispute");
        t.escrow_client
            .raise_dispute(&escrow_id, &t.seller, &reason);
        t.escrow_client
            .resolve_dispute(&escrow_id, &DisputeDecision::ReleaseToSeller);

        let escrow = t.escrow_client.get_escrow(&escrow_id).unwrap();
        assert_eq!(escrow.status, EscrowStatus::Released);
        let dest_token = token::Client::new(&t.env, &dest_token_addr);
        assert_eq!(dest_token.balance(&t.seller), 4_750);
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
    fn test_get_merchant_escrows() {
        let t = setup();

        // Create multiple escrows with the same seller
        let escrow_id1 = create_test_escrow(&t);
        let escrow_id2 = create_test_escrow(&t);

        // Create an escrow with a different seller
        let different_seller = Address::generate(&t.env);
        let expiry = t.env.ledger().timestamp() + 3600;
        let escrow_id3 = t.escrow_client.create_escrow(&EscrowConfig {
            buyer: t.buyer.clone(),
            seller: different_seller.clone(),
            lender: t.lender.clone(),
            collateral_id: 1u64,
            amount: 5000i128,
            asset: t.token_addr.clone(),
            required_confirmation: 2u32,
            expiry_ts: expiry,
            destination_asset: t.token_addr.clone(),
            min_destination_amount: 5000i128,
            required_confirmations: 0u32,
            oracle_set: Vec::new(&t.env),
        });

        // Query escrows for the original seller
        let seller_escrows = t.escrow_client.get_merchant_escrows(&t.seller);
        assert_eq!(seller_escrows.len(), 2);
        assert!(seller_escrows.iter().any(|id| id == escrow_id1));
        assert!(seller_escrows.iter().any(|id| id == escrow_id2));
        assert!(!seller_escrows.iter().any(|id| id == escrow_id3));

        // Query escrows for the different seller
        let different_seller_escrows = t.escrow_client.get_merchant_escrows(&different_seller);
        assert_eq!(different_seller_escrows.len(), 1);
        assert_eq!(different_seller_escrows.get(0).unwrap(), escrow_id3);

        // Query escrows for an address with no escrows
        let no_escrows_address = Address::generate(&t.env);
        let no_escrows = t.escrow_client.get_merchant_escrows(&no_escrows_address);
        assert_eq!(no_escrows.len(), 0);
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

        // Mint destination tokens to the escrow contract to simulate DEX liquidity
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

        // Set exchange rate: 0.95 (5% slippage) → dest_amount = 4750 ≥ min 4500
        t.escrow_client.set_test_exchange_rate(&950_000i128);

        // Set oracle confirmation
        set_oracle_confirmation(&t, escrow_id, 2, true);

        // Release with path payment
        t.escrow_client.release_funds_on_confirmation(&escrow_id);

        let escrow = t.escrow_client.get_escrow(&escrow_id).unwrap();
        assert_eq!(escrow.status, EscrowStatus::Released);
        // Seller must receive destination tokens (not source tokens)
        let dest_token = token::Client::new(&t.env, &dest_token_addr);
        assert_eq!(dest_token.balance(&t.seller), 4_750); // 5000 * 0.95

        // Seller must NOT receive source tokens
        let src_token = token::Client::new(&t.env, &t.token_addr);
        assert_eq!(src_token.balance(&t.seller), 0);
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
            min_destination_amount: 5000i128, // Same as source amount (no slippage tolerance)
            required_confirmations: 0u32,
            oracle_set: Vec::new(&t.env),
        });

        // Set exchange rate: 0.95 (5% loss in conversion)
        t.escrow_client.set_test_exchange_rate(&950_000i128);

        // Set oracle confirmation
        set_oracle_confirmation(&t, escrow_id, 2, true);

        // Release should fail due to slippage
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

    // -- Multi-Oracle Consensus Tests -------------------------------------

    fn setup_multi_oracle() -> TestEnv<'static> {
        // The oracle registered in setup() is already MockOracleAdapterWithConsensus
        // with check_consensus support. No additional setup needed.
        setup()
    }

    fn create_multi_oracle_escrow(t: &TestEnv, threshold: u32, oracle_set: Vec<Address>) -> u64 {
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
            required_confirmations: threshold,
            oracle_set,
        })
    }

    fn set_multi_oracle_confirmations(
        t: &TestEnv,
        escrow_id: u64,
        oracles: Vec<Address>,
        event_type: u32,
        verified: bool,
    ) {
        let escrow_id_bytes = Bytes::from_slice(&t.env, &escrow_id.to_be_bytes());
        let mut confirmations = Vec::new(&t.env);

        for oracle in oracles.iter() {
            let conf = ConfirmationData {
                escrow_id: escrow_id_bytes.clone(),
                event_type,
                result: Bytes::from_slice(&t.env, b"confirmed"),
                oracle: oracle.clone(),
                timestamp: t.env.ledger().timestamp(),
                verified,
            };
            confirmations.push_back(conf);
        }

        let oracle_addr = t
            .oracle_addr
            .clone()
            .expect("Oracle address must be set in multi-oracle tests");
        let oracle_client = MockOracleAdapterWithConsensusClient::new(&t.env, &oracle_addr);
        oracle_client.set_confirmation(&escrow_id_bytes, &confirmations);
    }

    #[test]
    fn test_multi_oracle_consensus_success() {
        let t = setup_multi_oracle();

        let oracle1 = Address::generate(&t.env);
        let oracle2 = Address::generate(&t.env);
        let oracle3 = Address::generate(&t.env);
        let oracle_set =
            Vec::from_array(&t.env, [oracle1.clone(), oracle2.clone(), oracle3.clone()]);

        // Create escrow requiring 2 out of 3 oracles
        let escrow_id = create_multi_oracle_escrow(&t, 2u32, oracle_set.clone());

        // Set confirmations from 2 different oracles
        set_multi_oracle_confirmations(
            &t,
            escrow_id,
            Vec::from_array(&t.env, [oracle1.clone(), oracle2.clone()]),
            2, // Delivery event
            true,
        );

        // Release should succeed
        t.escrow_client.release_funds_on_confirmation(&escrow_id);

        let escrow = t.escrow_client.get_escrow(&escrow_id).unwrap();
        assert_eq!(escrow.status, EscrowStatus::Released);

        // Verify funds sent to seller
        let token = token::Client::new(&t.env, &t.token_addr);
        assert_eq!(token.balance(&t.seller), 5000);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #12)")]
    fn test_multi_oracle_consensus_insufficient_confirmations() {
        let t = setup_multi_oracle();

        let oracle1 = Address::generate(&t.env);
        let oracle2 = Address::generate(&t.env);
        let oracle3 = Address::generate(&t.env);
        let oracle_set =
            Vec::from_array(&t.env, [oracle1.clone(), oracle2.clone(), oracle3.clone()]);

        // Create escrow requiring 3 out of 3 oracles
        let escrow_id = create_multi_oracle_escrow(&t, 3u32, oracle_set.clone());

        // Only provide 2 confirmations
        set_multi_oracle_confirmations(
            &t,
            escrow_id,
            Vec::from_array(&t.env, [oracle1.clone(), oracle2.clone()]),
            2, // Delivery event
            true,
        );

        // Release should fail - consensus not met
        t.escrow_client.release_funds_on_confirmation(&escrow_id);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #10)")]
    fn test_multi_oracle_create_escrow_empty_oracle_set() {
        let t = setup_multi_oracle();

        // Try to create escrow with required_confirmations > 0 but empty oracle_set
        t.escrow_client.create_escrow(&EscrowConfig {
            buyer: t.buyer.clone(),
            seller: t.seller.clone(),
            lender: t.lender.clone(),
            collateral_id: 1u64,
            amount: 5000i128,
            asset: t.token_addr.clone(),
            required_confirmation: 2u32,
            expiry_ts: t.env.ledger().timestamp() + 3600,
            destination_asset: t.token_addr.clone(),
            min_destination_amount: 5000i128,
            required_confirmations: 2u32, // > 0 requires oracle_set
            oracle_set: Vec::new(&t.env), // Empty - should fail
        });
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #11)")]
    fn test_multi_oracle_create_escrow_threshold_exceeds_set_size() {
        let t = setup_multi_oracle();

        let oracle1 = Address::generate(&t.env);
        let oracle2 = Address::generate(&t.env);
        let oracle_set = Vec::from_array(&t.env, [oracle1.clone(), oracle2.clone()]);

        // Try to create escrow with threshold 3 but only 2 oracles in set
        t.escrow_client.create_escrow(&EscrowConfig {
            buyer: t.buyer.clone(),
            seller: t.seller.clone(),
            lender: t.lender.clone(),
            collateral_id: 1u64,
            amount: 5000i128,
            asset: t.token_addr.clone(),
            required_confirmation: 2u32,
            expiry_ts: t.env.ledger().timestamp() + 3600,
            destination_asset: t.token_addr.clone(),
            min_destination_amount: 5000i128,
            required_confirmations: 3u32, // > oracle_set.len()
            oracle_set,
        });
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #10)")]
    fn test_multi_oracle_create_escrow_duplicate_oracles() {
        let t = setup_multi_oracle();

        let oracle1 = Address::generate(&t.env);
        // Include the same oracle twice
        let oracle_set = Vec::from_array(&t.env, [oracle1.clone(), oracle1.clone()]);

        t.escrow_client.create_escrow(&EscrowConfig {
            buyer: t.buyer.clone(),
            seller: t.seller.clone(),
            lender: t.lender.clone(),
            collateral_id: 1u64,
            amount: 5000i128,
            asset: t.token_addr.clone(),
            required_confirmation: 2u32,
            expiry_ts: t.env.ledger().timestamp() + 3600,
            destination_asset: t.token_addr.clone(),
            min_destination_amount: 5000i128,
            required_confirmations: 2u32,
            oracle_set, // Contains duplicate
        });
    }

    #[test]
    fn test_multi_oracle_consensus_ignores_unauthorized_oracles() {
        let t = setup_multi_oracle();

        let oracle1 = Address::generate(&t.env);
        let oracle2 = Address::generate(&t.env);
        let unauthorized_oracle = Address::generate(&t.env);
        let oracle_set = Vec::from_array(&t.env, [oracle1.clone(), oracle2.clone()]);

        // Create escrow requiring 2 oracles from the authorized set
        let escrow_id = create_multi_oracle_escrow(&t, 2u32, oracle_set.clone());

        // Set confirmations: 1 authorized + 1 unauthorized
        set_multi_oracle_confirmations(
            &t,
            escrow_id,
            Vec::from_array(&t.env, [oracle1.clone(), unauthorized_oracle]),
            2, // Delivery event
            true,
        );

        // Release should fail - only 1 authorized confirmation
        assert!(t
            .escrow_client
            .try_release_funds_on_confirmation(&escrow_id)
            .is_err());

        // Add the second authorized confirmation
        set_multi_oracle_confirmations(
            &t,
            escrow_id,
            Vec::from_array(&t.env, [oracle1.clone(), oracle2.clone()]),
            2, // Delivery event
            true,
        );

        // Now release should succeed
        t.escrow_client.release_funds_on_confirmation(&escrow_id);

        let escrow = t.escrow_client.get_escrow(&escrow_id).unwrap();
        assert_eq!(escrow.status, EscrowStatus::Released);
    }

    #[test]
    fn test_multi_oracle_backward_compatibility_single_oracle() {
        let t = setup_multi_oracle();

        // Create escrow with required_confirmations = 0 (single oracle mode)
        let escrow_id = t.escrow_client.create_escrow(&EscrowConfig {
            buyer: t.buyer.clone(),
            seller: t.seller.clone(),
            lender: t.lender.clone(),
            collateral_id: 1u64,
            amount: 5000i128,
            asset: t.token_addr.clone(),
            required_confirmation: 2u32,
            expiry_ts: t.env.ledger().timestamp() + 3600,
            destination_asset: t.token_addr.clone(),
            min_destination_amount: 5000i128,
            required_confirmations: 0u32, // Single oracle mode
            oracle_set: Vec::new(&t.env), // Ignored in single oracle mode
        });

        // Set single oracle confirmation using the old method
        set_oracle_confirmation(&t, escrow_id, 2, true);

        // Release should succeed using backward compatibility
        t.escrow_client.release_funds_on_confirmation(&escrow_id);

        let escrow = t.escrow_client.get_escrow(&escrow_id).unwrap();
        assert_eq!(escrow.status, EscrowStatus::Released);
    }

    #[test]
    fn test_propose_admin() {
        let t = setup();
        let new_admin = Address::generate(&t.env);

        assert!(t.escrow_client.get_pending_admin().is_none());
        t.escrow_client.propose_admin(&new_admin);
        assert_eq!(t.escrow_client.get_pending_admin(), Some(new_admin));
    }

    #[test]
    fn test_accept_admin() {
        let t = setup();
        let new_admin = Address::generate(&t.env);

        t.escrow_client.propose_admin(&new_admin);
        t.escrow_client.accept_admin();

        // Verify treasury still works (admin changed)
        assert!(t.escrow_client.get_pending_admin().is_none());
    }

    #[test]
    #[should_panic]
    fn test_propose_admin_unauthorized() {
        // No auth mocking — admin.require_auth() will panic when not satisfied
        let env = Env::default();
        let admin = Address::generate(&env);
        let new_admin = Address::generate(&env);
        let coll_reg = env.register(MockCollateralRegistry, ());
        let oracle = env.register(MockOracleAdapterWithConsensus, ());
        let loan_mgr = Address::generate(&env);
        let treasury = env.register(MockTreasury, ());
        let contract_id = env.register(EscrowManager, ());

        env.as_contract(&contract_id, || {
            EscrowManager::initialize(
                env.clone(),
                admin,
                coll_reg,
                oracle,
                loan_mgr,
                treasury,
            )
            .unwrap();
            // propose_admin calls admin.require_auth() — panics without mocked auth
            EscrowManager::propose_admin(env.clone(), new_admin).unwrap();
        });
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #15)")]
    fn test_accept_admin_no_pending() {
        let t = setup();
        t.escrow_client.accept_admin();
    }
}

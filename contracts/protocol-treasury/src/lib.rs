//! Protocol Treasury Contract for StelloVault
//!
//! Collects protocol fees from loan repayments and escrow releases,
//! distributes dividends to registered contributors based on share weights.

#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, token, Address, Env};

/// Default protocol fee in basis points (50 = 0.5%)
const DEFAULT_FEE_BPS: u32 = 50;

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

        env.events()
            .publish((symbol_short!("trs_init"),), (admin, DEFAULT_FEE_BPS));

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
        env.storage()
            .persistent()
            .set(&key, &(current + amount));

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
}

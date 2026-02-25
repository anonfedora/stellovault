//! Reputation Registry Contract for StelloVault
//!
//! This contract tracks long-term SME behavior (repayments, shipment accuracy, dispute history)
//! to influence future risk scores and interest rates. Good behavior leads to better borrowing terms.

#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContractError {
    Unauthorized = 1,
    AlreadyInitialized = 2,
    ProfileNotFound = 3,
    InvalidValue = 4,
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

/// Reputation profile tracking SME behavior over time
#[contracttype]
#[derive(Clone, Debug)]
pub struct ReputationProfile {
    pub user: Address,
    pub successful_trades: u32,
    pub total_volume: i128,
    pub defaults: u32,
    pub disputes_lost: u32,
    pub early_repayments: u32,
    pub on_time_repayments: u32,
    pub late_repayments: u32,
    pub created_at: u64,
    pub last_updated: u64,
}

/// Event types for reputation updates
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReputationEvent {
    TradeCompleted = 1,
    EarlyRepayment = 2,
    OnTimeRepayment = 3,
    LateRepayment = 4,
    Default = 5,
    DisputeLost = 6,
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct ReputationRegistry;

#[contractimpl]
impl ReputationRegistry {
    /// Initialize the contract with admin and authorized contract addresses
    pub fn initialize(
        env: Env,
        admin: Address,
        escrow_manager: Address,
        loan_management: Address,
    ) -> Result<(), ContractError> {
        if env.storage().instance().has(&symbol_short!("admin")) {
            return Err(ContractError::AlreadyInitialized);
        }

        env.storage()
            .instance()
            .set(&symbol_short!("admin"), &admin);
        env.storage()
            .instance()
            .set(&symbol_short!("esc_mgr"), &escrow_manager);
        env.storage()
            .instance()
            .set(&symbol_short!("loan_mgr"), &loan_management);

        env.events().publish((symbol_short!("rep_init"),), (admin,));

        Ok(())
    }

    /// Create or get reputation profile for a user
    pub fn get_or_create_profile(env: Env, user: Address) -> ReputationProfile {
        if let Some(profile) = env
            .storage()
            .persistent()
            .get::<Address, ReputationProfile>(&user)
        {
            profile
        } else {
            let profile = ReputationProfile {
                user: user.clone(),
                successful_trades: 0,
                total_volume: 0,
                defaults: 0,
                disputes_lost: 0,
                early_repayments: 0,
                on_time_repayments: 0,
                late_repayments: 0,
                created_at: env.ledger().timestamp(),
                last_updated: env.ledger().timestamp(),
            };
            env.storage().persistent().set(&user, &profile);
            profile
        }
    }

    /// Update reputation based on lifecycle events
    /// Only callable by authorized contracts (EscrowManager, LoanManagement)
    pub fn record_event(
        env: Env,
        caller: Address,
        user: Address,
        event_type: ReputationEvent,
        volume: i128,
    ) -> Result<(), ContractError> {
        // Verify caller is authorized
        Self::require_authorized_caller(&env, &caller)?;

        if volume < 0 {
            return Err(ContractError::InvalidValue);
        }

        let mut profile = Self::get_or_create_profile(env.clone(), user.clone());

        // Update profile based on event type
        match event_type {
            ReputationEvent::TradeCompleted => {
                profile.successful_trades += 1;
                profile.total_volume = profile.total_volume.saturating_add(volume);
            }
            ReputationEvent::EarlyRepayment => {
                profile.early_repayments += 1;
                profile.on_time_repayments += 1;
            }
            ReputationEvent::OnTimeRepayment => {
                profile.on_time_repayments += 1;
            }
            ReputationEvent::LateRepayment => {
                profile.late_repayments += 1;
            }
            ReputationEvent::Default => {
                profile.defaults += 1;
            }
            ReputationEvent::DisputeLost => {
                profile.disputes_lost += 1;
            }
        }

        profile.last_updated = env.ledger().timestamp();
        env.storage().persistent().set(&user, &profile);

        env.events()
            .publish((symbol_short!("rep_evt"),), (user, event_type, volume));

        Ok(())
    }

    /// Calculate reputation multiplier for risk assessment
    /// Returns a multiplier in basis points (10000 = 1.0x, 8000 = 0.8x, 12000 = 1.2x)
    /// Lower multiplier = better reputation = lower collateral requirement
    pub fn get_reputation_multiplier(env: Env, user: Address) -> u32 {
        let profile = match env
            .storage()
            .persistent()
            .get::<Address, ReputationProfile>(&user)
        {
            Some(p) => p,
            None => return 10000, // Neutral multiplier for new users
        };

        // Base multiplier
        let mut multiplier: i32 = 10000;

        // Positive factors (reduce collateral requirement)
        // Each successful trade reduces by 10 bps (max 500 bps reduction)
        let trade_bonus = (profile.successful_trades as i32 * 10).min(500);
        multiplier -= trade_bonus;

        // Early repayments reduce by 20 bps each (max 400 bps)
        let early_bonus = (profile.early_repayments as i32 * 20).min(400);
        multiplier -= early_bonus;

        // On-time repayments reduce by 5 bps each (max 300 bps)
        let ontime_bonus = (profile.on_time_repayments as i32 * 5).min(300);
        multiplier -= ontime_bonus;

        // Volume bonus: for every 100k in volume, reduce by 50 bps (max 500 bps)
        let volume_tiers = (profile.total_volume / 100_000_000_000) as i32; // Assuming 7 decimals
        let volume_bonus = (volume_tiers * 50).min(500);
        multiplier -= volume_bonus;

        // Negative factors (increase collateral requirement)
        // Each default increases by 500 bps
        let default_penalty = profile.defaults as i32 * 500;
        multiplier += default_penalty;

        // Each dispute lost increases by 300 bps
        let dispute_penalty = profile.disputes_lost as i32 * 300;
        multiplier += dispute_penalty;

        // Late repayments increase by 50 bps each
        let late_penalty = profile.late_repayments as i32 * 50;
        multiplier += late_penalty;

        // Ensure multiplier stays within reasonable bounds (50% to 200%)
        multiplier = multiplier.clamp(5000, 20000);

        multiplier as u32
    }

    /// Get reputation score (0-1000 scale for display purposes)
    pub fn get_reputation_score(env: Env, user: Address) -> u32 {
        let profile = match env
            .storage()
            .persistent()
            .get::<Address, ReputationProfile>(&user)
        {
            Some(p) => p,
            None => return 500, // Neutral score for new users
        };

        // Base score
        let mut score: i32 = 500;

        // Positive contributions
        score += (profile.successful_trades as i32 * 5).min(200);
        score += (profile.early_repayments as i32 * 10).min(150);
        score += (profile.on_time_repayments as i32 * 3).min(100);

        // Negative contributions
        score -= profile.defaults as i32 * 100;
        score -= profile.disputes_lost as i32 * 50;
        score -= profile.late_repayments as i32 * 10;

        // Clamp between 0 and 1000
        score.clamp(0, 1000) as u32
    }

    /// Slash reputation for severe violations (admin or governance only)
    pub fn slash_reputation(
        env: Env,
        user: Address,
        disputes_to_add: u32,
        defaults_to_add: u32,
    ) -> Result<(), ContractError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&symbol_short!("admin"))
            .ok_or(ContractError::Unauthorized)?;

        admin.require_auth();

        let mut profile = Self::get_or_create_profile(env.clone(), user.clone());

        profile.disputes_lost += disputes_to_add;
        profile.defaults += defaults_to_add;
        profile.last_updated = env.ledger().timestamp();

        env.storage().persistent().set(&user, &profile);

        env.events().publish(
            (symbol_short!("rep_slsh"),),
            (user, disputes_to_add, defaults_to_add),
        );

        Ok(())
    }

    /// Get full reputation profile
    pub fn get_profile(env: Env, user: Address) -> Option<ReputationProfile> {
        env.storage().persistent().get(&user)
    }

    /// Check if caller is authorized (EscrowManager or LoanManagement)
    fn require_authorized_caller(env: &Env, caller: &Address) -> Result<(), ContractError> {
        // Get authorized contract addresses
        let escrow_mgr: Address = env
            .storage()
            .instance()
            .get(&symbol_short!("esc_mgr"))
            .ok_or(ContractError::Unauthorized)?;

        let loan_mgr: Address = env
            .storage()
            .instance()
            .get(&symbol_short!("loan_mgr"))
            .ok_or(ContractError::Unauthorized)?;

        // Check if caller is one of the authorized contracts
        if caller == &escrow_mgr || caller == &loan_mgr {
            caller.require_auth();
            Ok(())
        } else {
            Err(ContractError::Unauthorized)
        }
    }

    /// Update authorized contract addresses (admin only)
    pub fn update_authorized_contracts(
        env: Env,
        escrow_manager: Address,
        loan_management: Address,
    ) -> Result<(), ContractError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&symbol_short!("admin"))
            .ok_or(ContractError::Unauthorized)?;

        admin.require_auth();

        env.storage()
            .instance()
            .set(&symbol_short!("esc_mgr"), &escrow_manager);
        env.storage()
            .instance()
            .set(&symbol_short!("loan_mgr"), &loan_management);

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Address, Env};

    struct TestEnv<'a> {
        env: Env,
        contract_id: Address,
        client: ReputationRegistryClient<'a>,
        admin: Address,
        escrow_mgr: Address,
        loan_mgr: Address,
        user: Address,
    }

    fn setup() -> TestEnv<'static> {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let escrow_mgr = Address::generate(&env);
        let loan_mgr = Address::generate(&env);
        let user = Address::generate(&env);

        let contract_id = env.register_contract(None, ReputationRegistry);
        let client = ReputationRegistryClient::new(&env, &contract_id);

        client.initialize(&admin, &escrow_mgr, &loan_mgr);

        let client = unsafe {
            core::mem::transmute::<ReputationRegistryClient<'_>, ReputationRegistryClient<'static>>(
                client,
            )
        };

        TestEnv {
            env,
            contract_id,
            client,
            admin,
            escrow_mgr,
            loan_mgr,
            user,
        }
    }

    #[test]
    fn test_initialize() {
        let t = setup();

        t.env.as_contract(&t.contract_id, || {
            let admin: Address = t
                .env
                .storage()
                .instance()
                .get(&symbol_short!("admin"))
                .unwrap();
            assert_eq!(admin, t.admin);
        });
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #2)")]
    fn test_initialize_already_initialized() {
        let t = setup();
        let dummy = Address::generate(&t.env);
        t.client.initialize(&dummy, &dummy, &dummy);
    }

    #[test]
    fn test_get_or_create_profile() {
        let t = setup();

        let profile = t.client.get_or_create_profile(&t.user);
        assert_eq!(profile.user, t.user);
        assert_eq!(profile.successful_trades, 0);
        assert_eq!(profile.total_volume, 0);
        assert_eq!(profile.defaults, 0);
    }

    #[test]
    fn test_record_trade_completed() {
        let t = setup();

        // Call from escrow manager
        t.client.record_event(
            &t.escrow_mgr,
            &t.user,
            &ReputationEvent::TradeCompleted,
            &10_000_000_000,
        );

        let profile = t.client.get_profile(&t.user).unwrap();
        assert_eq!(profile.successful_trades, 1);
        assert_eq!(profile.total_volume, 10_000_000_000);
    }

    #[test]
    fn test_record_early_repayment() {
        let t = setup();

        t.client
            .record_event(&t.loan_mgr, &t.user, &ReputationEvent::EarlyRepayment, &0);

        let profile = t.client.get_profile(&t.user).unwrap();
        assert_eq!(profile.early_repayments, 1);
        assert_eq!(profile.on_time_repayments, 1);
    }

    #[test]
    fn test_record_default() {
        let t = setup();

        t.client
            .record_event(&t.loan_mgr, &t.user, &ReputationEvent::Default, &0);

        let profile = t.client.get_profile(&t.user).unwrap();
        assert_eq!(profile.defaults, 1);
    }

    #[test]
    fn test_record_dispute_lost() {
        let t = setup();

        t.client
            .record_event(&t.escrow_mgr, &t.user, &ReputationEvent::DisputeLost, &0);

        let profile = t.client.get_profile(&t.user).unwrap();
        assert_eq!(profile.disputes_lost, 1);
    }

    #[test]
    fn test_record_event_from_authorized_contract() {
        let t = setup();

        // Both escrow_mgr and loan_mgr should be able to record events
        t.client.record_event(
            &t.escrow_mgr,
            &t.user,
            &ReputationEvent::TradeCompleted,
            &1000,
        );

        t.client
            .record_event(&t.loan_mgr, &t.user, &ReputationEvent::OnTimeRepayment, &0);

        let profile = t.client.get_profile(&t.user).unwrap();
        assert_eq!(profile.successful_trades, 1);
        assert_eq!(profile.on_time_repayments, 1);
    }

    #[test]
    fn test_reputation_multiplier_new_user() {
        let t = setup();

        let multiplier = t.client.get_reputation_multiplier(&t.user);
        assert_eq!(multiplier, 10000); // Neutral for new users
    }

    #[test]
    fn test_reputation_multiplier_good_behavior() {
        let t = setup();

        // Record multiple positive events
        for _ in 0..10 {
            t.client.record_event(
                &t.escrow_mgr,
                &t.user,
                &ReputationEvent::TradeCompleted,
                &10_000_000_000,
            );
        }

        for _ in 0..5 {
            t.client
                .record_event(&t.loan_mgr, &t.user, &ReputationEvent::EarlyRepayment, &0);
        }

        let multiplier = t.client.get_reputation_multiplier(&t.user);
        // Should be less than 10000 (better reputation = lower collateral)
        assert!(multiplier < 10000);
        assert!(multiplier >= 5000); // Within bounds
    }

    #[test]
    fn test_reputation_multiplier_bad_behavior() {
        let t = setup();

        // Record negative events
        t.client
            .record_event(&t.loan_mgr, &t.user, &ReputationEvent::Default, &0);
        t.client
            .record_event(&t.loan_mgr, &t.user, &ReputationEvent::LateRepayment, &0);

        t.client
            .record_event(&t.escrow_mgr, &t.user, &ReputationEvent::DisputeLost, &0);

        let multiplier = t.client.get_reputation_multiplier(&t.user);
        // Should be greater than 10000 (worse reputation = higher collateral)
        assert!(multiplier > 10000);
        assert!(multiplier <= 20000); // Within bounds
    }

    #[test]
    fn test_reputation_score_new_user() {
        let t = setup();

        let score = t.client.get_reputation_score(&t.user);
        assert_eq!(score, 500); // Neutral score
    }

    #[test]
    fn test_reputation_score_calculation() {
        let t = setup();

        // Build good reputation
        for _ in 0..20 {
            t.client.record_event(
                &t.escrow_mgr,
                &t.user,
                &ReputationEvent::TradeCompleted,
                &5_000_000_000,
            );
        }

        for _ in 0..10 {
            t.client
                .record_event(&t.loan_mgr, &t.user, &ReputationEvent::EarlyRepayment, &0);
        }

        let score = t.client.get_reputation_score(&t.user);
        assert!(score > 500); // Better than neutral
        assert!(score <= 1000); // Within max
    }

    #[test]
    fn test_slash_reputation() {
        let t = setup();

        // Create initial profile
        t.client.record_event(
            &t.escrow_mgr,
            &t.user,
            &ReputationEvent::TradeCompleted,
            &1000,
        );

        // Slash reputation
        t.client.slash_reputation(&t.user, &2, &1);

        let profile = t.client.get_profile(&t.user).unwrap();
        assert_eq!(profile.disputes_lost, 2);
        assert_eq!(profile.defaults, 1);
    }

    #[test]
    fn test_multiple_users_independent_profiles() {
        let t = setup();
        let user2 = Address::generate(&t.env);

        // Record events for user1
        t.client.record_event(
            &t.escrow_mgr,
            &t.user,
            &ReputationEvent::TradeCompleted,
            &1000,
        );

        // Record events for user2
        t.client
            .record_event(&t.loan_mgr, &user2, &ReputationEvent::Default, &0);

        let profile1 = t.client.get_profile(&t.user).unwrap();
        let profile2 = t.client.get_profile(&user2).unwrap();

        assert_eq!(profile1.successful_trades, 1);
        assert_eq!(profile1.defaults, 0);

        assert_eq!(profile2.successful_trades, 0);
        assert_eq!(profile2.defaults, 1);
    }

    #[test]
    fn test_reputation_multiplier_bounds() {
        let t = setup();

        // Create extreme negative reputation
        for _ in 0..50 {
            t.client
                .record_event(&t.loan_mgr, &t.user, &ReputationEvent::Default, &0);
        }

        let multiplier = t.client.get_reputation_multiplier(&t.user);
        assert_eq!(multiplier, 20000); // Capped at max
    }

    #[test]
    fn test_reputation_score_bounds() {
        let t = setup();

        // Create extreme negative reputation
        for _ in 0..20 {
            t.client
                .record_event(&t.loan_mgr, &t.user, &ReputationEvent::Default, &0);
        }

        let score = t.client.get_reputation_score(&t.user);
        assert_eq!(score, 0); // Capped at min
    }

    #[test]
    fn test_update_authorized_contracts() {
        let t = setup();

        let new_escrow = Address::generate(&t.env);
        let new_loan = Address::generate(&t.env);

        t.client.update_authorized_contracts(&new_escrow, &new_loan);

        // Verify new contracts can call
        t.client.record_event(
            &new_escrow,
            &t.user,
            &ReputationEvent::TradeCompleted,
            &1000,
        );

        let profile = t.client.get_profile(&t.user).unwrap();
        assert_eq!(profile.successful_trades, 1);
    }

    #[test]
    fn test_volume_accumulation() {
        let t = setup();

        t.client.record_event(
            &t.escrow_mgr,
            &t.user,
            &ReputationEvent::TradeCompleted,
            &50_000_000_000,
        );
        t.client.record_event(
            &t.escrow_mgr,
            &t.user,
            &ReputationEvent::TradeCompleted,
            &75_000_000_000,
        );

        let profile = t.client.get_profile(&t.user).unwrap();
        assert_eq!(profile.total_volume, 125_000_000_000);
        assert_eq!(profile.successful_trades, 2);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #4)")]
    fn test_record_event_negative_volume() {
        let t = setup();

        t.client.record_event(
            &t.escrow_mgr,
            &t.user,
            &ReputationEvent::TradeCompleted,
            &-1000,
        );
    }

    #[test]
    fn test_late_vs_ontime_repayments() {
        let t = setup();

        for _ in 0..5 {
            t.client
                .record_event(&t.loan_mgr, &t.user, &ReputationEvent::OnTimeRepayment, &0);
        }
        for _ in 0..2 {
            t.client
                .record_event(&t.loan_mgr, &t.user, &ReputationEvent::LateRepayment, &0);
        }

        let profile = t.client.get_profile(&t.user).unwrap();
        assert_eq!(profile.on_time_repayments, 5);
        assert_eq!(profile.late_repayments, 2);

        let multiplier = t.client.get_reputation_multiplier(&t.user);
        // With 5 on-time (25 bps reduction) and 2 late (100 bps increase), net is +75 bps
        // So multiplier should be slightly above 10000
        assert!(multiplier >= 10000);
        assert!(multiplier < 11000); // But not too high
    }
}

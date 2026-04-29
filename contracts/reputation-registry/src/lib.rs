#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Vec};

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContractError {
    Unauthorized = 1,
    AlreadyInitialized = 2,
    ProfileNotFound = 3,
    InvalidValue = 4,
    NoPendingAdmin = 5,
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

#[contracttype]
#[derive(Clone, Debug)]
pub struct TrustMetrics {
    pub successful_trades: u32,
    pub total_volume: i128,
    pub default_rate: u32,
    pub dispute_rate: u32,
    pub repayment_score: u32,
    pub behavior_score: u32,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct BehaviorData {
    pub action_type: ActionType,
    pub outcome: Outcome,
    pub volume: i128,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActionType {
    Trade = 1,
    Repayment = 2,
    Dispute = 3,
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Outcome {
    Success = 1,
    Failure = 2,
    Early = 3,
    OnTime = 4,
    Late = 5,
    Default = 6,
    Won = 7,
    Lost = 8,
}

#[contract]
pub struct ReputationRegistry;

#[contractimpl]
impl ReputationRegistry {
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

        Ok(())
    }

    fn get_or_create_profile(env: &Env, user: &Address) -> ReputationProfile {
        if let Some(profile) = env
            .storage()
            .persistent()
            .get::<Address, ReputationProfile>(user)
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
            env.storage().persistent().set(user, &profile);
            profile
        }
    }

    pub fn record_behavior(
        env: Env,
        caller: Address,
        user_address: Address,
        behavior_data: BehaviorData,
    ) -> Result<(), ContractError> {
        Self::require_authorized_caller(&env, &caller)?;

        let mut profile = Self::get_or_create_profile(&env, &user_address);

        match behavior_data.action_type {
            ActionType::Trade => {
                if behavior_data.outcome == Outcome::Success {
                    profile.successful_trades += 1;
                    profile.total_volume =
                        profile.total_volume.saturating_add(behavior_data.volume);
                }
            }
            ActionType::Repayment => match behavior_data.outcome {
                Outcome::Early => {
                    profile.early_repayments += 1;
                    profile.on_time_repayments += 1;
                }
                Outcome::OnTime => profile.on_time_repayments += 1,
                Outcome::Late => profile.late_repayments += 1,
                Outcome::Default => profile.defaults += 1,
                _ => {}
            },
            ActionType::Dispute => {
                if behavior_data.outcome == Outcome::Lost {
                    profile.disputes_lost += 1;
                }
            }
        }

        profile.last_updated = env.ledger().timestamp();
        env.storage().persistent().set(&user_address, &profile);

        // Use a composite key or a specialized history symbol for the user history.
        let mut user_history: Vec<BehaviorData> = env
            .storage()
            .persistent()
            .get(&(symbol_short!("hist"), user_address.clone()))
            .unwrap_or_else(|| Vec::new(&env));
        user_history.push_back(behavior_data.clone());
        if user_history.len() > 100 {
            user_history.pop_front();
        }
        env.storage().persistent().set(
            &(symbol_short!("hist"), user_address.clone()),
            &user_history,
        );

        Ok(())
    }

    pub fn update_reputation(
        env: Env,
        caller: Address,
        user_address: Address,
        action_type: ActionType,
        outcome: Outcome,
    ) -> Result<(), ContractError> {
        Self::record_behavior(
            env.clone(),
            caller,
            user_address,
            BehaviorData {
                action_type,
                outcome,
                volume: 0,
                timestamp: env.ledger().timestamp(),
            },
        )
    }

    pub fn calculate_reputation_score(env: Env, user_address: Address) -> u32 {
        let profile = Self::get_or_create_profile(&env, &user_address);
        let mut score: i32 = 500;

        score += (profile.successful_trades as i32 * 5).min(200);
        score += (profile.early_repayments as i32 * 10).min(150);
        score += (profile.on_time_repayments as i32 * 3).min(100);

        score -= profile.defaults as i32 * 100;
        score -= profile.disputes_lost as i32 * 50;
        score -= profile.late_repayments as i32 * 10;

        score.clamp(0, 1000) as u32
    }

    pub fn verify_reputation(env: Env, user_address: Address, minimum_score: u32) -> bool {
        let score = Self::calculate_reputation_score(env, user_address);
        score >= minimum_score
    }

    pub fn get_trust_metrics(env: Env, user_address: Address) -> TrustMetrics {
        let profile = Self::get_or_create_profile(&env, &user_address);
        let total_repayments = profile.early_repayments
            + profile.on_time_repayments
            + profile.late_repayments
            + profile.defaults;

        let repayment_score = {
            let good_repayments = profile.early_repayments + profile.on_time_repayments;
            (good_repayments * 1000)
                .checked_div(total_repayments)
                .unwrap_or(500)
        };

        let total_interactions = profile.successful_trades + profile.disputes_lost;
        let dispute_rate = (profile.disputes_lost * 10000)
            .checked_div(total_interactions)
            .unwrap_or(0);

        TrustMetrics {
            successful_trades: profile.successful_trades,
            total_volume: profile.total_volume,
            default_rate: (profile.defaults * 10000)
                .checked_div(total_repayments)
                .unwrap_or(0),
            dispute_rate,
            repayment_score,
            behavior_score: Self::calculate_reputation_score(env, user_address),
        }
    }

    pub fn reputation_history(env: Env, user_address: Address, period: u64) -> Vec<BehaviorData> {
        let user_history: Vec<BehaviorData> = env
            .storage()
            .persistent()
            .get(&(symbol_short!("hist"), user_address.clone()))
            .unwrap_or_else(|| Vec::new(&env));
        let current_time = env.ledger().timestamp();
        let cutoff_time = current_time.saturating_sub(period);

        let mut filtered_history = Vec::new(&env);
        for item in user_history.iter() {
            if item.timestamp >= cutoff_time {
                filtered_history.push_back(item);
            }
        }
        filtered_history
    }

    fn require_authorized_caller(env: &Env, caller: &Address) -> Result<(), ContractError> {
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
        env.storage().instance().remove(&symbol_short!("pend_adm"));

        env.events()
            .publish((symbol_short!("adm_acpt"),), (pending,));

        Ok(())
    }

    /// Return the pending admin address if a proposal is active.
    pub fn get_pending_admin(env: Env) -> Option<Address> {
        env.storage().instance().get(&symbol_short!("pend_adm"))
    }
}

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
    fn test_record_behavior_and_history() {
        let t = setup();

        let behavior = BehaviorData {
            action_type: ActionType::Trade,
            outcome: Outcome::Success,
            volume: 1000,
            timestamp: t.env.ledger().timestamp(),
        };

        t.client.record_behavior(&t.escrow_mgr, &t.user, &behavior);

        let history = t.client.reputation_history(&t.user, &1000);
        assert_eq!(history.len(), 1);

        let metrics = t.client.get_trust_metrics(&t.user);
        assert_eq!(metrics.successful_trades, 1);
        assert_eq!(metrics.total_volume, 1000);
    }

    #[test]
    fn test_update_reputation_and_score() {
        let t = setup();

        t.client.update_reputation(
            &t.escrow_mgr,
            &t.user,
            &ActionType::Trade,
            &Outcome::Success,
        );
        t.client.update_reputation(
            &t.escrow_mgr,
            &t.user,
            &ActionType::Repayment,
            &Outcome::Early,
        );

        let score = t.client.calculate_reputation_score(&t.user);
        assert!(score > 500);

        let verify = t.client.verify_reputation(&t.user, &505);
        assert!(verify);
    }
}

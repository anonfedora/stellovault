//! Risk Assessment & Liquidation Engine for StelloVault
//!
//! This contract monitors position health, calculates risk factors, and executes
//! automated liquidations for undercollateralized positions. It acts as the
//! systemic risk guardian, preventing cascading defaults and protecting lender capital.

#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, token, Address, Env, Symbol,
};

// ============================================================================
// Error Types
// ============================================================================

/// Contract errors
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContractError {
    // Initialization errors
    Unauthorized = 1,
    AlreadyInitialized = 2,

    // Position errors
    PositionNotFound = 3,
    PositionNotLiquidatable = 4,
    PositionAlreadyLiquidated = 5,

    // Liquidation errors
    LiquidationsPaused = 6,
    LiquidationCooldown = 7,
    ExceedsMaxLiquidation = 8,
    InsufficientLiquidatorFunds = 9,

    // Parameter errors
    InvalidThreshold = 10,
    InvalidPenalty = 11,
    InvalidHealthFactor = 12,
    InvalidMaxLiquidation = 13,
    InvalidGracePeriod = 14,
    InvalidBonus = 15,

    // Math errors
    MathOverflow = 16,
    DivisionByZero = 17,

    // Cross-contract errors
    CollateralNotFound = 18,
    LoanNotFound = 19,
    EscrowNotFound = 20,

    // Timelock errors
    TimelockNotExpired = 21,
    NoPendingUpdate = 22,

    // Governance errors
    NotGovernance = 23,

    // Loan status errors
    LoanNotActive = 24,

    // Auction errors
    AuctionAlreadyActive = 25,
    AuctionNotFound = 26,
    AuctionExpired = 27,
    AuctionNotActive = 28,
    BidBelowDebtFloor = 29,
    AuctionNotExpired = 30,
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

// ============================================================================
// Data Structures
// ============================================================================

/// Risk parameters configurable by governance
#[contracttype]
#[derive(Clone, Debug)]
pub struct RiskParameters {
    /// Liquidation threshold in basis points (e.g., 8000 = 80%)
    /// When collateral ratio drops below this, position becomes at risk
    pub liquidation_threshold: u32,

    /// Liquidation penalty in basis points (e.g., 500 = 5%)
    /// Extra amount added to debt during liquidation
    pub liquidation_penalty: u32,

    /// Minimum health factor in basis points (e.g., 10000 = 1.0)
    /// Below this threshold, liquidation can occur
    pub min_health_factor: u32,

    /// Maximum liquidation per call in basis points (e.g., 5000 = 50%)
    pub max_liquidation_ratio: u32,

    /// Grace period in seconds before liquidation allowed
    pub grace_period: u64,

    /// Liquidator bonus in basis points (e.g., 500 = 5%)
    pub liquidator_bonus: u32,
}

impl RiskParameters {
    /// Default risk parameters
    pub fn default() -> Self {
        Self {
            liquidation_threshold: 8000,    // 80%
            liquidation_penalty: 500,       // 5%
            min_health_factor: 10000,       // 1.0
            max_liquidation_ratio: 5000,    // 50%
            grace_period: 3600,             // 1 hour
            liquidator_bonus: 500,          // 5%
        }
    }
}

/// Risk status for a position
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PositionRisk {
    Healthy = 0,      // Health factor >= 1.5 (15000 basis points)
    Warning = 1,      // Health factor 1.2 - 1.5 (12000-15000)
    Danger = 2,       // Health factor 1.0 - 1.2 (10000-12000)
    Liquidatable = 3, // Health factor < 1.0 (< min_health_factor)
}

/// Loan status (mirrors LoanManagement)
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoanStatus {
    Active = 0,
    Repaid = 1,
    Defaulted = 2,
    Liquidated = 3,
}

/// Escrow status (mirrors StelloVault)
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EscrowStatus {
    Pending = 0,
    Active = 1,
    Released = 2,
    Cancelled = 3,
    Disputed = 4,
}

/// Aggregated position data for risk calculation
#[contracttype]
#[derive(Clone, Debug)]
pub struct PositionData {
    pub escrow_id: u64,
    pub loan_id: u64,
    pub collateral_id: u64,
    pub borrower: Address,
    pub lender: Address,
    pub collateral_value: i128,
    pub debt_amount: i128,
    pub interest_rate: u32,
    pub deadline: u64,
    pub health_factor: u32,
    pub risk_status: PositionRisk,
    pub last_updated: u64,
}

/// Record of a liquidation event
#[contracttype]
#[derive(Clone, Debug)]
pub struct LiquidationRecord {
    pub position_id: u64,
    pub liquidator: Address,
    pub debt_covered: i128,
    pub collateral_seized: i128,
    pub liquidator_bonus: i128,
    pub borrower_surplus: i128,
    pub timestamp: u64,
    pub partial: bool,
}

/// Pending parameter update with timelock
#[contracttype]
#[derive(Clone, Debug)]
pub struct PendingUpdate {
    pub new_params: RiskParameters,
    pub proposer: Address,
    pub proposed_at: u64,
    pub execute_after: u64,
}

/// Dutch Auction configuration (governance-controlled)
#[contracttype]
#[derive(Clone, Debug)]
pub struct AuctionConfig {
    /// Auction duration in seconds (default: 6 hours)
    pub duration: u64,
    /// Decay rate in basis points per second (default: ~2 bps/s → full decay over 6h)
    /// price = collateral_value * (1 - decay_rate_bps * elapsed / 1_000_000)
    pub decay_rate_bps_per_sec: u64,
    /// Auction fee charged on surplus, in basis points (default: 50 = 0.5%)
    pub auction_fee_bps: u32,
}

impl AuctionConfig {
    pub fn default() -> Self {
        Self {
            duration: 21_600,             // 6 hours
            decay_rate_bps_per_sec: 46,   // ~46 bps/s → 100% decay in ~6h at 10 000 bps scale
            auction_fee_bps: 50,          // 0.5%
        }
    }
}

/// Auction lifecycle status
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuctionStatus {
    Active = 0,
    Settled = 1,
    Expired = 2,
}

/// State of a Dutch Auction for one defaulted loan
#[contracttype]
#[derive(Clone, Debug)]
pub struct AuctionState {
    pub loan_id: u64,
    pub collateral_value: i128,
    pub debt_floor: i128,
    pub started_at: u64,
    pub ends_at: u64,
    pub status: AuctionStatus,
    /// Address of the winning bidder (zero-value until settled)
    pub winner: Option<Address>,
    pub winning_bid: i128,
    /// Amount paid to lender
    pub debt_covered: i128,
    /// Surplus returned to borrower (after auction fee)
    pub borrower_surplus: i128,
    /// Auction fee collected by protocol
    pub auction_fee: i128,
}

// ============================================================================
// External Contract Data Structures (for cross-contract calls)
// ============================================================================

/// Loan data structure (from LoanManagement)
#[contracttype]
#[derive(Clone, Debug)]
pub struct Loan {
    pub id: u64,
    pub escrow_id: u64,
    pub borrower: Address,
    pub lender: Address,
    pub amount: i128,
    pub interest_rate: u32,
    pub deadline: u64,
    pub status: LoanStatus,
}

/// Collateral data structure (from CollateralRegistry)
#[contracttype]
#[derive(Clone, Debug)]
pub struct Collateral {
    pub id: u64,
    pub owner: Address,
    pub face_value: i128,
    pub realized_value: i128,
    pub expiry_ts: u64,
    pub registered_at: u64,
    pub last_valuation_ts: u64,
    pub locked: bool,
}

/// Trade escrow data structure (from StelloVault)
#[contracttype]
#[derive(Clone, Debug)]
pub struct TradeEscrow {
    pub buyer: Address,
    pub seller: Address,
    pub lender: Address,
    pub collateral_token_id: u64,
    pub amount: i128,
    pub asset: Address,
    pub status: EscrowStatus,
    pub oracle_address: Address,
    pub release_conditions: Symbol,
    pub expiry_ts: u64,
    pub created_at: u64,
}

// ============================================================================
// Event Symbols
// ============================================================================

const EVT_INIT: Symbol = symbol_short!("risk_init");
const EVT_HF_UPD: Symbol = symbol_short!("hf_upd");
const EVT_LIQ_EXEC: Symbol = symbol_short!("liq_exec");
const EVT_COLL_SZD: Symbol = symbol_short!("coll_szd");
const EVT_PARAM_PROP: Symbol = symbol_short!("prm_prop");
const EVT_PARAM_UPD: Symbol = symbol_short!("prm_upd");
const EVT_PARAM_CANCEL: Symbol = symbol_short!("prm_cncl");
const EVT_PAUSED: Symbol = symbol_short!("liq_pause");
const EVT_UNPAUSED: Symbol = symbol_short!("liq_unpse");
const EVT_AUC_START: Symbol = symbol_short!("auc_start");
const EVT_AUC_BID: Symbol = symbol_short!("auc_bid");
const EVT_AUC_SETL: Symbol = symbol_short!("auc_setl");
const EVT_AUC_EXP: Symbol = symbol_short!("auc_exp");

// ============================================================================
// Contract Definition
// ============================================================================

#[contract]
pub struct RiskAssessment;

#[contractimpl]
impl RiskAssessment {
    // ========================================================================
    // Initialization
    // ========================================================================

    /// Initialize the contract with admin and external contract addresses
    ///
    /// # Arguments
    /// * `admin` - Admin address for emergency controls
    /// * `governance` - Governance contract address for parameter updates
    /// * `collateral_registry` - CollateralRegistry contract address
    /// * `loan_management` - LoanManagement contract address
    /// * `vault` - StelloVault contract address
    pub fn initialize(
        env: Env,
        admin: Address,
        governance: Address,
        collateral_registry: Address,
        loan_management: Address,
        vault: Address,
    ) -> Result<(), ContractError> {
        // Prevent re-initialization
        if env.storage().instance().has(&symbol_short!("admin")) {
            return Err(ContractError::AlreadyInitialized);
        }

        // Store contract addresses
        env.storage().instance().set(&symbol_short!("admin"), &admin);
        env.storage().instance().set(&symbol_short!("gov"), &governance);
        env.storage().instance().set(&symbol_short!("coll_reg"), &collateral_registry);
        env.storage().instance().set(&symbol_short!("loan_mgr"), &loan_management);
        env.storage().instance().set(&symbol_short!("vault"), &vault);

        // Set default risk parameters
        let default_params = RiskParameters::default();
        env.storage().instance().set(&symbol_short!("risk_prm"), &default_params);

        // Set paused to false
        env.storage().instance().set(&symbol_short!("paused"), &false);

        // Set default timelock duration (24 hours)
        env.storage().instance().set(&symbol_short!("timelock"), &86400u64);

        // Set default auction config
        let default_auction_cfg = AuctionConfig::default();
        env.storage().instance().set(&symbol_short!("auc_cfg"), &default_auction_cfg);

        // Emit initialization event
        env.events().publish(
            (EVT_INIT,),
            (admin.clone(), governance),
        );

        Ok(())
    }

    /// Get current risk parameters
    pub fn get_risk_parameters(env: Env) -> RiskParameters {
        env.storage()
            .instance()
            .get(&symbol_short!("risk_prm"))
            .unwrap_or(RiskParameters::default())
    }

    /// Get admin address
    pub fn admin(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&symbol_short!("admin"))
            .unwrap()
    }

    /// Get governance address
    pub fn governance(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&symbol_short!("gov"))
            .unwrap()
    }

    // ========================================================================
    // Health Factor Calculation
    // ========================================================================

    /// Calculate the health factor for a position
    ///
    /// Formula: (Collateral Value * Liquidation Threshold) / Total Debt
    /// Returns basis points (10000 = 1.0 healthy, <10000 = at risk)
    ///
    /// # Arguments
    /// * `position_id` - The escrow ID representing the position
    pub fn calculate_health_factor(env: Env, position_id: u64) -> Result<u32, ContractError> {
        let risk_params = Self::get_risk_parameters(env.clone());

        // Fetch position data from external contracts
        let (loan, collateral, _escrow) = Self::fetch_position_data(&env, position_id)?;

        // Check loan is active
        if loan.status != LoanStatus::Active {
            return Err(ContractError::LoanNotActive);
        }

        // Calculate total debt with interest
        let interest = loan.amount
            .checked_mul(loan.interest_rate as i128)
            .ok_or(ContractError::MathOverflow)?
            / 10000;

        let total_debt = loan.amount
            .checked_add(interest)
            .ok_or(ContractError::MathOverflow)?;

        // Handle zero debt case (infinite health factor)
        if total_debt == 0 {
            return Ok(u32::MAX);
        }

        // Calculate health factor
        // HF = (Collateral Value * Liquidation Threshold) / Total Debt
        let numerator = (collateral.realized_value)
            .checked_mul(risk_params.liquidation_threshold as i128)
            .ok_or(ContractError::MathOverflow)?;

        let health_factor = numerator
            .checked_div(total_debt)
            .ok_or(ContractError::DivisionByZero)? as u32;

        // Emit health factor update event
        let risk_status = Self::calculate_risk_status(health_factor, risk_params.min_health_factor);
        env.events().publish(
            (EVT_HF_UPD,),
            (position_id, health_factor, risk_status as u32),
        );

        Ok(health_factor)
    }

    /// Check if a position is liquidatable
    ///
    /// A position is liquidatable if:
    /// 1. Health factor < min_health_factor
    /// 2. Grace period has passed since position became undercollateralized
    pub fn is_liquidatable(env: Env, position_id: u64) -> Result<bool, ContractError> {
        let risk_params = Self::get_risk_parameters(env.clone());
        let health_factor = Self::calculate_health_factor(env.clone(), position_id)?;

        // Check if health factor is below minimum
        if health_factor >= risk_params.min_health_factor {
            return Ok(false);
        }

        // Check grace period (simplified - in production would track when position became underwater)
        // For now, we check cooldown instead
        let cooldown_key = (symbol_short!("cooldown"), position_id);
        if let Some(last_liquidation) = env.storage().persistent().get::<_, u64>(&cooldown_key) {
            let current_ts = env.ledger().timestamp();
            if current_ts < last_liquidation + risk_params.grace_period {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Get the risk status for a position
    pub fn get_position_risk(env: Env, position_id: u64) -> Result<PositionRisk, ContractError> {
        let risk_params = Self::get_risk_parameters(env.clone());
        let health_factor = Self::calculate_health_factor(env, position_id)?;

        Ok(Self::calculate_risk_status(health_factor, risk_params.min_health_factor))
    }

    /// Get aggregated position data
    pub fn get_position_data(env: Env, position_id: u64) -> Result<PositionData, ContractError> {
        let risk_params = Self::get_risk_parameters(env.clone());
        let (loan, collateral, _escrow) = Self::fetch_position_data(&env, position_id)?;

        // Calculate interest
        let interest = loan.amount
            .checked_mul(loan.interest_rate as i128)
            .ok_or(ContractError::MathOverflow)?
            / 10000;

        let total_debt = loan.amount
            .checked_add(interest)
            .ok_or(ContractError::MathOverflow)?;

        // Calculate health factor
        let health_factor = if total_debt == 0 {
            u32::MAX
        } else {
            let numerator = (collateral.realized_value)
                .checked_mul(risk_params.liquidation_threshold as i128)
                .ok_or(ContractError::MathOverflow)?;
            numerator
                .checked_div(total_debt)
                .ok_or(ContractError::DivisionByZero)? as u32
        };

        let risk_status = Self::calculate_risk_status(health_factor, risk_params.min_health_factor);

        Ok(PositionData {
            escrow_id: position_id,
            loan_id: loan.id,
            collateral_id: collateral.id,
            borrower: loan.borrower,
            lender: loan.lender,
            collateral_value: collateral.realized_value,
            debt_amount: total_debt,
            interest_rate: loan.interest_rate,
            deadline: loan.deadline,
            health_factor,
            risk_status,
            last_updated: env.ledger().timestamp(),
        })
    }

    /// Get borrower's overall risk factor based on all their positions
    /// 
    /// This function assesses the borrower's overall risk profile by analyzing
    /// all their active positions and returns a risk factor (0-3):
    /// - 0: Healthy (all positions healthy)
    /// - 1: Warning (some positions in warning range)
    /// - 2: Danger (some positions in danger range)
    /// - 3: Liquidatable (any position liquidatable)
    pub fn get_borrower_risk_factor(env: Env, borrower: Address) -> Result<u32, ContractError> {
        // For this implementation, we'll use a simplified approach
        // In a production environment, you would:
        // 1. Query all active positions for the borrower
        // 2. Calculate the worst risk status among all positions
        // 3. Return the corresponding risk factor
        
        // For now, we'll implement a basic version that checks if the borrower
        // has any existing positions and assigns risk based on that
        
        // Try to get borrower's risk history from storage (for testing/simulation)
        let borrower_risk_key = (symbol_short!("borrower_risk"), borrower.clone());
        if let Some(stored_risk) = env.storage().persistent().get::<_, u32>(&borrower_risk_key) {
            return Ok(stored_risk);
        }
        
        // If no stored risk data, use a simple heuristic based on address
        // In production, this would be replaced with actual position analysis
        let address_bytes = borrower.to_fixed_bytes();
        let risk_score = (address_bytes[31] % 4) as u32; // 0-3 range
        
        Ok(risk_score)
    }

    /// Set borrower risk factor (for testing purposes only)
    #[cfg(any(test, feature = "testutils"))]
    pub fn set_borrower_risk_factor(env: Env, borrower: Address, risk_factor: u32) -> Result<(), ContractError> {
        if risk_factor > 3 {
            return Err(ContractError::InvalidHealthFactor);
        }
        
        let borrower_risk_key = (symbol_short!("borrower_risk"), borrower);
        env.storage().persistent().set(&borrower_risk_key, &risk_factor);
        Ok(())
    }

    // ========================================================================
    // Liquidation Engine
    // ========================================================================

    /// Execute liquidation on an undercollateralized position
    ///
    /// # Arguments
    /// * `position_id` - The escrow ID representing the position
    /// * `liquidator` - Address of the liquidator
    /// * `amount` - Optional amount for partial liquidation (None = full liquidation)
    pub fn liquidate(
        env: Env,
        position_id: u64,
        liquidator: Address,
        amount: Option<i128>,
    ) -> Result<LiquidationRecord, ContractError> {
        // Require liquidator authorization
        liquidator.require_auth();

        // Check liquidations not paused
        let paused: bool = env.storage()
            .instance()
            .get(&symbol_short!("paused"))
            .unwrap_or(false);

        if paused {
            return Err(ContractError::LiquidationsPaused);
        }

        let risk_params = Self::get_risk_parameters(env.clone());

        // Check health factor qualifies for liquidation
        let health_factor = Self::calculate_health_factor(env.clone(), position_id)?;
        if health_factor >= risk_params.min_health_factor {
            return Err(ContractError::PositionNotLiquidatable);
        }

        // Check cooldown period
        let cooldown_key = (symbol_short!("cooldown"), position_id);
        if let Some(last_liquidation) = env.storage().persistent().get::<_, u64>(&cooldown_key) {
            let current_ts = env.ledger().timestamp();
            if current_ts < last_liquidation + risk_params.grace_period {
                return Err(ContractError::LiquidationCooldown);
            }
        }

        // Fetch position data
        let (loan, collateral, escrow) = Self::fetch_position_data(&env, position_id)?;

        // Check loan is active
        if loan.status != LoanStatus::Active {
            return Err(ContractError::PositionAlreadyLiquidated);
        }

        // Calculate total debt with interest
        let interest = loan.amount
            .checked_mul(loan.interest_rate as i128)
            .ok_or(ContractError::MathOverflow)?
            / 10000;

        let total_debt = loan.amount
            .checked_add(interest)
            .ok_or(ContractError::MathOverflow)?;

        // Determine liquidation amount
        let is_partial = amount.is_some();
        let liquidation_amount = match amount {
            Some(amt) => {
                // Partial liquidation - max allowed is max_liquidation_ratio of total debt
                let max_partial = total_debt
                    .checked_mul(risk_params.max_liquidation_ratio as i128)
                    .ok_or(ContractError::MathOverflow)?
                    / 10000;

                if amt > max_partial {
                    return Err(ContractError::ExceedsMaxLiquidation);
                }
                amt
            }
            None => total_debt, // Full liquidation
        };

        // Calculate penalty
        let penalty = liquidation_amount
            .checked_mul(risk_params.liquidation_penalty as i128)
            .ok_or(ContractError::MathOverflow)?
            / 10000;

        let total_to_pay = liquidation_amount
            .checked_add(penalty)
            .ok_or(ContractError::MathOverflow)?;

        // Calculate collateral to seize proportionally
        let collateral_ratio = if total_debt > 0 {
            liquidation_amount
                .checked_mul(10000)
                .ok_or(ContractError::MathOverflow)?
                / total_debt
        } else {
            10000 // 100% if no debt
        };

        let collateral_to_seize = collateral.face_value
            .checked_mul(collateral_ratio)
            .ok_or(ContractError::MathOverflow)?
            / 10000;

        // Calculate liquidator bonus
        let liquidator_bonus = collateral_to_seize
            .checked_mul(risk_params.liquidator_bonus as i128)
            .ok_or(ContractError::MathOverflow)?
            / 10000;

        // Calculate surplus to return to borrower
        let borrower_surplus = if collateral_to_seize > total_to_pay + liquidator_bonus {
            collateral_to_seize - total_to_pay - liquidator_bonus
        } else {
            0
        };

        // Execute liquidation atomically

        // 1. Transfer payment from liquidator to lender
        let token_client = token::Client::new(&env, &escrow.asset);
        token_client.transfer(&liquidator, &loan.lender, &liquidation_amount);

        // 2. Mark loan as liquidated via LoanManagement
        // Note: This requires LoanManagement to have mark_liquidated function
        // For now, we store the liquidation record and emit events
        // The integration with LoanManagement::mark_liquidated would be done here

        // 3. Record liquidation
        let liquidation_record = LiquidationRecord {
            position_id,
            liquidator: liquidator.clone(),
            debt_covered: liquidation_amount,
            collateral_seized: collateral_to_seize,
            liquidator_bonus,
            borrower_surplus,
            timestamp: env.ledger().timestamp(),
            partial: is_partial,
        };

        env.storage().persistent().set(
            &(symbol_short!("liq_rec"), position_id),
            &liquidation_record,
        );

        // 4. Update cooldown
        env.storage().persistent().set(&cooldown_key, &env.ledger().timestamp());

        // 5. Emit events
        env.events().publish(
            (EVT_LIQ_EXEC,),
            (position_id, liquidator.clone(), liquidation_amount, collateral_to_seize),
        );

        env.events().publish(
            (EVT_COLL_SZD,),
            (position_id, collateral.id, collateral_to_seize),
        );

        Ok(liquidation_record)
    }

    /// Get liquidation record for a position
    pub fn get_liquidation_record(env: Env, position_id: u64) -> Option<LiquidationRecord> {
        env.storage()
            .persistent()
            .get(&(symbol_short!("liq_rec"), position_id))
    }

    // ========================================================================
    // Governance Functions
    // ========================================================================

    /// Propose new risk parameters (governance only)
    /// Creates a pending update with timelock
    pub fn update_risk_parameters(
        env: Env,
        new_params: RiskParameters,
    ) -> Result<(), ContractError> {
        // Verify caller is governance
        let governance: Address = env.storage()
            .instance()
            .get(&symbol_short!("gov"))
            .ok_or(ContractError::Unauthorized)?;

        governance.require_auth();

        // Validate parameters
        Self::validate_parameters(&new_params)?;

        // Get timelock duration
        let timelock_duration: u64 = env.storage()
            .instance()
            .get(&symbol_short!("timelock"))
            .unwrap_or(86400);

        let current_ts = env.ledger().timestamp();
        let execute_after = current_ts
            .checked_add(timelock_duration)
            .ok_or(ContractError::MathOverflow)?;

        // Create pending update
        let pending = PendingUpdate {
            new_params: new_params.clone(),
            proposer: governance.clone(),
            proposed_at: current_ts,
            execute_after,
        };

        env.storage().instance().set(&symbol_short!("pending"), &pending);

        // Emit proposal event
        env.events().publish(
            (EVT_PARAM_PROP,),
            (
                new_params.liquidation_threshold,
                new_params.liquidation_penalty,
                new_params.min_health_factor,
                execute_after,
            ),
        );

        Ok(())
    }

    /// Execute pending parameter update after timelock
    pub fn execute_parameter_update(env: Env) -> Result<(), ContractError> {
        // Get pending update
        let pending: PendingUpdate = env.storage()
            .instance()
            .get(&symbol_short!("pending"))
            .ok_or(ContractError::NoPendingUpdate)?;

        // Check timelock expired
        let current_ts = env.ledger().timestamp();
        if current_ts < pending.execute_after {
            return Err(ContractError::TimelockNotExpired);
        }

        // Apply new parameters
        env.storage().instance().set(&symbol_short!("risk_prm"), &pending.new_params);

        // Clear pending update
        env.storage().instance().remove(&symbol_short!("pending"));

        // Emit update event
        env.events().publish(
            (EVT_PARAM_UPD,),
            (
                pending.new_params.liquidation_threshold,
                pending.new_params.liquidation_penalty,
                pending.new_params.min_health_factor,
            ),
        );

        Ok(())
    }

    /// Cancel pending parameter update (governance only)
    pub fn cancel_parameter_update(env: Env) -> Result<(), ContractError> {
        // Verify caller is governance
        let governance: Address = env.storage()
            .instance()
            .get(&symbol_short!("gov"))
            .ok_or(ContractError::Unauthorized)?;

        governance.require_auth();

        // Check pending update exists
        if !env.storage().instance().has(&symbol_short!("pending")) {
            return Err(ContractError::NoPendingUpdate);
        }

        // Clear pending update
        env.storage().instance().remove(&symbol_short!("pending"));

        // Emit cancel event
        env.events().publish(
            (EVT_PARAM_CANCEL,),
            (env.ledger().timestamp(),),
        );

        Ok(())
    }

    /// Get pending parameter update if any
    pub fn get_pending_update(env: Env) -> Option<PendingUpdate> {
        env.storage().instance().get(&symbol_short!("pending"))
    }

    // ========================================================================
    // Emergency Controls
    // ========================================================================

    /// Pause all liquidations (admin only)
    pub fn pause_liquidations(env: Env) -> Result<(), ContractError> {
        // Verify caller is admin
        let admin: Address = env.storage()
            .instance()
            .get(&symbol_short!("admin"))
            .ok_or(ContractError::Unauthorized)?;

        admin.require_auth();

        // Set paused flag
        env.storage().instance().set(&symbol_short!("paused"), &true);

        // Emit paused event
        env.events().publish(
            (EVT_PAUSED,),
            (admin, env.ledger().timestamp()),
        );

        Ok(())
    }

    /// Unpause liquidations (admin only)
    pub fn unpause_liquidations(env: Env) -> Result<(), ContractError> {
        // Verify caller is admin
        let admin: Address = env.storage()
            .instance()
            .get(&symbol_short!("admin"))
            .ok_or(ContractError::Unauthorized)?;

        admin.require_auth();

        // Clear paused flag
        env.storage().instance().set(&symbol_short!("paused"), &false);

        // Emit unpaused event
        env.events().publish(
            (EVT_UNPAUSED,),
            (admin, env.ledger().timestamp()),
        );

        Ok(())
    }

    /// Check if liquidations are paused
    pub fn is_paused(env: Env) -> bool {
        env.storage()
            .instance()
            .get(&symbol_short!("paused"))
            .unwrap_or(false)
    }

    // ========================================================================
    // Admin Functions
    // ========================================================================

    /// Set collateral registry address (admin only)
    pub fn set_collateral_registry(env: Env, address: Address) -> Result<(), ContractError> {
        let admin: Address = env.storage()
            .instance()
            .get(&symbol_short!("admin"))
            .ok_or(ContractError::Unauthorized)?;

        admin.require_auth();

        env.storage().instance().set(&symbol_short!("coll_reg"), &address);
        Ok(())
    }

    /// Set loan management address (admin only)
    pub fn set_loan_management(env: Env, address: Address) -> Result<(), ContractError> {
        let admin: Address = env.storage()
            .instance()
            .get(&symbol_short!("admin"))
            .ok_or(ContractError::Unauthorized)?;

        admin.require_auth();

        env.storage().instance().set(&symbol_short!("loan_mgr"), &address);
        Ok(())
    }

    /// Set vault address (admin only)
    pub fn set_vault(env: Env, address: Address) -> Result<(), ContractError> {
        let admin: Address = env.storage()
            .instance()
            .get(&symbol_short!("admin"))
            .ok_or(ContractError::Unauthorized)?;

        admin.require_auth();

        env.storage().instance().set(&symbol_short!("vault"), &address);
        Ok(())
    }

    /// Set timelock duration (admin only)
    pub fn set_timelock_duration(env: Env, duration: u64) -> Result<(), ContractError> {
        let admin: Address = env.storage()
            .instance()
            .get(&symbol_short!("admin"))
            .ok_or(ContractError::Unauthorized)?;

        admin.require_auth();

        env.storage().instance().set(&symbol_short!("timelock"), &duration);
        Ok(())
    }

    // ========================================================================
    // Internal Helper Functions
    // ========================================================================

    /// Calculate risk status from health factor
    fn calculate_risk_status(health_factor: u32, min_health_factor: u32) -> PositionRisk {
        if health_factor >= 15000 {
            PositionRisk::Healthy
        } else if health_factor >= 12000 {
            PositionRisk::Warning
        } else if health_factor >= min_health_factor {
            PositionRisk::Danger
        } else {
            PositionRisk::Liquidatable
        }
    }

    /// Validate risk parameters
    fn validate_parameters(params: &RiskParameters) -> Result<(), ContractError> {
        // Liquidation threshold: 50-95%
        if params.liquidation_threshold < 5000 || params.liquidation_threshold > 9500 {
            return Err(ContractError::InvalidThreshold);
        }

        // Liquidation penalty: 1-10%
        if params.liquidation_penalty < 100 || params.liquidation_penalty > 1000 {
            return Err(ContractError::InvalidPenalty);
        }

        // Min health factor: 1.0-1.5
        if params.min_health_factor < 10000 || params.min_health_factor > 15000 {
            return Err(ContractError::InvalidHealthFactor);
        }

        // Max liquidation ratio: 25-50%
        if params.max_liquidation_ratio < 2500 || params.max_liquidation_ratio > 5000 {
            return Err(ContractError::InvalidMaxLiquidation);
        }

        // Grace period: at least 5 minutes, max 24 hours
        if params.grace_period < 300 || params.grace_period > 86400 {
            return Err(ContractError::InvalidGracePeriod);
        }

        // Liquidator bonus: 1-10%
        if params.liquidator_bonus < 100 || params.liquidator_bonus > 1000 {
            return Err(ContractError::InvalidBonus);
        }

        Ok(())
    }

    /// Fetch position data from external contracts
    /// In production, this would use cross-contract calls
    /// For now, we use storage simulation for testing
    fn fetch_position_data(
        env: &Env,
        position_id: u64,
    ) -> Result<(Loan, Collateral, TradeEscrow), ContractError> {
        // Try to get from test storage first (for unit tests)
        let loan_key = (symbol_short!("test_loan"), position_id);
        let coll_key = (symbol_short!("test_coll"), position_id);
        let escrow_key = (symbol_short!("test_escr"), position_id);

        let loan: Loan = env.storage()
            .persistent()
            .get(&loan_key)
            .ok_or(ContractError::LoanNotFound)?;

        let collateral: Collateral = env.storage()
            .persistent()
            .get(&coll_key)
            .ok_or(ContractError::CollateralNotFound)?;

        let escrow: TradeEscrow = env.storage()
            .persistent()
            .get(&escrow_key)
            .ok_or(ContractError::EscrowNotFound)?;

        Ok((loan, collateral, escrow))
    }

    /// Set test data for a position (for testing only)
    #[cfg(any(test, feature = "testutils"))]
    pub fn set_test_position(
        env: Env,
        position_id: u64,
        loan: Loan,
        collateral: Collateral,
        escrow: TradeEscrow,
    ) {
        let loan_key = (symbol_short!("test_loan"), position_id);
        let coll_key = (symbol_short!("test_coll"), position_id);
        let escrow_key = (symbol_short!("test_escr"), position_id);

        env.storage().persistent().set(&loan_key, &loan);
        env.storage().persistent().set(&coll_key, &collateral);
        env.storage().persistent().set(&escrow_key, &escrow);
    }

    // ========================================================================
    // Dutch Auction — Governance Config
    // ========================================================================

    /// Get current auction configuration
    pub fn get_auction_config(env: Env) -> AuctionConfig {
        env.storage()
            .instance()
            .get(&symbol_short!("auc_cfg"))
            .unwrap_or(AuctionConfig::default())
    }

    /// Update auction configuration (governance only, with timelock)
    pub fn set_auction_config(
        env: Env,
        config: AuctionConfig,
    ) -> Result<(), ContractError> {
        let governance: Address = env.storage()
            .instance()
            .get(&symbol_short!("gov"))
            .ok_or(ContractError::Unauthorized)?;
        governance.require_auth();

        env.storage().instance().set(&symbol_short!("auc_cfg"), &config);
        Ok(())
    }

    // ========================================================================
    // Dutch Auction — Core Functions
    // ========================================================================

    /// Start a Dutch Auction for a defaulted / undercollateralised loan.
    ///
    /// Anyone may trigger this once the loan is liquidatable. The starting price
    /// equals the full collateral value and decays linearly toward the debt floor
    /// over the configured auction duration.
    pub fn start_auction(
        env: Env,
        loan_id: u64,
    ) -> Result<AuctionState, ContractError> {
        // Guard: liquidations not paused
        let paused: bool = env.storage()
            .instance()
            .get(&symbol_short!("paused"))
            .unwrap_or(false);
        if paused {
            return Err(ContractError::LiquidationsPaused);
        }

        // Guard: no active auction already running
        let auc_key = (symbol_short!("auction"), loan_id);
        if let Some(existing) = env.storage().persistent().get::<_, AuctionState>(&auc_key) {
            if existing.status == AuctionStatus::Active {
                return Err(ContractError::AuctionAlreadyActive);
            }
        }

        // Position must be liquidatable
        let health_factor = Self::calculate_health_factor(env.clone(), loan_id)?;
        let risk_params = Self::get_risk_parameters(env.clone());
        if health_factor >= risk_params.min_health_factor {
            return Err(ContractError::PositionNotLiquidatable);
        }

        // Fetch position data
        let (loan, collateral, _escrow) = Self::fetch_position_data(&env, loan_id)?;
        if loan.status != LoanStatus::Active {
            return Err(ContractError::LoanNotActive);
        }

        // Compute debt floor (principal + interest)
        let interest = loan.amount
            .checked_mul(loan.interest_rate as i128)
            .ok_or(ContractError::MathOverflow)?
            / 10_000;
        let debt_floor = loan.amount
            .checked_add(interest)
            .ok_or(ContractError::MathOverflow)?;

        let cfg = Self::get_auction_config(env.clone());
        let now = env.ledger().timestamp();
        let ends_at = now.checked_add(cfg.duration).ok_or(ContractError::MathOverflow)?;

        let state = AuctionState {
            loan_id,
            collateral_value: collateral.realized_value,
            debt_floor,
            started_at: now,
            ends_at,
            status: AuctionStatus::Active,
            winner: None,
            winning_bid: 0,
            debt_covered: 0,
            borrower_surplus: 0,
            auction_fee: 0,
        };

        env.storage().persistent().set(&auc_key, &state);

        env.events().publish(
            (EVT_AUC_START,),
            (loan_id, collateral.realized_value, debt_floor, ends_at),
        );

        Ok(state)
    }

    /// Compute the current Dutch-Auction price for a loan.
    ///
    /// `price = collateral_value * (1 - decay_rate_bps_per_sec * elapsed / 1_000_000)`
    /// Clamped at `debt_floor` so the lender is always fully repaid.
    pub fn get_auction_price(env: Env, loan_id: u64) -> Result<i128, ContractError> {
        let auc_key = (symbol_short!("auction"), loan_id);
        let state: AuctionState = env.storage()
            .persistent()
            .get(&auc_key)
            .ok_or(ContractError::AuctionNotFound)?;

        if state.status != AuctionStatus::Active {
            return Err(ContractError::AuctionNotActive);
        }

        let now = env.ledger().timestamp();
        if now >= state.ends_at {
            // Auction has timed out — price floors at debt_floor
            return Ok(state.debt_floor);
        }

        let cfg = Self::get_auction_config(env.clone());
        let elapsed = now.saturating_sub(state.started_at);

        // decay_factor_bps = decay_rate_bps_per_sec * elapsed  (capped at 1_000_000)
        let decay_factor = (cfg.decay_rate_bps_per_sec as u128)
            .saturating_mul(elapsed as u128)
            .min(1_000_000);

        // price = collateral_value * (1_000_000 - decay_factor) / 1_000_000
        let price = (state.collateral_value as u128)
            .saturating_mul(1_000_000u128.saturating_sub(decay_factor))
            / 1_000_000;

        let price = price as i128;
        // Floor at debt_floor
        Ok(if price < state.debt_floor { state.debt_floor } else { price })
    }

    /// Place a bid on an active Dutch Auction.
    ///
    /// The first bidder whose `payment_amount >= current_auction_price` wins
    /// immediately and triggers settlement:
    ///  - `debt_floor` goes to the lender.
    ///  - surplus above `debt_floor` minus auction fee goes to borrower.
    pub fn bid_auction(
        env: Env,
        loan_id: u64,
        bidder: Address,
        payment_amount: i128,
    ) -> Result<AuctionState, ContractError> {
        bidder.require_auth();

        // Guard: liquidations not paused
        let paused: bool = env.storage()
            .instance()
            .get(&symbol_short!("paused"))
            .unwrap_or(false);
        if paused {
            return Err(ContractError::LiquidationsPaused);
        }

        let auc_key = (symbol_short!("auction"), loan_id);
        let mut state: AuctionState = env.storage()
            .persistent()
            .get(&auc_key)
            .ok_or(ContractError::AuctionNotFound)?;

        if state.status != AuctionStatus::Active {
            return Err(ContractError::AuctionNotActive);
        }

        let now = env.ledger().timestamp();
        if now >= state.ends_at {
            // Mark expired and bail so settle_expired_auction can be called
            state.status = AuctionStatus::Expired;
            env.storage().persistent().set(&auc_key, &state);
            env.events().publish((EVT_AUC_EXP,), (loan_id,));
            return Err(ContractError::AuctionExpired);
        }

        // Bid must be at least the debt floor
        if payment_amount < state.debt_floor {
            return Err(ContractError::BidBelowDebtFloor);
        }

        // Fetch escrow asset for token transfers
        let (_loan, _collateral, escrow) = Self::fetch_position_data(&env, loan_id)?;
        let (loan, _collateral2, _escrow2) = Self::fetch_position_data(&env, loan_id)?;

        // Verify bid is >= current auction price
        let current_price = Self::get_auction_price(env.clone(), loan_id)?;
        // Accept any bid >= current price (first-bid-wins Dutch auction)
        if payment_amount < current_price {
            return Err(ContractError::BidBelowDebtFloor);
        }

        let cfg = Self::get_auction_config(env.clone());

        // Compute distribution
        let debt_covered = state.debt_floor; // lender gets exactly the debt
        let gross_surplus = payment_amount
            .checked_sub(debt_covered)
            .unwrap_or(0);
        let auction_fee = (gross_surplus as i128)
            .checked_mul(cfg.auction_fee_bps as i128)
            .ok_or(ContractError::MathOverflow)?
            / 10_000;
        let borrower_surplus = gross_surplus
            .checked_sub(auction_fee)
            .unwrap_or(0);

        // Transfer: bidder → lender (debt covered)
        let token_client = token::Client::new(&env, &escrow.asset);
        token_client.transfer(&bidder, &loan.lender, &debt_covered);

        // Transfer: bidder → borrower (surplus after fee)
        if borrower_surplus > 0 {
            token_client.transfer(&bidder, &loan.borrower, &borrower_surplus);
        }

        // Update state to Settled
        state.status = AuctionStatus::Settled;
        state.winner = Some(bidder.clone());
        state.winning_bid = payment_amount;
        state.debt_covered = debt_covered;
        state.borrower_surplus = borrower_surplus;
        state.auction_fee = auction_fee;

        env.storage().persistent().set(&auc_key, &state);

        env.events().publish(
            (EVT_AUC_BID,),
            (loan_id, bidder.clone(), payment_amount),
        );
        env.events().publish(
            (EVT_AUC_SETL,),
            (loan_id, debt_covered, borrower_surplus, auction_fee),
        );

        Ok(state)
    }

    /// Mark an expired auction as Expired so a new one can be started.
    pub fn expire_auction(env: Env, loan_id: u64) -> Result<(), ContractError> {
        let auc_key = (symbol_short!("auction"), loan_id);
        let mut state: AuctionState = env.storage()
            .persistent()
            .get(&auc_key)
            .ok_or(ContractError::AuctionNotFound)?;

        if state.status != AuctionStatus::Active {
            return Err(ContractError::AuctionNotActive);
        }

        let now = env.ledger().timestamp();
        if now < state.ends_at {
            return Err(ContractError::AuctionNotExpired);
        }

        state.status = AuctionStatus::Expired;
        env.storage().persistent().set(&auc_key, &state);
        env.events().publish((EVT_AUC_EXP,), (loan_id,));

        Ok(())
    }

    /// Retrieve the current auction state for a loan.
    pub fn get_auction(env: Env, loan_id: u64) -> Option<AuctionState> {
        env.storage()
            .persistent()
            .get(&(symbol_short!("auction"), loan_id))
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Address as _, testutils::Ledger as _, Env};

    fn setup_env() -> (Env, Address, Address, Address, Address, Address) {
        let env = Env::default();
        let admin = Address::generate(&env);
        let governance = Address::generate(&env);
        let collateral_registry = Address::generate(&env);
        let loan_management = Address::generate(&env);
        let vault = Address::generate(&env);

        (env, admin, governance, collateral_registry, loan_management, vault)
    }

    fn create_test_loan(env: &Env, position_id: u64, amount: i128, interest_rate: u32) -> Loan {
        Loan {
            id: position_id,
            escrow_id: position_id,
            borrower: Address::generate(env),
            lender: Address::generate(env),
            amount,
            interest_rate,
            deadline: env.ledger().timestamp() + 86400,
            status: LoanStatus::Active,
        }
    }

    fn create_test_collateral(env: &Env, position_id: u64, face_value: i128) -> Collateral {
        Collateral {
            id: position_id,
            owner: Address::generate(env),
            face_value,
            realized_value: face_value,
            expiry_ts: env.ledger().timestamp() + 86400 * 30,
            registered_at: env.ledger().timestamp(),
            last_valuation_ts: env.ledger().timestamp(),
            locked: true,
        }
    }

    fn create_test_escrow(env: &Env, amount: i128) -> TradeEscrow {
        TradeEscrow {
            buyer: Address::generate(env),
            seller: Address::generate(env),
            lender: Address::generate(env),
            collateral_token_id: 1,
            amount,
            asset: Address::generate(env),
            status: EscrowStatus::Active,
            oracle_address: Address::generate(env),
            release_conditions: symbol_short!("delivery"),
            expiry_ts: env.ledger().timestamp() + 86400,
            created_at: env.ledger().timestamp(),
        }
    }

    // ========================================================================
    // Initialization Tests
    // ========================================================================

    #[test]
    fn test_initialize_success() {
        let (env, admin, governance, coll_reg, loan_mgr, vault) = setup_env();
        let contract_id = env.register(RiskAssessment, ());

        env.mock_all_auths();

        env.as_contract(&contract_id, || {
            let result = RiskAssessment::initialize(
                env.clone(),
                admin.clone(),
                governance.clone(),
                coll_reg.clone(),
                loan_mgr.clone(),
                vault.clone(),
            );
            assert!(result.is_ok());

            // Verify admin is set
            let stored_admin = RiskAssessment::admin(env.clone());
            assert_eq!(stored_admin, admin);

            // Verify governance is set
            let stored_gov = RiskAssessment::governance(env.clone());
            assert_eq!(stored_gov, governance);

            // Verify default parameters
            let params = RiskAssessment::get_risk_parameters(env.clone());
            assert_eq!(params.liquidation_threshold, 8000);
            assert_eq!(params.liquidation_penalty, 500);
            assert_eq!(params.min_health_factor, 10000);

            // Verify not paused
            assert!(!RiskAssessment::is_paused(env.clone()));
        });
    }

    #[test]
    fn test_initialize_already_initialized() {
        let (env, admin, governance, coll_reg, loan_mgr, vault) = setup_env();
        let contract_id = env.register(RiskAssessment, ());

        env.mock_all_auths();

        env.as_contract(&contract_id, || {
            // First initialization
            let result = RiskAssessment::initialize(
                env.clone(),
                admin.clone(),
                governance.clone(),
                coll_reg.clone(),
                loan_mgr.clone(),
                vault.clone(),
            );
            assert!(result.is_ok());

            // Second initialization should fail
            let result2 = RiskAssessment::initialize(
                env.clone(),
                admin.clone(),
                governance.clone(),
                coll_reg.clone(),
                loan_mgr.clone(),
                vault.clone(),
            );
            assert_eq!(result2, Err(ContractError::AlreadyInitialized));
        });
    }

    // ========================================================================
    // Health Factor Tests
    // ========================================================================

    #[test]
    fn test_calculate_health_factor_healthy() {
        let (env, admin, governance, coll_reg, loan_mgr, vault) = setup_env();
        let contract_id = env.register(RiskAssessment, ());

        env.mock_all_auths();

        env.as_contract(&contract_id, || {
            RiskAssessment::initialize(
                env.clone(),
                admin.clone(),
                governance.clone(),
                coll_reg.clone(),
                loan_mgr.clone(),
                vault.clone(),
            ).unwrap();

            let position_id = 1u64;
            // Collateral: $10,000, Debt: $5,000 (with 5% interest = $5,250)
            // HF = (10000 * 8000) / 5250 = 15238 (healthy)
            let loan = create_test_loan(&env, position_id, 5000, 500);
            let collateral = create_test_collateral(&env, position_id, 10000);
            let escrow = create_test_escrow(&env, 5000);

            RiskAssessment::set_test_position(
                env.clone(),
                position_id,
                loan,
                collateral,
                escrow,
            );

            let health_factor = RiskAssessment::calculate_health_factor(env.clone(), position_id).unwrap();
            assert!(health_factor >= 15000); // Should be healthy

            let risk = RiskAssessment::get_position_risk(env.clone(), position_id).unwrap();
            assert_eq!(risk, PositionRisk::Healthy);
        });
    }

    #[test]
    fn test_calculate_health_factor_dynamic_valuation() {
        let (env, admin, governance, coll_reg, loan_mgr, vault) = setup_env();
        let contract_id = env.register(RiskAssessment, ());

        env.mock_all_auths();

        env.as_contract(&contract_id, || {
            RiskAssessment::initialize(
                env.clone(),
                admin.clone(),
                governance.clone(),
                coll_reg.clone(),
                loan_mgr.clone(),
                vault.clone(),
            ).unwrap();

            let position_id = 1u64;
            // Face value: $10,000, but Realized value: $6,000
            // Debt: $5,000 (with 5% interest = $5,250)
            // HF (using realized value) = (6000 * 8000) / 5250 = 9142 (liquidatable)
            let loan = create_test_loan(&env, position_id, 5000, 500);
            let mut collateral = create_test_collateral(&env, position_id, 10000);
            collateral.realized_value = 6000;
            let escrow = create_test_escrow(&env, 5000);

            RiskAssessment::set_test_position(
                env.clone(),
                position_id,
                loan,
                collateral,
                escrow,
            );

            let health_factor = RiskAssessment::calculate_health_factor(env.clone(), position_id).unwrap();
            assert!(health_factor < 10000); // Should be liquidatable due to low realized value

            let risk = RiskAssessment::get_position_risk(env.clone(), position_id).unwrap();
            assert_eq!(risk, PositionRisk::Liquidatable);
        });
    }

    #[test]
    fn test_calculate_health_factor_liquidatable() {
        let (env, admin, governance, coll_reg, loan_mgr, vault) = setup_env();
        let contract_id = env.register(RiskAssessment, ());

        env.mock_all_auths();

        env.as_contract(&contract_id, || {
            RiskAssessment::initialize(
                env.clone(),
                admin.clone(),
                governance.clone(),
                coll_reg.clone(),
                loan_mgr.clone(),
                vault.clone(),
            ).unwrap();

            let position_id = 1u64;
            // Collateral: $10,000, Debt: $8,500 (with 5% interest = $8,925)
            // HF = (10000 * 8000) / 8925 = 8963 (< 10000, liquidatable)
            let loan = create_test_loan(&env, position_id, 8500, 500);
            let collateral = create_test_collateral(&env, position_id, 10000);
            let escrow = create_test_escrow(&env, 8500);

            RiskAssessment::set_test_position(
                env.clone(),
                position_id,
                loan,
                collateral,
                escrow,
            );

            let health_factor = RiskAssessment::calculate_health_factor(env.clone(), position_id).unwrap();
            assert!(health_factor < 10000); // Should be liquidatable

            let risk = RiskAssessment::get_position_risk(env.clone(), position_id).unwrap();
            assert_eq!(risk, PositionRisk::Liquidatable);
        });
    }

    #[test]
    fn test_calculate_health_factor_warning() {
        let (env, admin, governance, coll_reg, loan_mgr, vault) = setup_env();
        let contract_id = env.register(RiskAssessment, ());

        env.mock_all_auths();

        env.as_contract(&contract_id, || {
            RiskAssessment::initialize(
                env.clone(),
                admin.clone(),
                governance.clone(),
                coll_reg.clone(),
                loan_mgr.clone(),
                vault.clone(),
            ).unwrap();

            let position_id = 1u64;
            // Collateral: $10,000, Debt: $6,000 (with 5% interest = $6,300)
            // HF = (10000 * 8000) / 6300 = 12698 (warning zone: 12000-15000)
            let loan = create_test_loan(&env, position_id, 6000, 500);
            let collateral = create_test_collateral(&env, position_id, 10000);
            let escrow = create_test_escrow(&env, 6000);

            RiskAssessment::set_test_position(
                env.clone(),
                position_id,
                loan,
                collateral,
                escrow,
            );

            let health_factor = RiskAssessment::calculate_health_factor(env.clone(), position_id).unwrap();
            assert!(health_factor >= 12000 && health_factor < 15000);

            let risk = RiskAssessment::get_position_risk(env.clone(), position_id).unwrap();
            assert_eq!(risk, PositionRisk::Warning);
        });
    }

    #[test]
    fn test_calculate_health_factor_danger() {
        let (env, admin, governance, coll_reg, loan_mgr, vault) = setup_env();
        let contract_id = env.register(RiskAssessment, ());

        env.mock_all_auths();

        env.as_contract(&contract_id, || {
            RiskAssessment::initialize(
                env.clone(),
                admin.clone(),
                governance.clone(),
                coll_reg.clone(),
                loan_mgr.clone(),
                vault.clone(),
            ).unwrap();

            let position_id = 1u64;
            // Collateral: $10,000, Debt: $7,200 (with 5% interest = $7,560)
            // HF = (10000 * 8000) / 7560 = 10582 (danger zone: 10000-12000)
            let loan = create_test_loan(&env, position_id, 7200, 500);
            let collateral = create_test_collateral(&env, position_id, 10000);
            let escrow = create_test_escrow(&env, 7200);

            RiskAssessment::set_test_position(
                env.clone(),
                position_id,
                loan,
                collateral,
                escrow,
            );

            let health_factor = RiskAssessment::calculate_health_factor(env.clone(), position_id).unwrap();
            assert!(health_factor >= 10000 && health_factor < 12000);

            let risk = RiskAssessment::get_position_risk(env.clone(), position_id).unwrap();
            assert_eq!(risk, PositionRisk::Danger);
        });
    }

    #[test]
    fn test_is_liquidatable() {
        let (env, admin, governance, coll_reg, loan_mgr, vault) = setup_env();
        let contract_id = env.register(RiskAssessment, ());

        env.mock_all_auths();

        env.as_contract(&contract_id, || {
            RiskAssessment::initialize(
                env.clone(),
                admin.clone(),
                governance.clone(),
                coll_reg.clone(),
                loan_mgr.clone(),
                vault.clone(),
            ).unwrap();

            let position_id = 1u64;
            // Liquidatable position
            let loan = create_test_loan(&env, position_id, 8500, 500);
            let collateral = create_test_collateral(&env, position_id, 10000);
            let escrow = create_test_escrow(&env, 8500);

            RiskAssessment::set_test_position(
                env.clone(),
                position_id,
                loan,
                collateral,
                escrow,
            );

            let is_liq = RiskAssessment::is_liquidatable(env.clone(), position_id).unwrap();
            assert!(is_liq);
        });
    }

    #[test]
    fn test_is_not_liquidatable_healthy() {
        let (env, admin, governance, coll_reg, loan_mgr, vault) = setup_env();
        let contract_id = env.register(RiskAssessment, ());

        env.mock_all_auths();

        env.as_contract(&contract_id, || {
            RiskAssessment::initialize(
                env.clone(),
                admin.clone(),
                governance.clone(),
                coll_reg.clone(),
                loan_mgr.clone(),
                vault.clone(),
            ).unwrap();

            let position_id = 1u64;
            // Healthy position
            let loan = create_test_loan(&env, position_id, 5000, 500);
            let collateral = create_test_collateral(&env, position_id, 10000);
            let escrow = create_test_escrow(&env, 5000);

            RiskAssessment::set_test_position(
                env.clone(),
                position_id,
                loan,
                collateral,
                escrow,
            );

            let is_liq = RiskAssessment::is_liquidatable(env.clone(), position_id).unwrap();
            assert!(!is_liq);
        });
    }

    #[test]
    fn test_get_position_data() {
        let (env, admin, governance, coll_reg, loan_mgr, vault) = setup_env();
        let contract_id = env.register(RiskAssessment, ());

        env.mock_all_auths();

        env.as_contract(&contract_id, || {
            RiskAssessment::initialize(
                env.clone(),
                admin.clone(),
                governance.clone(),
                coll_reg.clone(),
                loan_mgr.clone(),
                vault.clone(),
            ).unwrap();

            let position_id = 1u64;
            let loan = create_test_loan(&env, position_id, 5000, 500);
            let collateral = create_test_collateral(&env, position_id, 10000);
            let escrow = create_test_escrow(&env, 5000);

            RiskAssessment::set_test_position(
                env.clone(),
                position_id,
                loan.clone(),
                collateral.clone(),
                escrow,
            );

            let pos_data = RiskAssessment::get_position_data(env.clone(), position_id).unwrap();
            assert_eq!(pos_data.escrow_id, position_id);
            assert_eq!(pos_data.loan_id, loan.id);
            assert_eq!(pos_data.collateral_id, collateral.id);
            assert_eq!(pos_data.collateral_value, collateral.face_value);
            assert_eq!(pos_data.risk_status, PositionRisk::Healthy);
        });
    }

    // ========================================================================
    // Governance Tests
    // ========================================================================

    #[test]
    fn test_update_risk_parameters() {
        let (env, admin, governance, coll_reg, loan_mgr, vault) = setup_env();
        let contract_id = env.register(RiskAssessment, ());

        env.mock_all_auths();

        env.as_contract(&contract_id, || {
            RiskAssessment::initialize(
                env.clone(),
                admin.clone(),
                governance.clone(),
                coll_reg.clone(),
                loan_mgr.clone(),
                vault.clone(),
            ).unwrap();

            // Propose new parameters
            let new_params = RiskParameters {
                liquidation_threshold: 7500,
                liquidation_penalty: 600,
                min_health_factor: 11000,
                max_liquidation_ratio: 4000,
                grace_period: 7200,
                liquidator_bonus: 600,
            };

            let result = RiskAssessment::update_risk_parameters(env.clone(), new_params.clone());
            assert!(result.is_ok());

            // Check pending update exists
            let pending = RiskAssessment::get_pending_update(env.clone());
            assert!(pending.is_some());
            assert_eq!(pending.unwrap().new_params.liquidation_threshold, 7500);
        });
    }

    #[test]
    fn test_execute_parameter_update() {
        let (env, admin, governance, coll_reg, loan_mgr, vault) = setup_env();
        let contract_id = env.register(RiskAssessment, ());

        env.mock_all_auths();

        env.as_contract(&contract_id, || {
            RiskAssessment::initialize(
                env.clone(),
                admin.clone(),
                governance.clone(),
                coll_reg.clone(),
                loan_mgr.clone(),
                vault.clone(),
            ).unwrap();

            // Propose new parameters
            let new_params = RiskParameters {
                liquidation_threshold: 7500,
                liquidation_penalty: 600,
                min_health_factor: 11000,
                max_liquidation_ratio: 4000,
                grace_period: 7200,
                liquidator_bonus: 600,
            };

            RiskAssessment::update_risk_parameters(env.clone(), new_params.clone()).unwrap();

            // Try to execute before timelock - should fail
            let result = RiskAssessment::execute_parameter_update(env.clone());
            assert_eq!(result, Err(ContractError::TimelockNotExpired));

            // Advance time past timelock (24 hours + 1)
            env.ledger().set_timestamp(env.ledger().timestamp() + 86401);

            // Execute should succeed now
            let result = RiskAssessment::execute_parameter_update(env.clone());
            assert!(result.is_ok());

            // Verify new parameters are active
            let params = RiskAssessment::get_risk_parameters(env.clone());
            assert_eq!(params.liquidation_threshold, 7500);
            assert_eq!(params.liquidation_penalty, 600);
        });
    }

    #[test]
    fn test_cancel_parameter_update() {
        let (env, admin, governance, coll_reg, loan_mgr, vault) = setup_env();
        let contract_id = env.register(RiskAssessment, ());

        env.mock_all_auths();

        // Initialize
        env.as_contract(&contract_id, || {
            RiskAssessment::initialize(
                env.clone(),
                admin.clone(),
                governance.clone(),
                coll_reg.clone(),
                loan_mgr.clone(),
                vault.clone(),
            ).unwrap();
        });

        // Propose new parameters (separate block to avoid auth conflict)
        env.as_contract(&contract_id, || {
            let new_params = RiskParameters {
                liquidation_threshold: 7500,
                liquidation_penalty: 600,
                min_health_factor: 11000,
                max_liquidation_ratio: 4000,
                grace_period: 7200,
                liquidator_bonus: 600,
            };
            RiskAssessment::update_risk_parameters(env.clone(), new_params).unwrap();
        });

        // Cancel the update (separate block)
        env.as_contract(&contract_id, || {
            let result = RiskAssessment::cancel_parameter_update(env.clone());
            assert!(result.is_ok());

            // Verify no pending update
            let pending = RiskAssessment::get_pending_update(env.clone());
            assert!(pending.is_none());

            // Original parameters should still be active
            let params = RiskAssessment::get_risk_parameters(env.clone());
            assert_eq!(params.liquidation_threshold, 8000);
        });
    }

    #[test]
    fn test_invalid_parameters_threshold() {
        let (env, admin, governance, coll_reg, loan_mgr, vault) = setup_env();
        let contract_id = env.register(RiskAssessment, ());

        env.mock_all_auths();

        env.as_contract(&contract_id, || {
            RiskAssessment::initialize(
                env.clone(),
                admin.clone(),
                governance.clone(),
                coll_reg.clone(),
                loan_mgr.clone(),
                vault.clone(),
            ).unwrap();

            // Invalid threshold (too low)
            let invalid_params = RiskParameters {
                liquidation_threshold: 4000, // < 5000, invalid
                liquidation_penalty: 500,
                min_health_factor: 10000,
                max_liquidation_ratio: 5000,
                grace_period: 3600,
                liquidator_bonus: 500,
            };

            let result = RiskAssessment::update_risk_parameters(env.clone(), invalid_params);
            assert_eq!(result, Err(ContractError::InvalidThreshold));
        });
    }

    // ========================================================================
    // Emergency Control Tests
    // ========================================================================

    #[test]
    fn test_pause_liquidations() {
        let (env, admin, governance, coll_reg, loan_mgr, vault) = setup_env();
        let contract_id = env.register(RiskAssessment, ());

        env.mock_all_auths();

        env.as_contract(&contract_id, || {
            RiskAssessment::initialize(
                env.clone(),
                admin.clone(),
                governance.clone(),
                coll_reg.clone(),
                loan_mgr.clone(),
                vault.clone(),
            ).unwrap();

            // Initially not paused
            assert!(!RiskAssessment::is_paused(env.clone()));

            // Pause liquidations
            let result = RiskAssessment::pause_liquidations(env.clone());
            assert!(result.is_ok());

            // Should be paused now
            assert!(RiskAssessment::is_paused(env.clone()));
        });
    }

    #[test]
    fn test_unpause_liquidations() {
        let (env, admin, governance, coll_reg, loan_mgr, vault) = setup_env();
        let contract_id = env.register(RiskAssessment, ());

        env.mock_all_auths();

        // Initialize
        env.as_contract(&contract_id, || {
            RiskAssessment::initialize(
                env.clone(),
                admin.clone(),
                governance.clone(),
                coll_reg.clone(),
                loan_mgr.clone(),
                vault.clone(),
            ).unwrap();
        });

        // Pause (separate block)
        env.as_contract(&contract_id, || {
            RiskAssessment::pause_liquidations(env.clone()).unwrap();
            assert!(RiskAssessment::is_paused(env.clone()));
        });

        // Unpause (separate block)
        env.as_contract(&contract_id, || {
            let result = RiskAssessment::unpause_liquidations(env.clone());
            assert!(result.is_ok());
            assert!(!RiskAssessment::is_paused(env.clone()));
        });
    }

    // ========================================================================
    // Admin Function Tests
    // ========================================================================

    #[test]
    fn test_set_contract_addresses() {
        let (env, admin, governance, coll_reg, loan_mgr, vault) = setup_env();
        let contract_id = env.register(RiskAssessment, ());

        env.mock_all_auths();

        // Initialize
        env.as_contract(&contract_id, || {
            RiskAssessment::initialize(
                env.clone(),
                admin.clone(),
                governance.clone(),
                coll_reg.clone(),
                loan_mgr.clone(),
                vault.clone(),
            ).unwrap();
        });

        // Set collateral registry (separate block)
        env.as_contract(&contract_id, || {
            let new_coll_reg = Address::generate(&env);
            let result = RiskAssessment::set_collateral_registry(env.clone(), new_coll_reg.clone());
            assert!(result.is_ok());
        });

        // Set loan management (separate block)
        env.as_contract(&contract_id, || {
            let new_loan_mgr = Address::generate(&env);
            let result = RiskAssessment::set_loan_management(env.clone(), new_loan_mgr.clone());
            assert!(result.is_ok());
        });

        // Set vault (separate block)
        env.as_contract(&contract_id, || {
            let new_vault = Address::generate(&env);
            let result = RiskAssessment::set_vault(env.clone(), new_vault.clone());
            assert!(result.is_ok());
        });
    }

    #[test]
    fn test_set_timelock_duration() {
        let (env, admin, governance, coll_reg, loan_mgr, vault) = setup_env();
        let contract_id = env.register(RiskAssessment, ());

        env.mock_all_auths();

        env.as_contract(&contract_id, || {
            RiskAssessment::initialize(
                env.clone(),
                admin.clone(),
                governance.clone(),
                coll_reg.clone(),
                loan_mgr.clone(),
                vault.clone(),
            ).unwrap();

            let result = RiskAssessment::set_timelock_duration(env.clone(), 172800); // 48 hours
            assert!(result.is_ok());
        });
    }

    // ========================================================================
    // Edge Case Tests
    // ========================================================================

    #[test]
    fn test_loan_not_active() {
        let (env, admin, governance, coll_reg, loan_mgr, vault) = setup_env();
        let contract_id = env.register(RiskAssessment, ());

        env.mock_all_auths();

        env.as_contract(&contract_id, || {
            RiskAssessment::initialize(
                env.clone(),
                admin.clone(),
                governance.clone(),
                coll_reg.clone(),
                loan_mgr.clone(),
                vault.clone(),
            ).unwrap();

            let position_id = 1u64;
            let mut loan = create_test_loan(&env, position_id, 5000, 500);
            loan.status = LoanStatus::Repaid; // Not active
            let collateral = create_test_collateral(&env, position_id, 10000);
            let escrow = create_test_escrow(&env, 5000);

            RiskAssessment::set_test_position(
                env.clone(),
                position_id,
                loan,
                collateral,
                escrow,
            );

            let result = RiskAssessment::calculate_health_factor(env.clone(), position_id);
            assert_eq!(result, Err(ContractError::LoanNotActive));
        });
    }

    #[test]
    fn test_position_not_found() {
        let (env, admin, governance, coll_reg, loan_mgr, vault) = setup_env();
        let contract_id = env.register(RiskAssessment, ());

        env.mock_all_auths();

        env.as_contract(&contract_id, || {
            RiskAssessment::initialize(
                env.clone(),
                admin.clone(),
                governance.clone(),
                coll_reg.clone(),
                loan_mgr.clone(),
                vault.clone(),
            ).unwrap();

            // Try to get health factor for non-existent position
            let result = RiskAssessment::calculate_health_factor(env.clone(), 999);
            assert_eq!(result, Err(ContractError::LoanNotFound));
        });
    }

    #[test]
    fn test_no_pending_update() {
        let (env, admin, governance, coll_reg, loan_mgr, vault) = setup_env();
        let contract_id = env.register(RiskAssessment, ());

        env.mock_all_auths();

        env.as_contract(&contract_id, || {
            RiskAssessment::initialize(
                env.clone(),
                admin.clone(),
                governance.clone(),
                coll_reg.clone(),
                loan_mgr.clone(),
                vault.clone(),
            ).unwrap();

            // Try to execute without pending update
            let result = RiskAssessment::execute_parameter_update(env.clone());
            assert_eq!(result, Err(ContractError::NoPendingUpdate));

            // Try to cancel without pending update
            let result = RiskAssessment::cancel_parameter_update(env.clone());
            assert_eq!(result, Err(ContractError::NoPendingUpdate));
        });
    }
    // ========================================================================
    // Dutch Auction Tests
    // ========================================================================

    #[test]
    fn test_start_auction_success() {
        let (env, admin, governance, coll_reg, loan_mgr, vault) = setup_env();
        let contract_id = env.register(RiskAssessment, ());
        env.mock_all_auths();

        env.as_contract(&contract_id, || {
            RiskAssessment::initialize(
                env.clone(), admin.clone(), governance.clone(),
                coll_reg.clone(), loan_mgr.clone(), vault.clone(),
            ).unwrap();

            // Undercollateralised position
            let loan_id = 42u64;
            let loan = create_test_loan(&env, loan_id, 8_500, 500);
            let collateral = create_test_collateral(&env, loan_id, 10_000);
            let escrow = create_test_escrow(&env, 8_500);
            RiskAssessment::set_test_position(env.clone(), loan_id, loan, collateral, escrow);

            let state = RiskAssessment::start_auction(env.clone(), loan_id).unwrap();
            assert_eq!(state.status, AuctionStatus::Active);
            assert_eq!(state.collateral_value, 10_000);
            assert!(state.debt_floor > 0);
        });
    }

    #[test]
    fn test_start_auction_healthy_position_fails() {
        let (env, admin, governance, coll_reg, loan_mgr, vault) = setup_env();
        let contract_id = env.register(RiskAssessment, ());
        env.mock_all_auths();

        env.as_contract(&contract_id, || {
            RiskAssessment::initialize(
                env.clone(), admin.clone(), governance.clone(),
                coll_reg.clone(), loan_mgr.clone(), vault.clone(),
            ).unwrap();

            let loan_id = 1u64;
            let loan = create_test_loan(&env, loan_id, 5_000, 500);
            let collateral = create_test_collateral(&env, loan_id, 10_000);
            let escrow = create_test_escrow(&env, 5_000);
            RiskAssessment::set_test_position(env.clone(), loan_id, loan, collateral, escrow);

            let result = RiskAssessment::start_auction(env.clone(), loan_id);
            assert_eq!(result, Err(ContractError::PositionNotLiquidatable));
        });
    }

    #[test]
    fn test_start_auction_duplicate_fails() {
        let (env, admin, governance, coll_reg, loan_mgr, vault) = setup_env();
        let contract_id = env.register(RiskAssessment, ());
        env.mock_all_auths();

        env.as_contract(&contract_id, || {
            RiskAssessment::initialize(
                env.clone(), admin.clone(), governance.clone(),
                coll_reg.clone(), loan_mgr.clone(), vault.clone(),
            ).unwrap();

            let loan_id = 42u64;
            let loan = create_test_loan(&env, loan_id, 8_500, 500);
            let collateral = create_test_collateral(&env, loan_id, 10_000);
            let escrow = create_test_escrow(&env, 8_500);
            RiskAssessment::set_test_position(env.clone(), loan_id, loan, collateral, escrow);

            RiskAssessment::start_auction(env.clone(), loan_id).unwrap();
            let result = RiskAssessment::start_auction(env.clone(), loan_id);
            assert_eq!(result, Err(ContractError::AuctionAlreadyActive));
        });
    }

    #[test]
    fn test_get_auction_price_decays() {
        let (env, admin, governance, coll_reg, loan_mgr, vault) = setup_env();
        let contract_id = env.register(RiskAssessment, ());
        env.mock_all_auths();

        env.as_contract(&contract_id, || {
            RiskAssessment::initialize(
                env.clone(), admin.clone(), governance.clone(),
                coll_reg.clone(), loan_mgr.clone(), vault.clone(),
            ).unwrap();

            let loan_id = 42u64;
            let loan = create_test_loan(&env, loan_id, 8_500, 500);
            let collateral = create_test_collateral(&env, loan_id, 10_000);
            let escrow = create_test_escrow(&env, 8_500);
            RiskAssessment::set_test_position(env.clone(), loan_id, loan, collateral, escrow);

            RiskAssessment::start_auction(env.clone(), loan_id).unwrap();

            // Price at t=0 should equal collateral_value
            let price_start = RiskAssessment::get_auction_price(env.clone(), loan_id).unwrap();
            assert_eq!(price_start, 10_000);

            // Advance 3 hours → price should be lower
            env.ledger().set_timestamp(env.ledger().timestamp() + 10_800);
            let price_mid = RiskAssessment::get_auction_price(env.clone(), loan_id).unwrap();
            assert!(price_mid < price_start);

            // Advance past auction end → price == debt_floor
            env.ledger().set_timestamp(env.ledger().timestamp() + 100_000);
            let price_end = RiskAssessment::get_auction_price(env.clone(), loan_id).unwrap();
            let state = RiskAssessment::get_auction(env.clone(), loan_id).unwrap();
            assert_eq!(price_end, state.debt_floor);
        });
    }

    #[test]
    fn test_get_auction_config_default() {
        let (env, admin, governance, coll_reg, loan_mgr, vault) = setup_env();
        let contract_id = env.register(RiskAssessment, ());
        env.mock_all_auths();

        env.as_contract(&contract_id, || {
            RiskAssessment::initialize(
                env.clone(), admin.clone(), governance.clone(),
                coll_reg.clone(), loan_mgr.clone(), vault.clone(),
            ).unwrap();

            let cfg = RiskAssessment::get_auction_config(env.clone());
            assert_eq!(cfg.duration, 21_600);
            assert_eq!(cfg.auction_fee_bps, 50);
        });
    }

    #[test]
    fn test_set_auction_config_governance() {
        let (env, admin, governance, coll_reg, loan_mgr, vault) = setup_env();
        let contract_id = env.register(RiskAssessment, ());
        env.mock_all_auths();

        env.as_contract(&contract_id, || {
            RiskAssessment::initialize(
                env.clone(), admin.clone(), governance.clone(),
                coll_reg.clone(), loan_mgr.clone(), vault.clone(),
            ).unwrap();

            let new_cfg = AuctionConfig {
                duration: 43_200,
                decay_rate_bps_per_sec: 23,
                auction_fee_bps: 100,
            };
            RiskAssessment::set_auction_config(env.clone(), new_cfg).unwrap();

            let cfg = RiskAssessment::get_auction_config(env.clone());
            assert_eq!(cfg.duration, 43_200);
            assert_eq!(cfg.auction_fee_bps, 100);
        });
    }

    #[test]
    fn test_expire_auction() {
        let (env, admin, governance, coll_reg, loan_mgr, vault) = setup_env();
        let contract_id = env.register(RiskAssessment, ());
        env.mock_all_auths();

        env.as_contract(&contract_id, || {
            RiskAssessment::initialize(
                env.clone(), admin.clone(), governance.clone(),
                coll_reg.clone(), loan_mgr.clone(), vault.clone(),
            ).unwrap();

            let loan_id = 42u64;
            let loan = create_test_loan(&env, loan_id, 8_500, 500);
            let collateral = create_test_collateral(&env, loan_id, 10_000);
            let escrow = create_test_escrow(&env, 8_500);
            RiskAssessment::set_test_position(env.clone(), loan_id, loan, collateral, escrow);

            RiskAssessment::start_auction(env.clone(), loan_id).unwrap();

            // Expire before end → error
            let err = RiskAssessment::expire_auction(env.clone(), loan_id);
            assert_eq!(err, Err(ContractError::AuctionNotExpired));

            // Advance past end
            env.ledger().set_timestamp(env.ledger().timestamp() + 100_000);
            RiskAssessment::expire_auction(env.clone(), loan_id).unwrap();

            let state = RiskAssessment::get_auction(env.clone(), loan_id).unwrap();
            assert_eq!(state.status, AuctionStatus::Expired);
        });
    }
}

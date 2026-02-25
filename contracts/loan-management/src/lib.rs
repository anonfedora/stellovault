//! Loan Management Contract for StelloVault
//!
//! This contract manages the lifecycle of loans backed by escrowed collateral.
//! It handles loan issuance, repayment tracking, and default enforcement.

#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol, Val};

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoanStatus {
    Active = 0,
    Repaid = 1,
    Defaulted = 2,
    Liquidated = 3,
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContractError {
    Unauthorized = 1,
    AlreadyInitialized = 2,
    LoanNotFound = 3,
    LoanAlreadyIssued = 4,
    LoanNotActive = 5,
    DeadlineNotPassed = 6,
    DeadlinePassed = 7,
    InsufficientAmount = 8,
    InvalidRateParameters = 9,
    RiskEngineNotSet = 10,
    MathOverflow = 11,
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

/// Dynamic interest rate parameters
#[contracttype]
#[derive(Clone, Debug)]
pub struct RateParameters {
    /// Base interest rate in basis points (e.g., 200 = 2%)
    pub base_rate: u32,
    /// Risk premium multiplier in basis points (e.g., 100 = 1% per risk unit)
    pub risk_premium: u32,
    /// Utilization slope parameter in basis points (e.g., 50 = 0.5% per 10% utilization)
    pub slope_parameter: u32,
    /// Maximum interest rate cap in basis points (e.g., 5000 = 50%)
    pub max_rate: u32,
}

impl RateParameters {
    pub fn default() -> Self {
        Self {
            base_rate: 200,      // 2%
            risk_premium: 100,   // 1% per risk unit
            slope_parameter: 50, // 0.5% per 10% utilization
            max_rate: 5000,      // 50% cap
        }
    }
}

/// Risk score from RiskAssessment contract
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PositionRisk {
    Healthy = 0,
    Warning = 1,
    Danger = 2,
    Liquidatable = 3,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct Loan {
    pub id: u64,
    pub escrow_id: u64,
    pub borrower: Address,
    pub lender: Address,
    pub amount: i128,
    pub interest_rate: u32, // Basis points (e.g., 500 = 5%)
    pub deadline: u64,
    pub status: LoanStatus,
    pub principal_repaid: i128,
    pub interest_repaid: i128,
    pub last_repayment_ts: u64,
}

#[contract]
pub struct LoanManagement;

#[contractimpl]
impl LoanManagement {
    /// Initialize the contract with admin address
    pub fn initialize(env: Env, admin: Address) -> Result<(), ContractError> {
        if env.storage().instance().has(&symbol_short!("admin")) {
            return Err(ContractError::AlreadyInitialized);
        }
        env.storage()
            .instance()
            .set(&symbol_short!("admin"), &admin);
        env.storage()
            .instance()
            .set(&symbol_short!("next_id"), &1u64);

        // Initialize default rate parameters
        let default_params = RateParameters::default();
        env.storage()
            .instance()
            .set(&symbol_short!("rate_prm"), &default_params);

        // Initialize total liquidity tracking
        env.storage()
            .instance()
            .set(&symbol_short!("tot_liq"), &0i128);
        env.storage()
            .instance()
            .set(&symbol_short!("tot_bor"), &0i128);

        Ok(())
    }

    /// Calculate dynamic interest rate based on risk and utilization
    ///
    /// Formula: rate = base_rate + (risk_premium * risk_factor) + (utilization * slope_parameter)
    ///
    /// # Arguments
    /// * `borrower` - Address of the borrower
    /// * `amount` - Loan amount to calculate rate for
    ///
    /// # Returns
    /// Dynamic interest rate in basis points
    pub fn get_dynamic_rate(
        env: Env,
        borrower: Address,
        amount: i128,
    ) -> Result<u32, ContractError> {
        let rate_params: RateParameters = env
            .storage()
            .instance()
            .get(&symbol_short!("rate_prm"))
            .unwrap_or(RateParameters::default());

        // Get risk score from RiskAssessment contract
        let risk_factor = Self::get_borrower_risk_factor(&env, &borrower)?;

        // Calculate utilization ratio
        let utilization_bps = Self::calculate_utilization(&env, amount)?;

        // Calculate dynamic rate: base_rate + (risk_premium * risk_factor) + (utilization * slope_parameter / 1000)
        let risk_component = rate_params
            .risk_premium
            .checked_mul(risk_factor)
            .ok_or(ContractError::MathOverflow)?;

        let utilization_component = utilization_bps
            .checked_mul(rate_params.slope_parameter)
            .ok_or(ContractError::MathOverflow)?
            .checked_div(1000)
            .unwrap_or(0);

        let total_rate = rate_params
            .base_rate
            .checked_add(risk_component)
            .ok_or(ContractError::MathOverflow)?
            .checked_add(utilization_component)
            .ok_or(ContractError::MathOverflow)?;

        // Cap at max_rate
        let final_rate = if total_rate > rate_params.max_rate {
            rate_params.max_rate
        } else {
            total_rate
        };

        Ok(final_rate)
    }

    /// Get borrower's risk factor from RiskAssessment contract
    ///
    /// Maps PositionRisk enum to numeric risk factor:
    /// - Healthy: 0
    /// - Warning: 1
    /// - Danger: 2
    /// - Liquidatable: 3
    fn get_borrower_risk_factor(env: &Env, _borrower: &Address) -> Result<u32, ContractError> {
        let risk_engine: Option<Address> = env.storage().instance().get(&symbol_short!("risk_eng"));

        if risk_engine.is_none() {
            // If no risk engine set, use default risk factor of 1 (Warning)
            return Ok(1);
        }

        // Try to get the borrower's position risk
        // We'll use a simple approach: check if borrower has any active positions
        // In a real implementation, this would query the RiskAssessment contract
        // For now, we'll return a default risk factor
        // TODO: Implement cross-contract call to RiskAssessment::get_position_risk

        // Placeholder: return default risk factor
        Ok(1)
    }

    /// Calculate protocol utilization ratio in basis points
    ///
    /// Utilization = (total_borrowed / total_liquidity) * 10000
    fn calculate_utilization(env: &Env, new_loan_amount: i128) -> Result<u32, ContractError> {
        let total_liquidity: i128 = env
            .storage()
            .instance()
            .get(&symbol_short!("tot_liq"))
            .unwrap_or(0);

        let total_borrowed: i128 = env
            .storage()
            .instance()
            .get(&symbol_short!("tot_bor"))
            .unwrap_or(0);

        // If no liquidity, return 0 utilization
        if total_liquidity == 0 {
            return Ok(0);
        }

        // Calculate new total borrowed including this loan
        let new_total_borrowed = total_borrowed
            .checked_add(new_loan_amount)
            .ok_or(ContractError::MathOverflow)?;

        // Calculate utilization in basis points: (borrowed / liquidity) * 10000
        let utilization = (new_total_borrowed
            .checked_mul(10000)
            .ok_or(ContractError::MathOverflow)?)
        .checked_div(total_liquidity)
        .unwrap_or(0);

        // Cap at 10000 (100%)
        let utilization_u32 = if utilization > 10000 {
            10000u32
        } else {
            utilization as u32
        };

        Ok(utilization_u32)
    }

    /// Update total liquidity (callable by admin or governance)
    pub fn update_total_liquidity(env: Env, new_liquidity: i128) -> Result<(), ContractError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&symbol_short!("admin"))
            .ok_or(ContractError::Unauthorized)?;

        admin.require_auth();

        env.storage()
            .instance()
            .set(&symbol_short!("tot_liq"), &new_liquidity);

        env.events()
            .publish((symbol_short!("liq_upd"),), (new_liquidity,));

        Ok(())
    }

    /// Get current rate parameters
    pub fn get_rate_parameters(env: Env) -> RateParameters {
        env.storage()
            .instance()
            .get(&symbol_short!("rate_prm"))
            .unwrap_or(RateParameters::default())
    }

    /// Update rate parameters (governance only)
    pub fn update_rate_parameters(
        env: Env,
        new_params: RateParameters,
    ) -> Result<(), ContractError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&symbol_short!("admin"))
            .ok_or(ContractError::Unauthorized)?;

        admin.require_auth();

        // Validate parameters
        if new_params.base_rate > new_params.max_rate {
            return Err(ContractError::InvalidRateParameters);
        }

        if new_params.max_rate > 10000 {
            // Cap at 100%
            return Err(ContractError::InvalidRateParameters);
        }

        env.storage()
            .instance()
            .set(&symbol_short!("rate_prm"), &new_params);

        env.events().publish(
            (symbol_short!("rate_upd"),),
            (
                new_params.base_rate,
                new_params.risk_premium,
                new_params.slope_parameter,
                new_params.max_rate,
            ),
        );

        Ok(())
    }

    /// Get protocol utilization statistics
    pub fn get_utilization_stats(env: Env) -> (i128, i128, u32) {
        let total_liquidity: i128 = env
            .storage()
            .instance()
            .get(&symbol_short!("tot_liq"))
            .unwrap_or(0);

        let total_borrowed: i128 = env
            .storage()
            .instance()
            .get(&symbol_short!("tot_bor"))
            .unwrap_or(0);

        let utilization_bps = if total_liquidity > 0 {
            let util = (total_borrowed * 10000) / total_liquidity;
            if util > 10000 {
                10000u32
            } else {
                util as u32
            }
        } else {
            0u32
        };

        (total_liquidity, total_borrowed, utilization_bps)
    }

    /// Issue a new loan backed by an escrow with dynamic interest rate
    ///
    /// # Arguments
    /// * `escrow_id` - The unique identifier of the escrowed collateral
    /// * `borrower` - Address of the borrower
    /// * `lender` - Address of the lender
    /// * `amount` - Loan amount
    /// * `duration` - Duration in seconds
    ///
    /// # Returns
    /// Loan ID and calculated interest rate
    pub fn issue_loan(
        env: Env,
        escrow_id: u64,
        borrower: Address,
        lender: Address,
        amount: i128,
        duration: u64,
    ) -> Result<(u64, u32), ContractError> {
        lender.require_auth();

        // Prevent multiple loans per escrow
        let escrow_key = (symbol_short!("escrow"), escrow_id);
        if env.storage().persistent().has(&escrow_key) {
            return Err(ContractError::LoanAlreadyIssued);
        }

        // Calculate dynamic interest rate
        let interest_rate = Self::get_dynamic_rate(env.clone(), borrower.clone(), amount)?;

        let loan_id: u64 = env
            .storage()
            .instance()
            .get(&symbol_short!("next_id"))
            .unwrap_or(1);

        let current_ts = env.ledger().timestamp();
        let deadline = current_ts
            .checked_add(duration)
            .ok_or(ContractError::MathOverflow)?;

        let loan = Loan {
            id: loan_id,
            escrow_id,
            borrower: borrower.clone(),
            lender: lender.clone(),
            amount,
            interest_rate,
            deadline,
            status: LoanStatus::Active,
            principal_repaid: 0,
            interest_repaid: 0,
            last_repayment_ts: current_ts,
        };

        // Store loan by ID
        env.storage().persistent().set(&loan_id, &loan);
        // Map escrow to loan ID to prevent duplicates
        env.storage().persistent().set(&escrow_key, &loan_id);

        env.storage()
            .instance()
            .set(&symbol_short!("next_id"), &(loan_id + 1));

        // Update total borrowed
        let total_borrowed: i128 = env
            .storage()
            .instance()
            .get(&symbol_short!("tot_bor"))
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&symbol_short!("tot_bor"), &(total_borrowed + amount));

        // Emit LoanIssued event with dynamic rate
        env.events().publish(
            (symbol_short!("loan_iss"),),
            (
                loan_id,
                escrow_id,
                borrower,
                lender,
                amount,
                interest_rate,
                deadline,
            ),
        );

        Ok((loan_id, interest_rate))
    }

    /// Repay an active loan (supports partial repayments)
    ///
    /// Payment is applied first to accrued interest, then to principal.
    /// Loan transitions to Repaid only when the full principal is paid off.
    pub fn repay_loan(env: Env, loan_id: u64, amount: i128) -> Result<(), ContractError> {
        let mut loan: Loan = env
            .storage()
            .persistent()
            .get(&loan_id)
            .ok_or(ContractError::LoanNotFound)?;

        loan.borrower.require_auth();

        if loan.status != LoanStatus::Active {
            return Err(ContractError::LoanNotActive);
        }

        if amount <= 0 {
            return Err(ContractError::InsufficientAmount);
        }

        let current_ts = env.ledger().timestamp();
        if current_ts > loan.deadline {
            return Err(ContractError::DeadlinePassed);
        }

        // Calculate total repayment: principal + interest
        let interest = (loan.amount * (loan.interest_rate as i128)) / 10000;
        let total_due = loan.amount + interest;

        let interest_accrued = (principal_remaining * (loan.interest_rate as i128) * (elapsed as i128))
            / ((seconds_per_year as i128) * 10000);

        let interest_outstanding = interest_accrued;

        // Apply payment: interest first, then principal
        let mut remaining_payment = amount;

        // Pay off interest
        let interest_payment = if remaining_payment >= interest_outstanding {
            interest_outstanding
        } else {
            remaining_payment
        };
        remaining_payment -= interest_payment;
        loan.interest_repaid += interest_payment;

        // Pay off principal with whatever is left
        let principal_payment = if remaining_payment >= principal_remaining {
            principal_remaining
        } else {
            remaining_payment
        };
        loan.principal_repaid += principal_payment;

        // Update last repayment timestamp
        loan.last_repayment_ts = current_ts;

        // Check if fully repaid
        if loan.principal_repaid >= loan.amount {
            loan.status = LoanStatus::Repaid;
        }

        // Calculate protocol fee if treasury is configured
        let treasury_opt: Option<Address> =
            env.storage().instance().get(&symbol_short!("treasury"));

        let protocol_fee = if treasury_opt.is_some() {
            // Query fee_bps from ProtocolTreasury
            let treasury = treasury_opt.as_ref().unwrap();
            let fee_bps_args: soroban_sdk::Vec<Val> = soroban_sdk::Vec::new(&env);
            let fee_bps: u32 = env.invoke_contract(
                treasury,
                &Symbol::new(&env, "get_fee_bps"),
                fee_bps_args,
            );
            (total_due * fee_bps as i128) / 10000
        } else {
            0i128
        };

        loan.status = LoanStatus::Repaid;
        env.storage().persistent().set(&loan_id, &loan);

        // Update total borrowed (decrease by principal paid)
        if principal_payment > 0 {
            let total_borrowed: i128 = env
                .storage()
                .instance()
                .get(&symbol_short!("tot_bor"))
                .unwrap_or(0);
            let new_borrowed = total_borrowed.saturating_sub(principal_payment);
            env.storage()
                .instance()
                .set(&symbol_short!("tot_bor"), &new_borrowed);
        }

        // Emit LoanRepaid event including protocol fee
        env.events()
            .publish((symbol_short!("loan_rep"),), (loan_id, amount, protocol_fee));

        Ok(())
    }

    /// Get total amount currently due on a loan (principal remaining + accrued interest)
    pub fn get_total_due(env: Env, loan_id: u64) -> Result<i128, ContractError> {
        let loan: Loan = env
            .storage()
            .persistent()
            .get(&loan_id)
            .ok_or(ContractError::LoanNotFound)?;

        if loan.status != LoanStatus::Active {
            return Ok(0);
        }

        let seconds_per_year: u64 = 31_557_600;
        let current_ts = env.ledger().timestamp();
        let elapsed = current_ts - loan.last_repayment_ts;
        let principal_remaining = loan.amount - loan.principal_repaid;

        let interest_accrued = (principal_remaining * (loan.interest_rate as i128) * (elapsed as i128))
            / ((seconds_per_year as i128) * 10000);

        Ok(principal_remaining + interest_accrued)
    }

    /// Mark a loan as defaulted if the deadline has passed
    pub fn mark_default(env: Env, loan_id: u64) -> Result<(), ContractError> {
        let mut loan: Loan = env
            .storage()
            .persistent()
            .get(&loan_id)
            .ok_or(ContractError::LoanNotFound)?;

        if loan.status != LoanStatus::Active {
            return Err(ContractError::LoanNotActive);
        }

        let current_ts = env.ledger().timestamp();
        if current_ts <= loan.deadline {
            return Err(ContractError::DeadlineNotPassed);
        }

        loan.status = LoanStatus::Defaulted;
        env.storage().persistent().set(&loan_id, &loan);

        // Emit LoanDefaulted event
        env.events()
            .publish((symbol_short!("loan_def"),), (loan_id,));

        // Trigger collateral liquidation (logic would go here)
        // For this task, we emit the event and update status.
        // Actual liquidation might involve calling another contract.

        Ok(())
    }

    /// Mark a loan as liquidated by the risk assessment engine
    ///
    /// # Arguments
    /// * `loan_id` - The loan ID to mark as liquidated
    /// * `liquidator` - Address of the liquidator who executed the liquidation
    ///
    /// # Authorization
    /// Only callable by the registered risk engine contract
    pub fn mark_liquidated(
        env: Env,
        loan_id: u64,
        liquidator: Address,
    ) -> Result<(), ContractError> {
        // Verify caller is the risk engine
        let risk_engine: Address = env
            .storage()
            .instance()
            .get(&symbol_short!("risk_eng"))
            .ok_or(ContractError::Unauthorized)?;

        risk_engine.require_auth();

        let mut loan: Loan = env
            .storage()
            .persistent()
            .get(&loan_id)
            .ok_or(ContractError::LoanNotFound)?;

        if loan.status != LoanStatus::Active {
            return Err(ContractError::LoanNotActive);
        }

        loan.status = LoanStatus::Liquidated;
        env.storage().persistent().set(&loan_id, &loan);

        // Emit LoanLiquidated event
        env.events()
            .publish((symbol_short!("loan_liq"),), (loan_id, liquidator));

        Ok(())
    }

    /// Set the risk engine contract address
    ///
    /// # Arguments
    /// * `risk_engine` - Address of the risk assessment contract
    ///
    /// # Authorization
    /// Only callable by admin
    pub fn set_risk_engine(env: Env, risk_engine: Address) -> Result<(), ContractError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&symbol_short!("admin"))
            .ok_or(ContractError::Unauthorized)?;

        admin.require_auth();

        env.storage()
            .instance()
            .set(&symbol_short!("risk_eng"), &risk_engine);

        // Emit RiskEngineSet event
        env.events()
            .publish((symbol_short!("risk_set"),), (risk_engine,));

        Ok(())
    }

    /// Get the registered risk engine address
    pub fn get_risk_engine(env: Env) -> Option<Address> {
        env.storage().instance().get(&symbol_short!("risk_eng"))
    }

    /// Get loan details
    pub fn get_loan(env: Env, loan_id: u64) -> Option<Loan> {
        env.storage().persistent().get(&loan_id)
    }

    /// Set the ProtocolTreasury address. Admin only.
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

    /// Get the registered treasury address.
    pub fn get_treasury(env: Env) -> Option<Address> {
        env.storage().instance().get(&symbol_short!("treasury"))
    }

    /// Get loan ID for an escrow
    pub fn get_loan_id_by_escrow(env: Env, escrow_id: u64) -> Option<u64> {
        env.storage()
            .persistent()
            .get(&(symbol_short!("escrow"), escrow_id))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Address as _, testutils::Ledger as _, Env};

    fn setup_env() -> (Env, LoanManagementClient<'static>, Address, Address, Address) {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let borrower = Address::generate(&env);
        let lender = Address::generate(&env);

        let contract_id = env.register(LoanManagement, ());
        let client = LoanManagementClient::new(&env, &contract_id);
        client.initialize(&admin);
        client.update_total_liquidity(&100_000);

        // Leak to get 'static lifetime for tests
        let client = unsafe {
            core::mem::transmute::<LoanManagementClient<'_>, LoanManagementClient<'static>>(client)
        };
        (env, client, admin, borrower, lender)
    }

    #[test]
    fn test_initialize() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let contract_id = env.register(LoanManagement, ());
        let client = LoanManagementClient::new(&env, &contract_id);

        client.initialize(&admin);

        env.as_contract(&contract_id, || {
            let stored_admin: Address = env
                .storage()
                .instance()
                .get(&symbol_short!("admin"))
                .unwrap();
            assert_eq!(stored_admin, admin);
        });
    }

    #[test]
    fn test_issue_loan_success() {
        let (_env, client, _admin, borrower, lender) = setup_env();

        let escrow_id = 1u64;
        let amount = 1000i128;
        let duration = 3600u64; // 1 hour

        let (loan_id, interest_rate) =
            client.issue_loan(&escrow_id, &borrower, &lender, &amount, &duration);
        assert_eq!(loan_id, 1);
        assert!(interest_rate > 0);

        let loan = client.get_loan(&loan_id).unwrap();
        assert_eq!(loan.borrower, borrower);
        assert_eq!(loan.lender, lender);
        assert_eq!(loan.amount, amount);
        assert_eq!(loan.status, LoanStatus::Active);
        assert_eq!(loan.interest_rate, interest_rate);
        assert_eq!(loan.principal_repaid, 0);
        assert_eq!(loan.interest_repaid, 0);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #4)")]
    fn test_issue_loan_duplicate_escrow() {
        let (_env, client, _admin, borrower, lender) = setup_env();

        let escrow_id = 1u64;
        client.issue_loan(&escrow_id, &borrower, &lender, &1000, &3600);

        // Should fail
        client.issue_loan(&escrow_id, &borrower, &lender, &1000, &3600);
    }

    #[test]
    fn test_repay_loan_full() {
        let (env, client, _admin, borrower, lender) = setup_env();

        let (loan_id, _) = client.issue_loan(&1, &borrower, &lender, &1000, &3600);

        // Advance time by 1800 seconds (half an hour)
        env.ledger().with_mut(|li| {
            li.timestamp += 1800;
        });

        // Get total due and pay it all
        let total_due = client.get_total_due(&loan_id);
        client.repay_loan(&loan_id, &total_due);

        let loan = client.get_loan(&loan_id).unwrap();
        assert_eq!(loan.status, LoanStatus::Repaid);
        assert_eq!(loan.principal_repaid, 1000);
    }

    #[test]
    fn test_partial_repayment_keeps_active() {
        let (env, client, _admin, borrower, lender) = setup_env();

        let (loan_id, interest_rate) =
            client.issue_loan(&1, &borrower, &lender, &10000, &31_557_600);

        // Advance 1 year so interest accrues
        env.ledger().with_mut(|li| {
            li.timestamp += 31_557_600;
        });

        // interest after 1 year = 10000 * rate / 10000
        let expected_interest = (10000i128 * interest_rate as i128) / 10000;

        // Pay amount less than interest - should all go to interest
        let payment = expected_interest / 2;
        client.repay_loan(&loan_id, &payment);

        let loan = client.get_loan(&loan_id).unwrap();
        assert_eq!(loan.status, LoanStatus::Active);
        assert_eq!(loan.interest_repaid, payment);
        assert_eq!(loan.principal_repaid, 0);
    }

    #[test]
    fn test_multiple_partial_repayments() {
        let (env, client, _admin, borrower, lender) = setup_env();

        let (loan_id, interest_rate) =
            client.issue_loan(&1, &borrower, &lender, &10000, &31_557_600);

        // Advance 1 year
        env.ledger().with_mut(|li| {
            li.timestamp += 31_557_600;
        });

        let interest_1yr = (10000i128 * interest_rate as i128) / 10000;

        // First payment: pay all interest + 100 principal
        let first_payment = interest_1yr + 100;
        client.repay_loan(&loan_id, &first_payment);
        let loan = client.get_loan(&loan_id).unwrap();
        assert_eq!(loan.interest_repaid, interest_1yr);
        assert_eq!(loan.principal_repaid, 100);
        assert_eq!(loan.status, LoanStatus::Active);

        // No more time passes (so no new interest accrues)
        // Second payment: pay remaining principal (9900)
        client.repay_loan(&loan_id, &9900);
        let loan = client.get_loan(&loan_id).unwrap();
        assert_eq!(loan.principal_repaid, 10000);
        assert_eq!(loan.status, LoanStatus::Repaid);
    }

    #[test]
    fn test_get_total_due() {
        let (env, client, _admin, borrower, lender) = setup_env();

        let (loan_id, interest_rate) =
            client.issue_loan(&1, &borrower, &lender, &10000, &31_557_600);

        // At issuance (no time elapsed), total due is just principal
        let total = client.get_total_due(&loan_id);
        assert_eq!(total, 10000);

        // After 1 year, total due = principal + accrued interest
        env.ledger().with_mut(|li| {
            li.timestamp += 31_557_600;
        });

        let expected_interest = (10000i128 * interest_rate as i128) / 10000;
        let total = client.get_total_due(&loan_id);
        assert_eq!(total, 10000 + expected_interest);
    }

    #[test]
    fn test_get_total_due_after_partial_repayment() {
        let (env, client, _admin, borrower, lender) = setup_env();

        let (loan_id, interest_rate) =
            client.issue_loan(&1, &borrower, &lender, &10000, &(31_557_600 * 2));

        // After 1 year: interest accrues
        env.ledger().with_mut(|li| {
            li.timestamp += 31_557_600;
        });

        let interest_1yr = (10000i128 * interest_rate as i128) / 10000;

        // Pay interest + 2000 principal
        let payment = interest_1yr + 2000;
        client.repay_loan(&loan_id, &payment);

        // Immediately after payment, total due = 8000 (remaining principal, no new interest)
        let total = client.get_total_due(&loan_id);
        assert_eq!(total, 8000);

        // After another year: interest on 8000
        env.ledger().with_mut(|li| {
            li.timestamp += 31_557_600;
        });

        let interest_on_8000 = (8000i128 * interest_rate as i128) / 10000;
        let total = client.get_total_due(&loan_id);
        assert_eq!(total, 8000 + interest_on_8000);
    }

    #[test]
    fn test_mark_default_success() {
        let (env, client, _admin, borrower, lender) = setup_env();

        let duration = 3600u64;
        let (loan_id, _) = client.issue_loan(&1, &borrower, &lender, &1000, &duration);

        env.ledger().with_mut(|li| {
            li.timestamp += duration + 1;
        });

        client.mark_default(&loan_id);

        let loan = client.get_loan(&loan_id).unwrap();
        assert_eq!(loan.status, LoanStatus::Defaulted);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #6)")]
    fn test_mark_default_too_early() {
        let (_env, client, _admin, borrower, lender) = setup_env();

        let (loan_id, _) = client.issue_loan(&1, &borrower, &lender, &1000, &3600);

        // Try to mark default before deadline
        client.mark_default(&loan_id);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #7)")]
    fn test_repay_loan_after_deadline() {
        let (env, client, _admin, borrower, lender) = setup_env();

        let duration = 3600u64;
        let (loan_id, _) = client.issue_loan(&1, &borrower, &lender, &1000, &duration);

        env.ledger().with_mut(|li| {
            li.timestamp += duration + 1;
        });

        client.repay_loan(&loan_id, &1050);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #5)")]
    fn test_repay_loan_already_repaid() {
        let (_env, client, _admin, borrower, lender) = setup_env();

        let (loan_id, _) = client.issue_loan(&1, &borrower, &lender, &1000, &3600);

        // No time elapsed, so no interest. Pay full principal.
        client.repay_loan(&loan_id, &1000);

        // Try to repay again
        client.repay_loan(&loan_id, &1000);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #5)")]
    fn test_mark_default_already_repaid() {
        let (env, client, _admin, borrower, lender) = setup_env();

        let duration = 3600u64;
        let (loan_id, _) = client.issue_loan(&1, &borrower, &lender, &1000, &duration);

        // No time elapsed, pay full principal
        client.repay_loan(&loan_id, &1000);

        env.ledger().with_mut(|li| {
            li.timestamp += duration + 1;
        });

        // Should fail because status is already Repaid
        client.mark_default(&loan_id);
    }

    #[test]
    fn test_get_loan_not_found() {
        let env = Env::default();
        let contract_id = env.register(LoanManagement, ());
        let client = LoanManagementClient::new(&env, &contract_id);

        let loan = client.get_loan(&999);
        assert!(loan.is_none());
    }

    #[test]
    fn test_get_loan_id_by_escrow_not_found() {
        let env = Env::default();
        let contract_id = env.register(LoanManagement, ());
        let client = LoanManagementClient::new(&env, &contract_id);

        let loan_id = client.get_loan_id_by_escrow(&999);
        assert!(loan_id.is_none());
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #2)")]
    fn test_initialize_already_initialized() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let contract_id = env.register(LoanManagement, ());
        let client = LoanManagementClient::new(&env, &contract_id);

        client.initialize(&admin);
        client.initialize(&admin);
    }

    #[test]
    fn test_set_risk_engine() {
        let (env, client, _admin, _borrower, _lender) = setup_env();

        let risk_engine = Address::generate(&env);
        client.set_risk_engine(&risk_engine);

        let stored_engine = client.get_risk_engine();
        assert_eq!(stored_engine, Some(risk_engine));
    }

    #[test]
    fn test_mark_liquidated_success() {
        let (env, client, _admin, borrower, lender) = setup_env();

        let risk_engine = Address::generate(&env);
        let liquidator = Address::generate(&env);

        client.set_risk_engine(&risk_engine);

        let (loan_id, _) = client.issue_loan(&1, &borrower, &lender, &1000, &3600);
        client.mark_liquidated(&loan_id, &liquidator);

        let loan = client.get_loan(&loan_id).unwrap();
        assert_eq!(loan.status, LoanStatus::Liquidated);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #1)")]
    fn test_mark_liquidated_no_risk_engine() {
        let (env, client, _admin, borrower, lender) = setup_env();

        let liquidator = Address::generate(&env);

        let (loan_id, _) = client.issue_loan(&1, &borrower, &lender, &1000, &3600);

        // Should fail - no risk engine set
        client.mark_liquidated(&loan_id, &liquidator);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #5)")]
    fn test_mark_liquidated_not_active() {
        let (env, client, _admin, borrower, lender) = setup_env();

        let risk_engine = Address::generate(&env);
        let liquidator = Address::generate(&env);

        client.set_risk_engine(&risk_engine);

        let (loan_id, _) = client.issue_loan(&1, &borrower, &lender, &1000, &3600);

        // Repay the loan first (no time elapsed, pay full principal)
        client.repay_loan(&loan_id, &1000);

        // Should fail - loan is already repaid
        client.mark_liquidated(&loan_id, &liquidator);
    }

    #[test]
    fn test_repay_zero_interest_at_issuance() {
        let (_env, client, _admin, borrower, lender) = setup_env();

        // No time passes, so no interest. Full principal payment should mark as Repaid.
        let (loan_id, _) = client.issue_loan(&1, &borrower, &lender, &5000, &3600);
        client.repay_loan(&loan_id, &5000);

        let loan = client.get_loan(&loan_id).unwrap();
        assert_eq!(loan.status, LoanStatus::Repaid);
        assert_eq!(loan.principal_repaid, 5000);
        assert_eq!(loan.interest_repaid, 0);
    }

    #[test]
    fn test_overpayment_caps_at_principal() {
        let (_env, client, _admin, borrower, lender) = setup_env();

        let (loan_id, _) = client.issue_loan(&1, &borrower, &lender, &1000, &3600);

        // Pay way more than needed
        client.repay_loan(&loan_id, &99999);

        let loan = client.get_loan(&loan_id).unwrap();
        assert_eq!(loan.status, LoanStatus::Repaid);
        assert_eq!(loan.principal_repaid, 1000);
    }

    #[test]
    fn test_dynamic_rate_calculation() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let borrower = Address::generate(&env);

        let contract_id = env.register(LoanManagement, ());
        let client = LoanManagementClient::new(&env, &contract_id);

        client.initialize(&admin);

        // Set liquidity for utilization calculation
        client.update_total_liquidity(&10000);

        // Get dynamic rate for a loan
        let rate = client.get_dynamic_rate(&borrower, &1000);

        // Rate should be > 0 and include base_rate + risk_premium + utilization component
        assert!(rate > 0);

        // With default params: base_rate=200, risk_premium=100, risk_factor=1
        // utilization = 1000/10000 = 10% = 1000 bps
        // utilization_component = 1000 * 50 / 1000 = 50
        // Expected: 200 + 100 + 50 = 350
        assert_eq!(rate, 350);
    }

    #[test]
    fn test_update_rate_parameters() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);

        let contract_id = env.register(LoanManagement, ());
        let client = LoanManagementClient::new(&env, &contract_id);

        client.initialize(&admin);

        let new_params = RateParameters {
            base_rate: 300,
            risk_premium: 150,
            slope_parameter: 75,
            max_rate: 6000,
        };

        client.update_rate_parameters(&new_params);

        let stored_params = client.get_rate_parameters();
        assert_eq!(stored_params.base_rate, 300);
        assert_eq!(stored_params.risk_premium, 150);
        assert_eq!(stored_params.slope_parameter, 75);
        assert_eq!(stored_params.max_rate, 6000);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #9)")]
    fn test_update_rate_parameters_invalid() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);

        let contract_id = env.register(LoanManagement, ());
        let client = LoanManagementClient::new(&env, &contract_id);

        client.initialize(&admin);

        // Invalid: base_rate > max_rate
        let invalid_params = RateParameters {
            base_rate: 7000,
            risk_premium: 100,
            slope_parameter: 50,
            max_rate: 5000,
        };

        client.update_rate_parameters(&invalid_params);
    }

    #[test]
    fn test_utilization_tracking() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let borrower = Address::generate(&env);
        let lender = Address::generate(&env);

        let contract_id = env.register(LoanManagement, ());
        let client = LoanManagementClient::new(&env, &contract_id);

        client.initialize(&admin);
        client.update_total_liquidity(&10000);

        // Issue first loan
        let (loan_id_1, _) = client.issue_loan(&1, &borrower, &lender, &2000, &3600);

        let (total_liq, total_bor, util_bps) = client.get_utilization_stats();
        assert_eq!(total_liq, 10000);
        assert_eq!(total_bor, 2000);
        assert_eq!(util_bps, 2000); // 20%

        // Issue second loan
        let (_, _) = client.issue_loan(&2, &borrower, &lender, &3000, &3600);

        let (_, total_bor_2, util_bps_2) = client.get_utilization_stats();
        assert_eq!(total_bor_2, 5000);
        assert_eq!(util_bps_2, 5000); // 50%

        // Repay first loan (no time elapsed, pay full principal)
        client.repay_loan(&loan_id_1, &2000);

        let (_, total_bor_3, util_bps_3) = client.get_utilization_stats();
        assert_eq!(total_bor_3, 3000);
        assert_eq!(util_bps_3, 3000); // 30%
    }

    #[test]
    fn test_rate_increases_with_utilization() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let borrower1 = Address::generate(&env);
        let borrower2 = Address::generate(&env);
        let lender = Address::generate(&env);

        let contract_id = env.register(LoanManagement, ());
        let client = LoanManagementClient::new(&env, &contract_id);

        client.initialize(&admin);
        client.update_total_liquidity(&10000);

        // First loan at low utilization
        let (_, rate1) = client.issue_loan(&1, &borrower1, &lender, &1000, &3600);

        // Second loan at higher utilization
        let (_, rate2) = client.issue_loan(&2, &borrower2, &lender, &3000, &3600);

        // Rate should increase with utilization
        assert!(rate2 > rate1);
    }

    #[test]
    fn test_rate_cap_enforcement() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let borrower = Address::generate(&env);

        let contract_id = env.register(LoanManagement, ());
        let client = LoanManagementClient::new(&env, &contract_id);

        client.initialize(&admin);

        // Set parameters that would exceed max_rate
        let params = RateParameters {
            base_rate: 4000,
            risk_premium: 2000,
            slope_parameter: 1000,
            max_rate: 5000,
        };
        client.update_rate_parameters(&params);

        client.update_total_liquidity(&10000);

        // Calculate rate - should be capped at max_rate
        let rate = client.get_dynamic_rate(&borrower, &5000);
        assert_eq!(rate, 5000); // Capped at max_rate
    }

    #[test]
    fn test_zero_liquidity_utilization() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let borrower = Address::generate(&env);

        let contract_id = env.register(LoanManagement, ());
        let client = LoanManagementClient::new(&env, &contract_id);

        client.initialize(&admin);

        // Don't set any liquidity (defaults to 0)
        let rate = client.get_dynamic_rate(&borrower, &1000);

        // Should still calculate rate with 0 utilization component
        // base_rate (200) + risk_premium * risk_factor (100 * 1) = 300
        assert_eq!(rate, 300);
    }
}

//! Loan Management Contract for StelloVault
//!
//! This contract handles the complete loan lifecycle:
//! - Loan origination with collateral validation
//! - Interest calculation with multiple compounding methods
//! - Repayment processing with partial payment support
//! - Default detection and handling
//! - Loan restructuring and modification
//! - Performance tracking and analytics
//! - Risk-based pricing integration

#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, BytesN, Env, String, Symbol,
    Vec, Map,
};

mod loan;
mod interest;
mod repayment;
mod default_handling;
mod restructuring;
mod analytics;

pub use loan::*;
pub use interest::*;
pub use repayment::*;
pub use default_handling::*;
pub use restructuring::*;
pub use analytics::*;

/// Contract errors
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContractError {
    Unauthorized = 1,
    AlreadyInitialized = 2,
    InvalidLoanData = 3,
    LoanNotFound = 4,
    CollateralNotFound = 5,
    InsufficientCollateral = 6,
    InvalidInterestRate = 7,
    InvalidTerm = 8,
    LoanAlreadyDefaulted = 9,
    InvalidRepaymentAmount = 10,
    RepaymentExceedsBalance = 11,
    LoanNotDue = 12,
    LoanAlreadyPaid = 13,
    InvalidRestructuring = 14,
    RestructuringNotAllowed = 15,
    InsufficientFunds = 16,
    TransferFailed = 17,
    InvalidCurrency = 18,
    RiskAssessmentFailed = 19,
    CollateralValueError = 20,
    InterestCalculationError = 21,
    FeeCalculationError = 22,
    InvalidGracePeriod = 23,
    LoanInDefault = 24,
    InvalidPrepaymentAmount = 25,
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

/// Main loan management contract
#[contract]
pub struct LoanContract;

/// Contract implementation
#[contractimpl]
impl LoanContract {
    /// Initialize the contract with admin and treasury addresses
    ///
    /// # Arguments
    /// * `admin` - Admin address for contract management
    /// * `treasury` - Treasury address for fee collection
    /// * `collateral_registry` - Address of collateral registry contract
    ///
    /// # Events
    /// Emits `ContractInitialized` event
    pub fn initialize(
        env: Env,
        admin: Address,
        treasury: Address,
        collateral_registry: Address,
    ) -> Result<(), ContractError> {
        if env.storage().instance().has(&symbol_short!("admin")) {
            return Err(ContractError::AlreadyInitialized);
        }

        env.storage()
            .instance()
            .set(&symbol_short!("admin"), &admin);
        env.storage()
            .instance()
            .set(&symbol_short!("treasury"), &treasury);
        env.storage()
            .instance()
            .set(&symbol_short!("collateral_reg"), &collateral_registry);
        env.storage()
            .instance()
            .set(&symbol_short!("next_loan_id"), &1u64);

        // Initialize default parameters
        env.storage()
            .instance()
            .set(&symbol_short!("min_ltv"), &5000u32); // 50% minimum LTV
        env.storage()
            .instance()
            .set(&symbol_short!("max_ltv"), &8000u32); // 80% maximum LTV
        env.storage()
            .instance()
            .set(&symbol_short!("min_term"), &86400u64); // 1 day minimum
        env.storage()
            .instance()
            .set(&symbol_short!("max_term"), &31536000u64); // 1 year maximum

        env.events().publish(
            (symbol_short!("loan_init"),),
            (admin.clone(), treasury),
        );

        Ok(())
    }

    /// Create a new loan
    ///
    /// # Arguments
    /// * `borrower` - Address of the borrower
    /// * `collateral_id` - ID of collateral asset
    /// * `principal` - Loan principal amount
    /// * `interest_rate` - Annual interest rate (basis points)
    /// * `term` - Loan term in seconds
    /// * `interest_type` - Type of interest calculation
    ///
    /// # Returns
    /// The loan ID
    ///
    /// # Events
    /// Emits `LoanCreated` event
    pub fn create_loan(
        env: Env,
        borrower: Address,
        collateral_id: u64,
        principal: i128,
        interest_rate: u32,
        term: u64,
        interest_type: InterestType,
    ) -> Result<u64, ContractError> {
        borrower.require_auth();

        // Validate loan parameters
        if principal <= 0 {
            return Err(ContractError::InvalidLoanData);
        }

        if interest_rate > 100000 {
            // Max 1000% annual rate
            return Err(ContractError::InvalidInterestRate);
        }

        let min_term: u64 = env
            .storage()
            .instance()
            .get(&symbol_short!("min_term"))
            .unwrap_or(86400);
        let max_term: u64 = env
            .storage()
            .instance()
            .get(&symbol_short!("max_term"))
            .unwrap_or(31536000);

        if term < min_term || term > max_term {
            return Err(ContractError::InvalidTerm);
        }

        // Validate collateral LTV
        // Note: In production, would call collateral registry to get value
        let collateral_value = principal * 2; // Simplified: assume 2x collateral
        let ltv = (principal * 10000 / collateral_value) as u32;

        let min_ltv: u32 = env
            .storage()
            .instance()
            .get(&symbol_short!("min_ltv"))
            .unwrap_or(5000);
        let max_ltv: u32 = env
            .storage()
            .instance()
            .get(&symbol_short!("max_ltv"))
            .unwrap_or(8000);

        if ltv < min_ltv || ltv > max_ltv {
            return Err(ContractError::InsufficientCollateral);
        }

        // Get next loan ID
        let loan_id: u64 = env
            .storage()
            .instance()
            .get(&symbol_short!("next_loan_id"))
            .unwrap_or(1u64);

        // Create loan record
        let loan = Loan {
            id: loan_id,
            borrower: borrower.clone(),
            principal,
            outstanding_balance: principal,
            interest_rate,
            interest_type,
            collateral_id,
            collateral_value,
            ltv,
            term,
            start_date: env.ledger().timestamp(),
            maturity_date: env.ledger().timestamp() + term,
            status: LoanStatus::Active,
            total_interest_paid: 0,
            total_repaid: 0,
            last_payment_date: env.ledger().timestamp(),
            next_payment_date: env.ledger().timestamp() + 86400 * 30, // 30 days
            grace_period: 86400 * 5, // 5 days grace period
            default_date: 0,
            created_at: env.ledger().timestamp(),
            updated_at: env.ledger().timestamp(),
        };

        // Store loan
        let storage_key = format_loan_storage_key(loan_id);
        env.storage().persistent().set(&storage_key, &loan);

        // Update next loan ID
        env.storage()
            .instance()
            .set(&symbol_short!("next_loan_id"), &(loan_id + 1));

        // Emit event
        env.events().publish(
            (symbol_short!("loan_created"),),
            (loan_id, borrower, principal, interest_rate),
        );

        Ok(loan_id)
    }

    /// Calculate accrued interest for a loan
    ///
    /// # Arguments
    /// * `loan_id` - ID of the loan
    /// * `period` - Period in seconds to calculate interest for
    ///
    /// # Returns
    /// Accrued interest amount
    pub fn calculate_interest(
        env: Env,
        loan_id: u64,
        period: u64,
    ) -> Result<i128, ContractError> {
        // Get loan
        let storage_key = format_loan_storage_key(loan_id);
        let loan: Loan = env
            .storage()
            .persistent()
            .get(&storage_key)
            .ok_or(ContractError::LoanNotFound)?;

        // Calculate interest based on type
        let interest = match loan.interest_type {
            InterestType::Simple => {
                calculate_simple_interest(loan.outstanding_balance, loan.interest_rate, period)?
            }
            InterestType::Compound => {
                calculate_compound_interest(loan.outstanding_balance, loan.interest_rate, period)?
            }
            InterestType::Fixed => {
                calculate_fixed_interest(loan.principal, loan.interest_rate, period)?
            }
        };

        Ok(interest)
    }

    /// Make a loan repayment
    ///
    /// # Arguments
    /// * `loan_id` - ID of the loan
    /// * `amount` - Repayment amount
    /// * `payer` - Address of the payer
    ///
    /// # Events
    /// Emits `RepaymentMade` event
    pub fn make_repayment(
        env: Env,
        loan_id: u64,
        amount: i128,
        payer: Address,
    ) -> Result<(), ContractError> {
        payer.require_auth();

        // Get loan
        let storage_key = format_loan_storage_key(loan_id);
        let mut loan: Loan = env
            .storage()
            .persistent()
            .get(&storage_key)
            .ok_or(ContractError::LoanNotFound)?;

        // Validate loan status
        if loan.status == LoanStatus::Defaulted {
            return Err(ContractError::LoanInDefault);
        }

        if loan.status == LoanStatus::Paid {
            return Err(ContractError::LoanAlreadyPaid);
        }

        // Validate repayment amount
        if amount <= 0 {
            return Err(ContractError::InvalidRepaymentAmount);
        }

        // Calculate current interest
        let time_since_last_payment = env.ledger().timestamp() - loan.last_payment_date;
        let accrued_interest = self.calculate_interest(env.clone(), loan_id, time_since_last_payment)?;

        // Calculate total owed
        let total_owed = loan.outstanding_balance + accrued_interest;

        if amount > total_owed {
            return Err(ContractError::RepaymentExceedsBalance);
        }

        // Apply repayment
        let principal_payment = if amount >= loan.outstanding_balance {
            loan.outstanding_balance
        } else {
            // Allocate to interest first, then principal
            if accrued_interest > 0 {
                let interest_payment = std::cmp::min(amount, accrued_interest);
                amount - interest_payment
            } else {
                amount
            }
        };

        loan.outstanding_balance -= principal_payment;
        loan.total_repaid += amount;
        loan.total_interest_paid += accrued_interest;
        loan.last_payment_date = env.ledger().timestamp();
        loan.updated_at = env.ledger().timestamp();

        // Update status
        if loan.outstanding_balance <= 0 {
            loan.status = LoanStatus::Paid;
        } else {
            loan.status = LoanStatus::Active;
        }

        // Store updated loan
        env.storage().persistent().set(&storage_key, &loan);

        // Record repayment
        let repayment = RepaymentRecord {
            loan_id,
            payer: payer.clone(),
            amount,
            principal_payment,
            interest_payment: accrued_interest,
            timestamp: env.ledger().timestamp(),
        };

        let repayment_key = format_repayment_key(loan_id, env.ledger().timestamp());
        env.storage().persistent().set(&repayment_key, &repayment);

        // Emit event
        env.events().publish(
            (symbol_short!("repayment_made"),),
            (loan_id, amount, loan.outstanding_balance),
        );

        Ok(())
    }

    /// Check and handle loan default
    ///
    /// # Arguments
    /// * `loan_id` - ID of the loan
    ///
    /// # Returns
    /// Whether loan is in default
    ///
    /// # Events
    /// Emits `LoanDefaulted` event if default detected
    pub fn check_default(env: Env, loan_id: u64) -> Result<bool, ContractError> {
        // Get loan
        let storage_key = format_loan_storage_key(loan_id);
        let mut loan: Loan = env
            .storage()
            .persistent()
            .get(&storage_key)
            .ok_or(ContractError::LoanNotFound)?;

        // Check if loan is past due
        let current_time = env.ledger().timestamp();
        let grace_period_end = loan.next_payment_date + loan.grace_period;

        if current_time > grace_period_end && loan.outstanding_balance > 0 {
            // Loan is in default
            loan.status = LoanStatus::Defaulted;
            loan.default_date = current_time;
            loan.updated_at = current_time;

            // Store updated loan
            env.storage().persistent().set(&storage_key, &loan);

            // Emit event
            env.events().publish(
                (symbol_short!("loan_defaulted"),),
                (loan_id, loan.outstanding_balance),
            );

            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Restructure loan terms
    ///
    /// # Arguments
    /// * `loan_id` - ID of the loan
    /// * `new_terms` - New loan terms
    ///
    /// # Events
    /// Emits `LoanRestructured` event
    pub fn restructure_loan(
        env: Env,
        loan_id: u64,
        new_terms: LoanRestructuring,
    ) -> Result<(), ContractError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&symbol_short!("admin"))
            .ok_or(ContractError::Unauthorized)?;

        admin.require_auth();

        // Get loan
        let storage_key = format_loan_storage_key(loan_id);
        let mut loan: Loan = env
            .storage()
            .persistent()
            .get(&storage_key)
            .ok_or(ContractError::LoanNotFound)?;

        // Validate restructuring
        if loan.status == LoanStatus::Paid {
            return Err(ContractError::InvalidRestructuring);
        }

        // Apply new terms
        if let Some(new_rate) = new_terms.new_interest_rate {
            if new_rate > 100000 {
                return Err(ContractError::InvalidInterestRate);
            }
            loan.interest_rate = new_rate;
        }

        if let Some(new_term) = new_terms.new_term {
            let min_term: u64 = env
                .storage()
                .instance()
                .get(&symbol_short!("min_term"))
                .unwrap_or(86400);
            let max_term: u64 = env
                .storage()
                .instance()
                .get(&symbol_short!("max_term"))
                .unwrap_or(31536000);

            if new_term < min_term || new_term > max_term {
                return Err(ContractError::InvalidTerm);
            }
            loan.maturity_date = env.ledger().timestamp() + new_term;
        }

        if let Some(new_grace) = new_terms.new_grace_period {
            loan.grace_period = new_grace;
        }

        loan.updated_at = env.ledger().timestamp();

        // Store updated loan
        env.storage().persistent().set(&storage_key, &loan);

        // Emit event
        env.events().publish(
            (symbol_short!("loan_restructured"),),
            (loan_id, loan.interest_rate),
        );

        Ok(())
    }

    /// Get current loan status
    ///
    /// # Arguments
    /// * `loan_id` - ID of the loan
    ///
    /// # Returns
    /// Current loan details
    pub fn get_loan_status(env: Env, loan_id: u64) -> Result<Loan, ContractError> {
        let storage_key = format_loan_storage_key(loan_id);
        env.storage()
            .persistent()
            .get(&storage_key)
            .ok_or(ContractError::LoanNotFound)
    }

    /// Get loan repayment history
    ///
    /// # Arguments
    /// * `loan_id` - ID of the loan
    ///
    /// # Returns
    /// Vector of repayment records
    pub fn get_repayment_history(
        env: Env,
        loan_id: u64,
    ) -> Result<Vec<RepaymentRecord>, ContractError> {
        // Note: In production, would iterate through repayment records
        Ok(Vec::new(&env))
    }

    /// Calculate early repayment amount
    ///
    /// # Arguments
    /// * `loan_id` - ID of the loan
    /// * `prepayment_penalty_rate` - Penalty rate for early repayment (basis points)
    ///
    /// # Returns
    /// Total amount needed for early repayment
    pub fn calculate_early_repayment(
        env: Env,
        loan_id: u64,
        prepayment_penalty_rate: u32,
    ) -> Result<i128, ContractError> {
        // Get loan
        let storage_key = format_loan_storage_key(loan_id);
        let loan: Loan = env
            .storage()
            .persistent()
            .get(&storage_key)
            .ok_or(ContractError::LoanNotFound)?;

        // Calculate remaining interest
        let time_remaining = if loan.maturity_date > env.ledger().timestamp() {
            loan.maturity_date - env.ledger().timestamp()
        } else {
            0
        };

        let remaining_interest = self.calculate_interest(env, loan_id, time_remaining)?;

        // Calculate penalty
        let penalty = (loan.outstanding_balance * prepayment_penalty_rate as i128) / 10000;

        // Total = outstanding balance + remaining interest + penalty
        Ok(loan.outstanding_balance + remaining_interest + penalty)
    }

    /// Update LTV parameters
    ///
    /// # Arguments
    /// * `min_ltv` - Minimum LTV ratio (basis points)
    /// * `max_ltv` - Maximum LTV ratio (basis points)
    pub fn update_ltv_parameters(
        env: Env,
        min_ltv: u32,
        max_ltv: u32,
    ) -> Result<(), ContractError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&symbol_short!("admin"))
            .ok_or(ContractError::Unauthorized)?;

        admin.require_auth();

        if min_ltv >= max_ltv || max_ltv > 10000 {
            return Err(ContractError::InvalidLoanData);
        }

        env.storage()
            .instance()
            .set(&symbol_short!("min_ltv"), &min_ltv);
        env.storage()
            .instance()
            .set(&symbol_short!("max_ltv"), &max_ltv);

        env.events()
            .publish((symbol_short!("ltv_updated"),), (min_ltv, max_ltv));

        Ok(())
    }

    /// Get loan portfolio analytics
    ///
    /// # Arguments
    /// * `borrower` - Address of borrower (optional for all loans)
    ///
    /// # Returns
    /// Portfolio analytics
    pub fn get_portfolio_analytics(
        env: Env,
        borrower: Option<Address>,
    ) -> Result<PortfolioAnalytics, ContractError> {
        // Note: In production, would aggregate loan data
        Ok(PortfolioAnalytics {
            total_loans: 0,
            total_principal: 0,
            total_outstanding: 0,
            total_interest_paid: 0,
            active_loans: 0,
            defaulted_loans: 0,
            paid_loans: 0,
            average_interest_rate: 0,
            portfolio_ltv: 0,
        })
    }
}

// Helper functions

fn format_loan_storage_key(loan_id: u64) -> String {
    String::from_slice(&Env::default(), &format!("loan_{}", loan_id))
}

fn format_repayment_key(loan_id: u64, timestamp: u64) -> String {
    String::from_slice(
        &Env::default(),
        &format!("repay_{}_{}", loan_id, timestamp),
    )
}

fn calculate_simple_interest(
    principal: i128,
    annual_rate: u32,
    period: u64,
) -> Result<i128, ContractError> {
    // Simple Interest = P * r * t
    // where P = principal, r = annual rate, t = time in years
    let interest = (principal as u128 * annual_rate as u128 * period as u128)
        / (10000 * 365 * 86400) as u128;
    Ok(interest as i128)
}

fn calculate_compound_interest(
    principal: i128,
    annual_rate: u32,
    period: u64,
) -> Result<i128, ContractError> {
    // Compound Interest = P * (1 + r)^t - P
    // Simplified calculation for on-chain efficiency
    let rate_per_second = (annual_rate as u128 * period as u128) / (10000 * 365 * 86400) as u128;
    let interest = (principal as u128 * rate_per_second) / 10000;
    Ok(interest as i128)
}

fn calculate_fixed_interest(
    principal: i128,
    annual_rate: u32,
    period: u64,
) -> Result<i128, ContractError> {
    // Fixed Interest = P * r * t (same as simple, but fixed for loan term)
    let interest = (principal as u128 * annual_rate as u128 * period as u128)
        / (10000 * 365 * 86400) as u128;
    Ok(interest as i128)
}

//! Loan Management Contract for StelloVault
//!
//! This contract manages the lifecycle of loans backed by escrowed collateral.
//! It handles loan issuance, repayment tracking, and default enforcement.

#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env};

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
pub struct Loan {
    pub id: u64,
    pub escrow_id: u64,
    pub borrower: Address,
    pub lender: Address,
    pub amount: i128,
    pub interest_rate: u32, // Basis points (e.g., 500 = 5%)
    pub deadline: u64,
    pub status: LoanStatus,
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
        Ok(())
    }

    /// Issue a new loan backed by an escrow
    ///
    /// # Arguments
    /// * `escrow_id` - The unique identifier of the escrowed collateral
    /// * `borrower` - Address of the borrower
    /// * `lender` - Address of the lender
    /// * `amount` - Loan amount
    /// * `interest_rate` - Interest rate in basis points
    /// * `duration` - Duration in seconds
    pub fn issue_loan(
        env: Env,
        escrow_id: u64,
        borrower: Address,
        lender: Address,
        amount: i128,
        interest_rate: u32,
        duration: u64,
    ) -> Result<u64, ContractError> {
        lender.require_auth();

        // Prevent multiple loans per escrow
        let escrow_key = (symbol_short!("escrow"), escrow_id);
        if env.storage().persistent().has(&escrow_key) {
            return Err(ContractError::LoanAlreadyIssued);
        }

        let loan_id: u64 = env
            .storage()
            .instance()
            .get(&symbol_short!("next_id"))
            .unwrap_or(1);

        let current_ts = env.ledger().timestamp();
        let deadline = current_ts
            .checked_add(duration)
            .ok_or(ContractError::LoanNotActive)?;

        let loan = Loan {
            id: loan_id,
            escrow_id,
            borrower: borrower.clone(),
            lender: lender.clone(),
            amount,
            interest_rate,
            deadline,
            status: LoanStatus::Active,
        };

        // Store loan by ID
        env.storage().persistent().set(&loan_id, &loan);
        // Map escrow to loan ID to prevent duplicates
        env.storage().persistent().set(&escrow_key, &loan_id);

        env.storage()
            .instance()
            .set(&symbol_short!("next_id"), &(loan_id + 1));

        // Emit LoanIssued event
        env.events().publish(
            (symbol_short!("loan_iss"),),
            (loan_id, escrow_id, borrower, lender, amount, deadline),
        );

        Ok(loan_id)
    }

    /// Repay an active loan
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

        let current_ts = env.ledger().timestamp();
        if current_ts > loan.deadline {
            return Err(ContractError::DeadlinePassed);
        }

        // Calculate total repayment: amount + interest
        // For simplicity, we assume interest is fixed and "amount" passed is total
        // In a real scenario, we'd calculate interest: amount * (1 + rate/10000)
        let interest = (loan.amount * (loan.interest_rate as i128)) / 10000;
        let total_due = loan.amount + interest;

        if amount < total_due {
            return Err(ContractError::InsufficientAmount);
        }

        loan.status = LoanStatus::Repaid;
        env.storage().persistent().set(&loan_id, &loan);

        // Emit LoanRepaid event
        env.events()
            .publish((symbol_short!("loan_rep"),), (loan_id, amount));

        Ok(())
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
        env.events().publish(
            (symbol_short!("loan_liq"),),
            (loan_id, liquidator),
        );

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
        env.events().publish(
            (symbol_short!("risk_set"),),
            (risk_engine,),
        );

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
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let borrower = Address::generate(&env);
        let lender = Address::generate(&env);

        let contract_id = env.register(LoanManagement, ());
        let client = LoanManagementClient::new(&env, &contract_id);

        client.initialize(&admin);

        let escrow_id = 1u64;
        let amount = 1000i128;
        let interest_rate = 500u32; // 5%
        let duration = 3600u64; // 1 hour

        let loan_id = client.issue_loan(
            &escrow_id,
            &borrower,
            &lender,
            &amount,
            &interest_rate,
            &duration,
        );
        assert_eq!(loan_id, 1);

        let loan = client.get_loan(&loan_id).unwrap();
        assert_eq!(loan.borrower, borrower);
        assert_eq!(loan.lender, lender);
        assert_eq!(loan.amount, amount);
        assert_eq!(loan.status, LoanStatus::Active);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #4)")]
    fn test_issue_loan_duplicate_escrow() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let borrower = Address::generate(&env);
        let lender = Address::generate(&env);

        let contract_id = env.register(LoanManagement, ());
        let client = LoanManagementClient::new(&env, &contract_id);

        client.initialize(&admin);

        let escrow_id = 1u64;
        client.issue_loan(&escrow_id, &borrower, &lender, &1000, &500, &3600);

        // Should fail
        client.issue_loan(&escrow_id, &borrower, &lender, &1000, &500, &3600);
    }

    #[test]
    fn test_repay_loan_success() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let borrower = Address::generate(&env);
        let lender = Address::generate(&env);

        let contract_id = env.register(LoanManagement, ());
        let client = LoanManagementClient::new(&env, &contract_id);

        client.initialize(&admin);

        let loan_id = client.issue_loan(&1, &borrower, &lender, &1000, &500, &3600);

        // Total due = 1000 + (1000 * 500 / 10000) = 1050
        client.repay_loan(&loan_id, &1050);

        let loan = client.get_loan(&loan_id).unwrap();
        assert_eq!(loan.status, LoanStatus::Repaid);
    }

    #[test]
    fn test_mark_default_success() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let borrower = Address::generate(&env);
        let lender = Address::generate(&env);

        let contract_id = env.register(LoanManagement, ());
        let client = LoanManagementClient::new(&env, &contract_id);

        client.initialize(&admin);

        let duration = 3600u64;
        let loan_id = client.issue_loan(&1, &borrower, &lender, &1000, &500, &duration);

        // Advance ledger time
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
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let borrower = Address::generate(&env);
        let lender = Address::generate(&env);

        let contract_id = env.register(LoanManagement, ());
        let client = LoanManagementClient::new(&env, &contract_id);

        client.initialize(&admin);

        let loan_id = client.issue_loan(&1, &borrower, &lender, &1000, &500, &3600);

        // Try to mark default before deadline
        client.mark_default(&loan_id);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #8)")]
    fn test_repay_loan_insufficient_amount() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let borrower = Address::generate(&env);
        let lender = Address::generate(&env);

        let contract_id = env.register(LoanManagement, ());
        let client = LoanManagementClient::new(&env, &contract_id);

        client.initialize(&admin);

        let loan_id = client.issue_loan(&1, &borrower, &lender, &1000, &500, &3600);

        // Required: 1050, Providing: 1000
        client.repay_loan(&loan_id, &1000);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #7)")]
    fn test_repay_loan_after_deadline() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let borrower = Address::generate(&env);
        let lender = Address::generate(&env);

        let contract_id = env.register(LoanManagement, ());
        let client = LoanManagementClient::new(&env, &contract_id);

        client.initialize(&admin);

        let duration = 3600u64;
        let loan_id = client.issue_loan(&1, &borrower, &lender, &1000, &500, &duration);

        // Advance ledger time past deadline
        env.ledger().with_mut(|li| {
            li.timestamp += duration + 1;
        });

        client.repay_loan(&loan_id, &1050);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #5)")]
    fn test_repay_loan_already_repaid() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let borrower = Address::generate(&env);
        let lender = Address::generate(&env);

        let contract_id = env.register(LoanManagement, ());
        let client = LoanManagementClient::new(&env, &contract_id);

        client.initialize(&admin);

        let loan_id = client.issue_loan(&1, &borrower, &lender, &1000, &500, &3600);
        client.repay_loan(&loan_id, &1050);

        // Try to repay again
        client.repay_loan(&loan_id, &1050);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #5)")]
    fn test_mark_default_already_repaid() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let borrower = Address::generate(&env);
        let lender = Address::generate(&env);

        let contract_id = env.register(LoanManagement, ());
        let client = LoanManagementClient::new(&env, &contract_id);

        client.initialize(&admin);

        let duration = 3600u64;
        let loan_id = client.issue_loan(&1, &borrower, &lender, &1000, &500, &duration);
        client.repay_loan(&loan_id, &1050);

        // Advance ledger time past deadline
        env.ledger().with_mut(|li| {
            li.timestamp += duration + 1;
        });

        // Should fail because status is already Repaid, not Active
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
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let risk_engine = Address::generate(&env);

        let contract_id = env.register(LoanManagement, ());
        let client = LoanManagementClient::new(&env, &contract_id);

        client.initialize(&admin);
        client.set_risk_engine(&risk_engine);

        let stored_engine = client.get_risk_engine();
        assert_eq!(stored_engine, Some(risk_engine));
    }

    #[test]
    fn test_mark_liquidated_success() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let borrower = Address::generate(&env);
        let lender = Address::generate(&env);
        let risk_engine = Address::generate(&env);
        let liquidator = Address::generate(&env);

        let contract_id = env.register(LoanManagement, ());
        let client = LoanManagementClient::new(&env, &contract_id);

        client.initialize(&admin);
        client.set_risk_engine(&risk_engine);

        let loan_id = client.issue_loan(&1, &borrower, &lender, &1000, &500, &3600);

        client.mark_liquidated(&loan_id, &liquidator);

        let loan = client.get_loan(&loan_id).unwrap();
        assert_eq!(loan.status, LoanStatus::Liquidated);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #1)")]
    fn test_mark_liquidated_no_risk_engine() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let borrower = Address::generate(&env);
        let lender = Address::generate(&env);
        let liquidator = Address::generate(&env);

        let contract_id = env.register(LoanManagement, ());
        let client = LoanManagementClient::new(&env, &contract_id);

        client.initialize(&admin);

        let loan_id = client.issue_loan(&1, &borrower, &lender, &1000, &500, &3600);

        // Should fail - no risk engine set
        client.mark_liquidated(&loan_id, &liquidator);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #5)")]
    fn test_mark_liquidated_not_active() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let borrower = Address::generate(&env);
        let lender = Address::generate(&env);
        let risk_engine = Address::generate(&env);
        let liquidator = Address::generate(&env);

        let contract_id = env.register(LoanManagement, ());
        let client = LoanManagementClient::new(&env, &contract_id);

        client.initialize(&admin);
        client.set_risk_engine(&risk_engine);

        let loan_id = client.issue_loan(&1, &borrower, &lender, &1000, &500, &3600);

        // Repay the loan first
        client.repay_loan(&loan_id, &1050);

        // Should fail - loan is already repaid
        client.mark_liquidated(&loan_id, &liquidator);
    }
}

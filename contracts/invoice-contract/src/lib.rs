#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env};

#[contracttype]
#[derive(Clone)]
pub struct Invoice {
    pub id: u64,
    pub issuer: Address,
    pub payer: Address,
    pub amount: i128,
    pub due_date: u64,
    pub is_paid: bool,
}

#[contracttype]
pub enum DataKey {
    Invoice(u64),
    Counter,
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContractError {
    InvoiceNotFound = 1,
    AlreadyPaid = 2,
    Unauthorized = 3,
}

impl From<soroban_sdk::Error> for ContractError {
    fn from(_: soroban_sdk::Error) -> Self {
        ContractError::Unauthorized
    }
}

impl From<&ContractError> for soroban_sdk::Error {
    fn from(e: &ContractError) -> Self {
        soroban_sdk::Error::from_contract_error(*e as u32)
    }
}

#[contract]
pub struct InvoiceContract;

#[contractimpl]
impl InvoiceContract {
    /// Issue a new invoice. Returns the new invoice id.
    pub fn issue_invoice(
        env: Env,
        issuer: Address,
        payer: Address,
        amount: i128,
        due_date: u64,
    ) -> u64 {
        issuer.require_auth();

        let id: u64 = env.storage().instance().get(&DataKey::Counter).unwrap_or(0) + 1;

        let invoice = Invoice {
            id,
            issuer,
            payer,
            amount,
            due_date,
            is_paid: false,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Invoice(id), &invoice);
        env.storage().instance().set(&DataKey::Counter, &id);

        id
    }

    /// Mark an invoice as paid. Only the payer may call this.
    pub fn pay_invoice(env: Env, id: u64) -> Result<(), ContractError> {
        let mut invoice: Invoice = env
            .storage()
            .persistent()
            .get(&DataKey::Invoice(id))
            .ok_or(ContractError::InvoiceNotFound)?;

        if invoice.is_paid {
            return Err(ContractError::AlreadyPaid);
        }

        invoice.payer.require_auth();
        invoice.is_paid = true;

        env.storage()
            .persistent()
            .set(&DataKey::Invoice(id), &invoice);

        Ok(())
    }

    /// Retrieve an invoice by id.
    pub fn get_invoice(env: Env, id: u64) -> Result<Invoice, ContractError> {
        env.storage()
            .persistent()
            .get(&DataKey::Invoice(id))
            .ok_or(ContractError::InvoiceNotFound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::Env;

    #[test]
    fn test_invoice_lifecycle() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(InvoiceContract, ());
        let client = InvoiceContractClient::new(&env, &contract_id);

        let issuer = Address::generate(&env);
        let payer = Address::generate(&env);

        let id = client.issue_invoice(&issuer, &payer, &1000, &9999999);
        assert_eq!(id, 1);

        let invoice = client.get_invoice(&id);
        assert_eq!(invoice.amount, 1000);
        assert!(!invoice.is_paid);

        client.pay_invoice(&id);

        let invoice = client.get_invoice(&id);
        assert!(invoice.is_paid);
    }

    #[test]
    fn test_pay_invoice_twice_fails() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(InvoiceContract, ());
        let client = InvoiceContractClient::new(&env, &contract_id);

        let issuer = Address::generate(&env);
        let payer = Address::generate(&env);

        let id = client.issue_invoice(&issuer, &payer, &500, &9999999);
        client.pay_invoice(&id);

        let result = client.try_pay_invoice(&id);
        assert!(result.is_err());
    }
}

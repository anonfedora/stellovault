use crate::{ContractError, Escrow, EscrowStatus};
use soroban_sdk::{token, Env};

/// Process a refund for an escrow
///
/// This function executes the actual token transfer from the contract
/// back to the buyer's address. It also updates the escrow status
/// and handles any error reporting.
pub fn process_refund(
    env: &Env,
    escrow: &mut Escrow,
    _escrow_id: u64,
) -> Result<(), ContractError> {
    // 1. Validate that the escrow has sufficient balance
    let token_client = token::Client::new(env, &escrow.asset);
    let contract_balance = token_client.balance(&env.current_contract_address());

    if contract_balance < escrow.amount {
        return Err(ContractError::InsufficientBalance);
    }

    // 2. Execute the transfer back to the buyer
    // Note: The original implementation transferred to the lender,
    // but the requirement is to refund the buyer who funded it.
    token_client.transfer(
        &env.current_contract_address(),
        &escrow.buyer,
        &escrow.amount,
    );

    // 3. Update the state
    escrow.status = EscrowStatus::Refunded;

    Ok(())
}

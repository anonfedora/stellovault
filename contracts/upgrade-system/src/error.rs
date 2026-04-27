use soroban_sdk::contracttype;

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContractError {
    Unauthorized = 1,
    AlreadyInitialized = 2,
    ContractNotRegistered = 3,
    ImplementationNotRegistered = 4,
    ProposalNotFound = 5,
    ProposalAlreadyProcessed = 6,
    ProposalNotValidated = 7,
    ProposalNotApproved = 8,
    UpgradeNotReady = 9,
    ValidationFailed = 10,
    ContractPaused = 11,
    NoRollbackAvailable = 12,
    RollbackWindowExpired = 13,
    InvalidVersion = 14,
    AlreadyRegistered = 15,
    ProposalExpired = 16,
}

impl From<ContractError> for soroban_sdk::Error {
    fn from(err: ContractError) -> Self {
        soroban_sdk::Error::from_contract_error(err as u32)
    }
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

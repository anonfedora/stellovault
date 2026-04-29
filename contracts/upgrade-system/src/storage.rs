use soroban_sdk::{contracttype, Address};

#[contracttype]
pub enum DataKey {
    Config,
    ProposalCount,
    Managed(Address),
    ImplMeta(Address),
    Proposal(u64),
    Validation(u64),
    History(Address),
    Notifications(Address),
}

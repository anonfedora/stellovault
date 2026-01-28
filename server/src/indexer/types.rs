use serde::{Deserialize, Serialize};

/// Soroban RPC getEvents response
#[derive(Debug, Deserialize, Serialize)]
pub struct GetEventsResponse {
    pub events: Vec<SorobanEvent>,
    pub latestLedger: u64,
}

/// Raw Soroban Event from RPC
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SorobanEvent {
    pub id: String, // Paging token/cursor
    pub type_: String,
    pub ledger: u64,
    pub ledger_closed_at: String,
    pub contract_id: String,
    pub topic: Vec<String>, // XDR (base64)
    pub value: SorobanEventValue, 
    pub paging_token: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SorobanEventValue {
    pub xdr: String,
}

/// Domain Events
#[derive(Debug, Clone)]
pub enum ContractEvent {
    Collateral(CollateralEvent),
    Escrow(EscrowEvent),
    Loan(LoanEvent),
}

#[derive(Debug, Clone)]
pub enum CollateralEvent {
    Registered {
        id: u64,
        owner: String,
        face_value: i128,
        expiry_ts: u64,
    },
    Locked {
        id: u64,
    },
    Unlocked {
        id: u64,
    },
}

#[derive(Debug, Clone)]
pub enum EscrowEvent {
    Created {
        id: u64, // Escrow ID
        buyer: String,
        seller: String,
        amount: i128,
    },
    Activated {
        id: u64,
    },
    Released {
        id: u64,
    },
    Cancelled {
        id: u64,
    },
}

#[derive(Debug, Clone)]
pub enum LoanEvent {
    Issued {
        id: u64,
        escrow_id: u64,
        amount: i128,
    },
    Repaid {
        id: u64,
        amount: i128,
    },
    Defaulted {
        id: u64,
    },
}

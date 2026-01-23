use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OraclePayload {
    pub public_key: String,
    pub timestamp: u64,
    pub data_type: String, // e.g., "shipping", "iot", "manual" - I use this to classify the data
    pub value: String,     // JSON string or specific format - I store the actual data here
    pub signature: String, // Hex-encoded signature - I verify this for authenticity
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OracleConfirmation {
    pub initial_tx_hash: String,
    pub status: String,
    pub block: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OracleEvent {
    pub event_type: String,
    pub payload: OraclePayload,
    pub processed_at: i64,
}

#[derive(Debug, Clone)]
pub struct AggregationState {
    pub required_signatures: usize,
    pub received_signatures: Vec<String>, // I track the list of sources that signed
}

impl Default for AggregationState {
    fn default() -> Self {
        Self {
            required_signatures: 2, // I require 2 out of N for MVP
            received_signatures: Vec::new(),
        }
    }
}

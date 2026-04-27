//! Common test utilities and fixtures for all contract tests

use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env, Symbol,
};

/// Test environment setup with default ledger state
pub fn setup_env() -> Env {
    let env = Env::default();
    env.ledger().set_timestamp(1_000_000);
    env
}

/// Create a test address
pub fn test_address(id: u32) -> Address {
    Address::generate(&Env::default())
}

/// Helper to advance ledger time
pub fn advance_ledger(env: &Env, seconds: u64) {
    let current = env.ledger().timestamp();
    env.ledger().set_timestamp(current + seconds);
}

/// Helper to set ledger sequence
pub fn set_ledger_sequence(env: &Env, sequence: u32) {
    env.ledger().set_sequence_number(sequence);
}

/// Test constants
pub mod constants {
    pub const TEST_ADMIN: &str = "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAY5V3";
    pub const TEST_USER: &str = "GBAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAY5V3";
    pub const TEST_ORACLE: &str = "GCAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAY5V3";
    
    pub const ONE_YEAR_SECONDS: u64 = 365 * 24 * 60 * 60;
    pub const ONE_DAY_SECONDS: u64 = 24 * 60 * 60;
    pub const ONE_HOUR_SECONDS: u64 = 60 * 60;
    
    pub const MIN_COLLATERAL_VALUE: i128 = 1_000_000; // 0.1 units (7 decimals)
    pub const MAX_COLLATERAL_VALUE: i128 = 1_000_000_000_000_000; // 100M units
    pub const MIN_LOAN_AMOUNT: i128 = 100_000; // 0.01 units
    pub const MAX_LOAN_AMOUNT: i128 = 500_000_000_000_000; // 50M units
}

/// Mock oracle data for testing
#[derive(Clone, Debug)]
pub struct MockOracleData {
    pub price: i128,
    pub timestamp: u64,
    pub confidence: u32,
}

impl Default for MockOracleData {
    fn default() -> Self {
        Self {
            price: 1_000_000, // 1.0 with 6 decimals
            timestamp: 1_000_000,
            confidence: 100, // 100% confidence
        }
    }
}

/// Test result type
pub type TestResult<T> = Result<T, Box<dyn std::error::Error>>;

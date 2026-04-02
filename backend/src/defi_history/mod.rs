//! DeFi History Provider Module
//!
//! Fetches on-chain activity from DeFi protocols (Soroswap, Aquarius)
//! to calculate on-chain activity scores for risk assessment.

pub mod aquarius_client;
pub mod model;
pub mod provider;
pub mod soroswap_client;

pub use model::*;
pub use provider::DeFiHistoryProvider;

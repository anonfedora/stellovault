//! Data models for DeFi activity tracking

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Aggregated DeFi activity metrics for a wallet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeFiActivityMetrics {
    pub wallet_address: String,
    pub soroswap_metrics: SoroswapMetrics,
    pub aquarius_metrics: AquariusMetrics,
    pub aggregated: AggregatedMetrics,
    pub fetched_at: DateTime<Utc>,
}

/// Soroswap-specific metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SoroswapMetrics {
    pub tx_count: i32,
    pub total_volume_usd: i64,
    pub liquidity_provided: i64,
    pub unique_pairs_traded: i32,
    pub last_activity_at: Option<DateTime<Utc>>,
    pub average_trade_size_usd: i64,
}

/// Aquarius-specific metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AquariusMetrics {
    pub tx_count: i32,
    pub total_volume_usd: i64,
    pub positions_count: i32,
    pub last_activity_at: Option<DateTime<Utc>>,
    pub average_position_size_usd: i64,
}

/// Aggregated metrics across all protocols
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedMetrics {
    pub total_tx_count: i32,
    pub total_volume_usd: i64,
    pub protocol_diversity_score: i32,      // 0-100
    pub activity_consistency_score: i32,    // 0-100
    pub metrics_start_date: DateTime<Utc>,
    pub metrics_end_date: DateTime<Utc>,
}

/// Individual transaction from Soroswap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoroswapTransaction {
    pub tx_hash: String,
    pub timestamp: DateTime<Utc>,
    pub tx_type: SoroswapTxType,
    pub token_in: String,
    pub token_out: String,
    pub amount_in: i64,
    pub amount_out: i64,
    pub usd_value: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SoroswapTxType {
    Swap,
    AddLiquidity,
    RemoveLiquidity,
}

/// Individual transaction from Aquarius
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AquariusTransaction {
    pub tx_hash: String,
    pub timestamp: DateTime<Utc>,
    pub tx_type: AquariusTxType,
    pub asset_pair: String,
    pub amount: i64,
    pub usd_value: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AquariusTxType {
    Trade,
    OpenPosition,
    ClosePosition,
    AddCollateral,
}

/// Query parameters for fetching DeFi history
#[derive(Debug, Clone)]
pub struct DeFiHistoryQuery {
    pub wallet_address: String,
    pub days_back: i32,
    pub include_soroswap: bool,
    pub include_aquarius: bool,
}

impl Default for DeFiHistoryQuery {
    fn default() -> Self {
        Self {
            wallet_address: String::new(),
            days_back: 90,
            include_soroswap: true,
            include_aquarius: true,
        }
    }
}

/// Error types for DeFi history fetching
#[derive(Debug, thiserror::Error)]
pub enum DeFiHistoryError {
    #[error("API request failed: {0}")]
    ApiError(String),
    
    #[error("Rate limit exceeded for {0}")]
    RateLimitExceeded(String),
    
    #[error("Invalid wallet address: {0}")]
    InvalidWalletAddress(String),
    
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
    
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("Cache error: {0}")]
    CacheError(String),
}

pub type DeFiHistoryResult<T> = Result<T, DeFiHistoryError>;

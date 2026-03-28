//! Soroswap API client for fetching DEX activity

use chrono::{DateTime, Duration, Utc};
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration as StdDuration;

use super::model::*;

/// Client for interacting with Soroswap API
pub struct SoroswapClient {
    api_url: String,
    api_key: Option<String>,
    client: Client,
}

impl SoroswapClient {
    /// Create a new Soroswap client
    pub fn new(api_url: String, api_key: Option<String>) -> Self {
        let client = Client::builder()
            .timeout(StdDuration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            api_url,
            api_key,
            client,
        }
    }

    /// Fetch user activity from Soroswap
    pub async fn fetch_user_activity(
        &self,
        wallet_address: &str,
        days_back: i32,
    ) -> DeFiHistoryResult<SoroswapMetrics> {
        tracing::debug!(
            wallet = %wallet_address,
            days_back = days_back,
            "Fetching Soroswap activity"
        );

        // Calculate date range
        let end_date = Utc::now();
        let start_date = end_date - Duration::days(days_back as i64);

        // Fetch transactions from Soroswap API
        let transactions = self
            .fetch_transactions(wallet_address, start_date, end_date)
            .await?;

        // Calculate metrics from transactions
        let metrics = self.calculate_metrics(transactions);

        tracing::info!(
            wallet = %wallet_address,
            tx_count = metrics.tx_count,
            volume_usd = metrics.total_volume_usd,
            "Fetched Soroswap metrics"
        );

        Ok(metrics)
    }

    /// Fetch transactions from Soroswap API
    async fn fetch_transactions(
        &self,
        wallet_address: &str,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> DeFiHistoryResult<Vec<SoroswapTransaction>> {
        // Build API request
        let url = format!("{}/user/{}/transactions", self.api_url, wallet_address);

        let mut request = self.client.get(&url).query(&[
            ("start_date", start_date.to_rfc3339()),
            ("end_date", end_date.to_rfc3339()),
        ]);

        // Add API key if provided
        if let Some(ref api_key) = self.api_key {
            request = request.header("X-API-Key", api_key);
        }

        // Execute request
        let response = request.send().await.map_err(|e| {
            tracing::error!("Soroswap API request failed: {}", e);
            DeFiHistoryError::NetworkError(e)
        })?;

        // Check for rate limiting
        if response.status() == 429 {
            return Err(DeFiHistoryError::RateLimitExceeded("Soroswap".to_string()));
        }

        // Check for errors
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(DeFiHistoryError::ApiError(format!(
                "Soroswap API error {}: {}",
                status, error_text
            )));
        }

        // Parse response
        let api_response: SoroswapApiResponse = response.json().await.map_err(|e| {
            tracing::error!("Failed to parse Soroswap response: {}", e);
            DeFiHistoryError::ParseError(e.to_string())
        })?;

        Ok(api_response.transactions)
    }

    /// Calculate metrics from transactions
    fn calculate_metrics(&self, transactions: Vec<SoroswapTransaction>) -> SoroswapMetrics {
        if transactions.is_empty() {
            return SoroswapMetrics::default();
        }

        let tx_count = transactions.len() as i32;
        let total_volume_usd: i64 = transactions.iter().map(|tx| tx.usd_value).sum();

        // Calculate liquidity provided (sum of add_liquidity transactions)
        let liquidity_provided: i64 = transactions
            .iter()
            .filter(|tx| matches!(tx.tx_type, SoroswapTxType::AddLiquidity))
            .map(|tx| tx.usd_value)
            .sum();

        // Count unique trading pairs
        let unique_pairs: std::collections::HashSet<String> = transactions
            .iter()
            .map(|tx| format!("{}/{}", tx.token_in, tx.token_out))
            .collect();
        let unique_pairs_traded = unique_pairs.len() as i32;

        // Find last activity
        let last_activity_at = transactions
            .iter()
            .map(|tx| tx.timestamp)
            .max();

        // Calculate average trade size
        let average_trade_size_usd = if tx_count > 0 {
            total_volume_usd / tx_count as i64
        } else {
            0
        };

        SoroswapMetrics {
            tx_count,
            total_volume_usd,
            liquidity_provided,
            unique_pairs_traded,
            last_activity_at,
            average_trade_size_usd,
        }
    }
}

/// Soroswap API response structure
#[derive(Debug, Deserialize)]
struct SoroswapApiResponse {
    transactions: Vec<SoroswapTransaction>,
}

/// Mock implementation for testing when Soroswap API is not available
impl SoroswapClient {
    /// Generate mock data for testing
    #[allow(dead_code)]
    pub async fn fetch_mock_activity(
        &self,
        wallet_address: &str,
        days_back: i32,
    ) -> DeFiHistoryResult<SoroswapMetrics> {
        tracing::warn!(
            wallet = %wallet_address,
            "Using mock Soroswap data - API not available"
        );

        // Generate realistic mock data based on wallet address hash
        let hash = wallet_address.chars().map(|c| c as u32).sum::<u32>();
        let tx_count = ((hash % 50) + 5) as i32;
        let volume_multiplier = ((hash % 10) + 1) as i64;

        Ok(SoroswapMetrics {
            tx_count,
            total_volume_usd: tx_count as i64 * 1000 * volume_multiplier,
            liquidity_provided: tx_count as i64 * 500 * volume_multiplier,
            unique_pairs_traded: (tx_count / 3).max(1),
            last_activity_at: Some(Utc::now() - Duration::days((hash % days_back as u32) as i64)),
            average_trade_size_usd: 1000 * volume_multiplier,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_metrics_empty() {
        let client = SoroswapClient::new("http://test".to_string(), None);
        let metrics = client.calculate_metrics(vec![]);
        assert_eq!(metrics.tx_count, 0);
        assert_eq!(metrics.total_volume_usd, 0);
    }

    #[test]
    fn test_calculate_metrics_with_transactions() {
        let client = SoroswapClient::new("http://test".to_string(), None);
        let transactions = vec![
            SoroswapTransaction {
                tx_hash: "hash1".to_string(),
                timestamp: Utc::now(),
                tx_type: SoroswapTxType::Swap,
                token_in: "USDC".to_string(),
                token_out: "XLM".to_string(),
                amount_in: 1000,
                amount_out: 5000,
                usd_value: 1000,
            },
            SoroswapTransaction {
                tx_hash: "hash2".to_string(),
                timestamp: Utc::now(),
                tx_type: SoroswapTxType::AddLiquidity,
                token_in: "USDC".to_string(),
                token_out: "XLM".to_string(),
                amount_in: 2000,
                amount_out: 10000,
                usd_value: 2000,
            },
        ];

        let metrics = client.calculate_metrics(transactions);
        assert_eq!(metrics.tx_count, 2);
        assert_eq!(metrics.total_volume_usd, 3000);
        assert_eq!(metrics.liquidity_provided, 2000);
        assert_eq!(metrics.average_trade_size_usd, 1500);
    }
}

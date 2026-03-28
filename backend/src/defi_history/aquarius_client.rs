//! Aquarius API client for fetching trading activity

use chrono::{DateTime, Duration, Utc};
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration as StdDuration;

use super::model::*;

/// Client for interacting with Aquarius API
pub struct AquariusClient {
    api_url: String,
    api_key: Option<String>,
    client: Client,
}

impl AquariusClient {
    /// Create a new Aquarius client
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

    /// Fetch user activity from Aquarius
    pub async fn fetch_user_activity(
        &self,
        wallet_address: &str,
        days_back: i32,
    ) -> DeFiHistoryResult<AquariusMetrics> {
        tracing::debug!(
            wallet = %wallet_address,
            days_back = days_back,
            "Fetching Aquarius activity"
        );

        // Calculate date range
        let end_date = Utc::now();
        let start_date = end_date - Duration::days(days_back as i64);

        // Fetch transactions from Aquarius API
        let transactions = self
            .fetch_transactions(wallet_address, start_date, end_date)
            .await?;

        // Calculate metrics from transactions
        let metrics = self.calculate_metrics(transactions);

        tracing::info!(
            wallet = %wallet_address,
            tx_count = metrics.tx_count,
            volume_usd = metrics.total_volume_usd,
            "Fetched Aquarius metrics"
        );

        Ok(metrics)
    }

    /// Fetch transactions from Aquarius API
    async fn fetch_transactions(
        &self,
        wallet_address: &str,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> DeFiHistoryResult<Vec<AquariusTransaction>> {
        // Build API request
        let url = format!("{}/accounts/{}/transactions", self.api_url, wallet_address);

        let mut request = self.client.get(&url).query(&[
            ("start_time", start_date.timestamp().to_string()),
            ("end_time", end_date.timestamp().to_string()),
            ("limit", "1000".to_string()),
        ]);

        // Add API key if provided
        if let Some(ref api_key) = self.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        // Execute request
        let response = request.send().await.map_err(|e| {
            tracing::error!("Aquarius API request failed: {}", e);
            DeFiHistoryError::NetworkError(e)
        })?;

        // Check for rate limiting
        if response.status() == 429 {
            return Err(DeFiHistoryError::RateLimitExceeded("Aquarius".to_string()));
        }

        // Check for errors
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(DeFiHistoryError::ApiError(format!(
                "Aquarius API error {}: {}",
                status, error_text
            )));
        }

        // Parse response
        let api_response: AquariusApiResponse = response.json().await.map_err(|e| {
            tracing::error!("Failed to parse Aquarius response: {}", e);
            DeFiHistoryError::ParseError(e.to_string())
        })?;

        Ok(api_response.transactions)
    }

    /// Calculate metrics from transactions
    fn calculate_metrics(&self, transactions: Vec<AquariusTransaction>) -> AquariusMetrics {
        if transactions.is_empty() {
            return AquariusMetrics::default();
        }

        let tx_count = transactions.len() as i32;
        let total_volume_usd: i64 = transactions.iter().map(|tx| tx.usd_value).sum();

        // Count open positions
        let positions_count = transactions
            .iter()
            .filter(|tx| matches!(tx.tx_type, AquariusTxType::OpenPosition))
            .count() as i32;

        // Find last activity
        let last_activity_at = transactions
            .iter()
            .map(|tx| tx.timestamp)
            .max();

        // Calculate average position size
        let average_position_size_usd = if positions_count > 0 {
            let position_volume: i64 = transactions
                .iter()
                .filter(|tx| matches!(tx.tx_type, AquariusTxType::OpenPosition))
                .map(|tx| tx.usd_value)
                .sum();
            position_volume / positions_count as i64
        } else {
            0
        };

        AquariusMetrics {
            tx_count,
            total_volume_usd,
            positions_count,
            last_activity_at,
            average_position_size_usd,
        }
    }
}

/// Aquarius API response structure
#[derive(Debug, Deserialize)]
struct AquariusApiResponse {
    transactions: Vec<AquariusTransaction>,
}

/// Mock implementation for testing when Aquarius API is not available
impl AquariusClient {
    /// Generate mock data for testing
    #[allow(dead_code)]
    pub async fn fetch_mock_activity(
        &self,
        wallet_address: &str,
        days_back: i32,
    ) -> DeFiHistoryResult<AquariusMetrics> {
        tracing::warn!(
            wallet = %wallet_address,
            "Using mock Aquarius data - API not available"
        );

        // Generate realistic mock data based on wallet address hash
        let hash = wallet_address.chars().map(|c| c as u32).sum::<u32>();
        let tx_count = ((hash % 40) + 3) as i32;
        let volume_multiplier = ((hash % 8) + 1) as i64;
        let positions = (tx_count / 4).max(1);

        Ok(AquariusMetrics {
            tx_count,
            total_volume_usd: tx_count as i64 * 800 * volume_multiplier,
            positions_count: positions,
            last_activity_at: Some(Utc::now() - Duration::days((hash % days_back as u32) as i64)),
            average_position_size_usd: 3200 * volume_multiplier,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_metrics_empty() {
        let client = AquariusClient::new("http://test".to_string(), None);
        let metrics = client.calculate_metrics(vec![]);
        assert_eq!(metrics.tx_count, 0);
        assert_eq!(metrics.total_volume_usd, 0);
    }

    #[test]
    fn test_calculate_metrics_with_transactions() {
        let client = AquariusClient::new("http://test".to_string(), None);
        let transactions = vec![
            AquariusTransaction {
                tx_hash: "hash1".to_string(),
                timestamp: Utc::now(),
                tx_type: AquariusTxType::Trade,
                asset_pair: "XLM/USDC".to_string(),
                amount: 5000,
                usd_value: 1000,
            },
            AquariusTransaction {
                tx_hash: "hash2".to_string(),
                timestamp: Utc::now(),
                tx_type: AquariusTxType::OpenPosition,
                asset_pair: "XLM/USDC".to_string(),
                amount: 10000,
                usd_value: 2000,
            },
        ];

        let metrics = client.calculate_metrics(transactions);
        assert_eq!(metrics.tx_count, 2);
        assert_eq!(metrics.total_volume_usd, 3000);
        assert_eq!(metrics.positions_count, 1);
        assert_eq!(metrics.average_position_size_usd, 2000);
    }
}

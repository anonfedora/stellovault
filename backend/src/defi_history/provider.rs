//! DeFi History Provider - Main interface for fetching DeFi activity

use chrono::{Duration, Utc};
use std::sync::Arc;

use super::aquarius_client::AquariusClient;
use super::model::*;
use super::soroswap_client::SoroswapClient;

/// Main provider for fetching DeFi activity across multiple protocols
pub struct DeFiHistoryProvider {
    soroswap_client: Arc<SoroswapClient>,
    aquarius_client: Arc<AquariusClient>,
    enabled: bool,
}

impl DeFiHistoryProvider {
    /// Create a new DeFi history provider
    pub fn new(
        soroswap_api_url: String,
        soroswap_api_key: Option<String>,
        aquarius_api_url: String,
        aquarius_api_key: Option<String>,
        enabled: bool,
    ) -> Self {
        Self {
            soroswap_client: Arc::new(SoroswapClient::new(soroswap_api_url, soroswap_api_key)),
            aquarius_client: Arc::new(AquariusClient::new(aquarius_api_url, aquarius_api_key)),
            enabled,
        }
    }

    /// Fetch aggregated DeFi activity metrics for a wallet
    pub async fn fetch_activity_metrics(
        &self,
        query: DeFiHistoryQuery,
    ) -> DeFiHistoryResult<DeFiActivityMetrics> {
        if !self.enabled {
            tracing::warn!("DeFi history provider is disabled, returning empty metrics");
            return Ok(self.empty_metrics(&query.wallet_address));
        }

        // Validate wallet address
        if !self.is_valid_stellar_address(&query.wallet_address) {
            return Err(DeFiHistoryError::InvalidWalletAddress(
                query.wallet_address.clone(),
            ));
        }

        let start_time = std::time::Instant::now();

        // Fetch from both protocols concurrently
        let (soroswap_result, aquarius_result) = tokio::join!(
            self.fetch_soroswap_metrics(&query),
            self.fetch_aquarius_metrics(&query)
        );

        // Handle errors gracefully - if one fails, continue with the other
        let soroswap_metrics = match soroswap_result {
            Ok(metrics) => metrics,
            Err(e) => {
                tracing::warn!("Failed to fetch Soroswap metrics: {}", e);
                SoroswapMetrics::default()
            }
        };

        let aquarius_metrics = match aquarius_result {
            Ok(metrics) => metrics,
            Err(e) => {
                tracing::warn!("Failed to fetch Aquarius metrics: {}", e);
                AquariusMetrics::default()
            }
        };

        // Calculate aggregated metrics
        let aggregated = self.calculate_aggregated_metrics(
            &soroswap_metrics,
            &aquarius_metrics,
            query.days_back,
        );

        let elapsed = start_time.elapsed();
        tracing::info!(
            wallet = %query.wallet_address,
            total_tx = aggregated.total_tx_count,
            total_volume = aggregated.total_volume_usd,
            elapsed_ms = elapsed.as_millis(),
            "Fetched DeFi activity metrics"
        );

        Ok(DeFiActivityMetrics {
            wallet_address: query.wallet_address,
            soroswap_metrics,
            aquarius_metrics,
            aggregated,
            fetched_at: Utc::now(),
        })
    }

    /// Fetch Soroswap metrics
    async fn fetch_soroswap_metrics(
        &self,
        query: &DeFiHistoryQuery,
    ) -> DeFiHistoryResult<SoroswapMetrics> {
        if !query.include_soroswap {
            return Ok(SoroswapMetrics::default());
        }

        self.soroswap_client
            .fetch_user_activity(&query.wallet_address, query.days_back)
            .await
    }

    /// Fetch Aquarius metrics
    async fn fetch_aquarius_metrics(
        &self,
        query: &DeFiHistoryQuery,
    ) -> DeFiHistoryResult<AquariusMetrics> {
        if !query.include_aquarius {
            return Ok(AquariusMetrics::default());
        }

        self.aquarius_client
            .fetch_user_activity(&query.wallet_address, query.days_back)
            .await
    }

    /// Calculate aggregated metrics from individual protocol metrics
    fn calculate_aggregated_metrics(
        &self,
        soroswap: &SoroswapMetrics,
        aquarius: &AquariusMetrics,
        days_back: i32,
    ) -> AggregatedMetrics {
        let total_tx_count = soroswap.tx_count + aquarius.tx_count;
        let total_volume_usd = soroswap.total_volume_usd + aquarius.total_volume_usd;

        // Protocol diversity: 0-100 score based on number of protocols used
        let protocols_used = [
            soroswap.tx_count > 0,
            aquarius.tx_count > 0,
        ]
        .iter()
        .filter(|&&used| used)
        .count();

        let protocol_diversity_score = match protocols_used {
            0 => 0,
            1 => 50,
            2 => 100,
            _ => 100,
        };

        // Activity consistency: 0-100 score based on regular activity
        let activity_consistency_score = self.calculate_consistency_score(
            total_tx_count,
            days_back,
            soroswap.last_activity_at,
            aquarius.last_activity_at,
        );

        let end_date = Utc::now();
        let start_date = end_date - Duration::days(days_back as i64);

        AggregatedMetrics {
            total_tx_count,
            total_volume_usd,
            protocol_diversity_score,
            activity_consistency_score,
            metrics_start_date: start_date,
            metrics_end_date: end_date,
        }
    }

    /// Calculate activity consistency score
    fn calculate_consistency_score(
        &self,
        total_tx: i32,
        days_back: i32,
        soroswap_last: Option<chrono::DateTime<Utc>>,
        aquarius_last: Option<chrono::DateTime<Utc>>,
    ) -> i32 {
        if total_tx == 0 {
            return 0;
        }

        // Find most recent activity
        let last_activity = [soroswap_last, aquarius_last]
            .iter()
            .filter_map(|&dt| dt)
            .max();

        let days_since_last = match last_activity {
            Some(dt) => (Utc::now() - dt).num_days(),
            None => days_back as i64,
        };

        // Calculate average transactions per week
        let weeks = (days_back as f64 / 7.0).max(1.0);
        let tx_per_week = total_tx as f64 / weeks;

        // Score based on recency and frequency
        let recency_score = if days_since_last <= 7 {
            50
        } else if days_since_last <= 30 {
            30
        } else if days_since_last <= 60 {
            15
        } else {
            5
        };

        let frequency_score = if tx_per_week >= 5.0 {
            50
        } else if tx_per_week >= 2.0 {
            35
        } else if tx_per_week >= 1.0 {
            20
        } else if tx_per_week >= 0.5 {
            10
        } else {
            5
        };

        (recency_score + frequency_score).min(100)
    }

    /// Validate Stellar address format
    fn is_valid_stellar_address(&self, address: &str) -> bool {
        // Stellar public keys start with 'G' and are 56 characters
        address.starts_with('G') && address.len() == 56
    }

    /// Return empty metrics when provider is disabled
    fn empty_metrics(&self, wallet_address: &str) -> DeFiActivityMetrics {
        let now = Utc::now();
        DeFiActivityMetrics {
            wallet_address: wallet_address.to_string(),
            soroswap_metrics: SoroswapMetrics::default(),
            aquarius_metrics: AquariusMetrics::default(),
            aggregated: AggregatedMetrics {
                total_tx_count: 0,
                total_volume_usd: 0,
                protocol_diversity_score: 0,
                activity_consistency_score: 0,
                metrics_start_date: now - Duration::days(90),
                metrics_end_date: now,
            },
            fetched_at: now,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_stellar_address() {
        let provider = DeFiHistoryProvider::new(
            "http://test".to_string(),
            None,
            "http://test".to_string(),
            None,
            true,
        );

        assert!(provider.is_valid_stellar_address("GABC1234567890123456789012345678901234567890123456789012"));
        assert!(!provider.is_valid_stellar_address("MABC1234567890123456789012345678901234567890123456789012")); // Wrong prefix
        assert!(!provider.is_valid_stellar_address("GABC12345")); // Too short
    }

    #[test]
    fn test_protocol_diversity_score() {
        let provider = DeFiHistoryProvider::new(
            "http://test".to_string(),
            None,
            "http://test".to_string(),
            None,
            true,
        );

        let soroswap = SoroswapMetrics {
            tx_count: 10,
            ..Default::default()
        };
        let aquarius = AquariusMetrics {
            tx_count: 5,
            ..Default::default()
        };

        let aggregated = provider.calculate_aggregated_metrics(&soroswap, &aquarius, 90);
        assert_eq!(aggregated.protocol_diversity_score, 100); // Both protocols used
    }
}

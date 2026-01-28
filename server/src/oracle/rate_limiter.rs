//! Per-oracle rate limiting for abuse prevention
//!
//! I'm implementing stricter rate limits for oracles than the general API to prevent spam.

use std::{collections::HashMap, sync::Arc, time::Instant};
use tokio::sync::RwLock;

/// Token bucket for per-oracle rate limiting
#[derive(Debug, Clone)]
struct TokenBucket {
    tokens: f64,
    last_update: Instant,
}

impl TokenBucket {
    fn new(max_tokens: f64) -> Self {
        Self {
            tokens: max_tokens,
            last_update: Instant::now(),
        }
    }

    fn try_consume(&mut self, tokens_per_second: f64, max_tokens: f64) -> bool {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update).as_secs_f64();

        // I'm refilling tokens based on elapsed time since last request.
        self.tokens = (self.tokens + elapsed * tokens_per_second).min(max_tokens);
        self.last_update = now;

        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

/// Per-oracle rate limiter with stricter limits than general API
#[derive(Clone)]
pub struct OracleRateLimiter {
    buckets: Arc<RwLock<HashMap<String, TokenBucket>>>,
    tokens_per_minute: f64,
    max_tokens: f64,
}

impl OracleRateLimiter {
    /// Create a new oracle rate limiter with specified requests per minute limit
    pub fn new(requests_per_minute: u32) -> Self {
        Self {
            buckets: Arc::new(RwLock::new(HashMap::new())),
            tokens_per_minute: requests_per_minute as f64,
            // I'm allowing a small burst of 2x the per-minute rate.
            max_tokens: (requests_per_minute * 2) as f64,
        }
    }

    /// Check if an oracle address is allowed to make a request
    pub async fn check(&self, oracle_address: &str) -> bool {
        let mut buckets = self.buckets.write().await;

        let bucket = buckets
            .entry(oracle_address.to_string())
            .or_insert_with(|| TokenBucket::new(self.max_tokens));

        // I'm converting per-minute rate to per-second for the token bucket algorithm.
        let tokens_per_second = self.tokens_per_minute / 60.0;
        bucket.try_consume(tokens_per_second, self.max_tokens)
    }

    /// Get remaining tokens for an oracle address (for rate limit headers)
    #[allow(dead_code)] // I'm keeping this for future rate limit header support.
    pub async fn remaining(&self, oracle_address: &str) -> u32 {
        let buckets = self.buckets.read().await;
        buckets
            .get(oracle_address)
            .map(|b| b.tokens as u32)
            .unwrap_or(self.max_tokens as u32)
    }

    /// Cleanup old entries to prevent memory bloat
    #[allow(dead_code)] // I'm keeping this for future scheduled cleanup.
    pub async fn cleanup(&self, max_age: std::time::Duration) {
        let mut buckets = self.buckets.write().await;
        let now = Instant::now();
        buckets.retain(|_, bucket| now.duration_since(bucket.last_update) < max_age);
    }
}

impl Default for OracleRateLimiter {
    fn default() -> Self {
        // I'm defaulting to 10 requests per minute per oracle.
        Self::new(10)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_oracle_rate_limiter() {
        let limiter = OracleRateLimiter::new(5); // 5 requests per minute

        // Should allow first 10 requests (burst capacity = 2x)
        for _ in 0..10 {
            assert!(limiter.check("oracle-1").await);
        }

        // Next request should be denied (bucket empty)
        assert!(!limiter.check("oracle-1").await);
    }

    #[tokio::test]
    async fn test_oracle_rate_limiter_different_oracles() {
        let limiter = OracleRateLimiter::new(2);

        // Different oracles have separate buckets
        assert!(limiter.check("oracle-a").await);
        assert!(limiter.check("oracle-b").await);
        assert!(limiter.check("oracle-a").await);
        assert!(limiter.check("oracle-b").await);
    }
}

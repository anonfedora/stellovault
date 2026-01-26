//! Rate limiting middleware

use axum::{
    body::Body,
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::{collections::HashMap, sync::Arc, time::Instant};
use tokio::sync::RwLock;

/// Token bucket for rate limiting
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

        // Refill tokens
        self.tokens = (self.tokens + elapsed * tokens_per_second).min(max_tokens);
        self.last_update = now;

        // Try to consume a token
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

/// Rate limiter state
#[derive(Clone)]
pub struct RateLimiter {
    buckets: Arc<RwLock<HashMap<String, TokenBucket>>>,
    tokens_per_second: f64,
    max_tokens: f64,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(requests_per_second: u32) -> Self {
        Self {
            buckets: Arc::new(RwLock::new(HashMap::new())),
            tokens_per_second: requests_per_second as f64,
            max_tokens: (requests_per_second * 2) as f64, // Allow burst of 2x
        }
    }

    /// Check if a request is allowed
    pub async fn check(&self, key: &str) -> bool {
        let mut buckets = self.buckets.write().await;

        let bucket = buckets
            .entry(key.to_string())
            .or_insert_with(|| TokenBucket::new(self.max_tokens));

        bucket.try_consume(self.tokens_per_second, self.max_tokens)
    }

    /// Cleanup old entries (call periodically)
    #[allow(dead_code)]
    pub async fn cleanup(&self, max_age: std::time::Duration) {
        let mut buckets = self.buckets.write().await;
        let now = Instant::now();

        buckets.retain(|_, bucket| now.duration_since(bucket.last_update) < max_age);
    }
}

/// Create rate limiting middleware layer
pub fn rate_limit_layer(
    rate_limiter: RateLimiter,
) -> impl Fn(
    Request<Body>,
    Next,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>>
       + Clone
       + Send {
    move |request: Request<Body>, next: Next| {
        let rate_limiter = rate_limiter.clone();
        Box::pin(async move {
            // Extract client identifier (IP address)
            let client_key = extract_client_ip(&request);

            if !rate_limiter.check(&client_key).await {
                tracing::warn!(client = %client_key, "Rate limit exceeded");
                return (
                    StatusCode::TOO_MANY_REQUESTS,
                    [(header::RETRY_AFTER, "1")],
                    "Too many requests. Please try again later.",
                )
                    .into_response();
            }

            next.run(request).await
        })
    }
}

/// Extract client IP from request headers
fn extract_client_ip(request: &Request<Body>) -> String {
    // Try X-Forwarded-For first
    if let Some(forwarded) = request.headers().get("x-forwarded-for") {
        if let Ok(s) = forwarded.to_str() {
            if let Some(ip) = s.split(',').next() {
                return ip.trim().to_string();
            }
        }
    }

    // Try X-Real-IP
    if let Some(real_ip) = request.headers().get("x-real-ip") {
        if let Ok(s) = real_ip.to_str() {
            return s.to_string();
        }
    }

    // Fallback to a default
    "unknown".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter() {
        let limiter = RateLimiter::new(5); // 5 requests per second

        // Should allow first 10 requests (burst capacity = 2x)
        for _ in 0..10 {
            assert!(limiter.check("test-client").await);
        }

        // Next request should be denied (bucket empty)
        assert!(!limiter.check("test-client").await);
    }

    #[tokio::test]
    async fn test_rate_limiter_different_clients() {
        let limiter = RateLimiter::new(2);

        // Different clients have separate buckets
        assert!(limiter.check("client-a").await);
        assert!(limiter.check("client-b").await);
        assert!(limiter.check("client-a").await);
        assert!(limiter.check("client-b").await);
    }
}

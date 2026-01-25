//! Middleware for StelloVault API
//!
//! This module provides middleware for request tracing, rate limiting,
//! and security headers.

mod rate_limiter;
mod security;
mod tracing;

pub use rate_limiter::{rate_limit_layer, RateLimiter};
pub use security::{hsts_header, security_headers};
pub use tracing::request_tracing;

// Re-export auth middleware placeholder
pub async fn auth_middleware(
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    // TODO: Implement authentication middleware
    next.run(request).await
}

//! Middleware for StelloVault API
//!
//! This module provides middleware for request tracing, rate limiting,
//! security headers, and authentication.

pub mod auth;
mod rate_limiter;
mod security;
mod tracing;

pub use auth::{AdminUser, AuthenticatedUser, OptionalUser};
pub use rate_limiter::{rate_limit_layer, RateLimiter};
pub use security::{hsts_header, security_headers};
pub use tracing::request_tracing;

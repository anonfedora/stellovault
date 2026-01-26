//! Authentication module for StelloVault
//!
//! Provides wallet-based authentication using Stellar addresses.
//! - Challenge-response authentication with nonces
//! - JWT token generation and validation
//! - Session management with refresh tokens

#[allow(unused_imports)]
mod crypto;
mod jwt;
mod service;

#[allow(unused_imports)]
pub use crypto::verify_stellar_signature;
#[allow(unused_imports)]
pub use jwt::{generate_access_token, generate_refresh_token, verify_token, Claims};
pub use service::AuthService;

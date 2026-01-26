//! API handlers for StelloVault backend

pub mod analytics;
pub mod auth;
mod escrow;
pub mod user;
pub mod wallet;

pub use analytics::get_analytics;
pub use auth::*;
pub use escrow::*;
pub use user::{create_user, get_user};
pub use wallet::*;

// Re-export AuthenticatedUser from middleware for handler use
pub use crate::middleware::auth::{AdminUser, AuthenticatedUser, OptionalUser};

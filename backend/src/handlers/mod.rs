//! API handlers for StelloVault backend

pub mod analytics;
pub mod auth;
pub mod collateral;
pub mod document;
mod escrow;
pub mod kyc;
pub mod oracle;
pub mod risk;
pub mod risk_score;
pub mod user;
pub mod wallet;

pub use analytics::get_analytics;
pub use auth::*;
pub use collateral::*;
pub use document::*;
pub use escrow::*;
pub use risk::*;
pub use risk_score::*;
pub use kyc::*;
pub use user::{create_user, get_user};
pub use wallet::*;

// Re-export AuthenticatedUser from middleware for handler use
pub use crate::middleware::auth::{AdminUser, AuthenticatedUser, OptionalUser};

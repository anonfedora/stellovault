//! Route definitions for StelloVault API

mod analytics;
mod auth;
mod collateral;
mod escrow;
mod loan;
mod risk;
mod user;
mod wallet;

pub use analytics::analytics_routes;
pub use auth::auth_routes;
pub use collateral::collateral_routes;
pub use escrow::escrow_routes;
pub use loan::loan_routes;
pub use risk::risk_routes;
pub use user::user_routes;
pub use wallet::wallet_routes;

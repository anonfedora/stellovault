//! Route definitions for StelloVault API

mod analytics;
mod collateral;
mod escrow;
mod loan;
mod user;

pub use analytics::analytics_routes;
pub use collateral::collateral_routes;
pub use escrow::escrow_routes;
pub use loan::loan_routes;
pub use user::user_routes;

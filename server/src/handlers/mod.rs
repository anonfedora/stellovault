//! API handlers for StelloVault backend

pub mod analytics;
pub mod auth;
pub mod collateral;
mod escrow;
pub mod user;
pub mod wallet;

#[allow(unused_imports)]
pub use analytics::get_analytics;
#[allow(unused_imports)]
pub use auth::*;
#[allow(unused_imports)]
pub use collateral::*;
#[allow(unused_imports)]
pub use escrow::{
    create_escrow, create_loan, get_escrow, get_loan, list_escrows, list_loans, record_repayment,
    webhook_escrow_update,
};
#[allow(unused_imports)]
pub use user::{create_user, get_user};
#[allow(unused_imports)]
pub use wallet::*;

// Re-export AuthenticatedUser from middleware for handler use
#[allow(unused_imports)]
pub use crate::middleware::auth::{AdminUser, AuthenticatedUser, OptionalUser};

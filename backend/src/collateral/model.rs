use serde::Deserialize;
use uuid::Uuid;

pub use crate::models::{Collateral, CollateralStatus, PaginatedResponse};

#[derive(Debug, Deserialize)]
pub struct CreateCollateralRequest {
    pub owner_id: Uuid,
    pub collateral_id: String,
    pub face_value: i64,
    pub expiry_ts: i64,
    pub metadata_hash: String,
}

#[derive(Debug, Deserialize)]
pub struct CollateralFilter {
    pub owner_id: Option<Uuid>,
    pub status: Option<CollateralStatus>,
    pub page: Option<i32>,
    pub limit: Option<i32>,
}

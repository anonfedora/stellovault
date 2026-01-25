//! Application state shared across handlers

use std::sync::Arc;

use crate::collateral::CollateralService;
use crate::escrow::EscrowService;
use crate::loan_service::LoanService;
use crate::websocket::WsState;

use axum::extract::FromRef;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub escrow_service: Arc<EscrowService>,
    pub collateral_service: Arc<CollateralService>,
    pub loan_service: Arc<LoanService>,
    pub ws_state: WsState,
    pub webhook_secret: Option<String>,
}

impl AppState {
    pub fn new(
        escrow_service: Arc<EscrowService>,
        collateral_service: Arc<CollateralService>,
        loan_service: Arc<LoanService>,
        ws_state: WsState,
        webhook_secret: Option<String>,
    ) -> Self {
        Self {
            escrow_service,
            collateral_service,
            loan_service,
            ws_state,
            webhook_secret,
        }
    }
}

impl FromRef<AppState> for WsState {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.ws_state.clone()
    }
}

impl FromRef<AppState> for Arc<EscrowService> {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.escrow_service.clone()
    }
}

impl FromRef<AppState> for Arc<CollateralService> {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.collateral_service.clone()
    }
}

impl FromRef<AppState> for Arc<LoanService> {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.loan_service.clone()
    }
}

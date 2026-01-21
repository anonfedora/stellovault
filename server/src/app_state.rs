//! Application state shared across handlers

use std::sync::Arc;

use crate::escrow_service::EscrowService;
use crate::websocket::WsState;

use axum::extract::FromRef;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub escrow_service: Arc<EscrowService>,
    pub ws_state: WsState,
}

impl AppState {
    pub fn new(escrow_service: Arc<EscrowService>, ws_state: WsState) -> Self {
        Self {
            escrow_service,
            ws_state,
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

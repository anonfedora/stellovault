//! StelloVault Backend Library
//!
//! This library exports the core modules for the StelloVault backend server.

pub mod app_state;
pub mod auth;
pub mod collateral;
pub mod config;
pub mod db;
pub mod defi_history;
pub mod document_verification;
pub mod error;
pub mod escrow;
pub mod governance_service;
pub mod handlers;
pub mod indexer;
pub mod loan;
pub mod loan_service;
pub mod middleware;
pub mod models;
pub mod oracle;
pub mod oracle_service;
pub mod routes;
pub mod services;
pub mod state;
pub mod websocket;

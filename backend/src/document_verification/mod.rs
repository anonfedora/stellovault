//! Document Verification Module
//!
//! AI-based OCR for invoice and document verification

pub mod model;
pub mod ocr_provider;
pub mod service;
pub mod validator;

pub use model::*;
pub use service::DocumentVerificationService;

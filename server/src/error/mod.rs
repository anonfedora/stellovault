//! Centralized API error handling for StelloVault
//!
//! This module provides a unified error type for API responses with proper
//! HTTP status code mapping and JSON error responses.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

/// API error type with HTTP status code mapping
#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Unprocessable entity: {0}")]
    UnprocessableEntity(String),

    #[error("Too many requests")]
    TooManyRequests,

    #[error("Internal server error: {0}")]
    InternalError(String),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("External service error: {0}")]
    ExternalServiceError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),
}

/// JSON error response body
#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: ErrorDetails,
}

/// Error details in the response
#[derive(Serialize)]
pub struct ErrorDetails {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl ApiError {
    /// Get the error code string
    pub fn error_code(&self) -> &'static str {
        match self {
            ApiError::NotFound(_) => "NOT_FOUND",
            ApiError::BadRequest(_) => "BAD_REQUEST",
            ApiError::Unauthorized(_) => "UNAUTHORIZED",
            ApiError::Forbidden(_) => "FORBIDDEN",
            ApiError::Conflict(_) => "CONFLICT",
            ApiError::UnprocessableEntity(_) => "UNPROCESSABLE_ENTITY",
            ApiError::TooManyRequests => "TOO_MANY_REQUESTS",
            ApiError::InternalError(_) => "INTERNAL_ERROR",
            ApiError::ServiceUnavailable(_) => "SERVICE_UNAVAILABLE",
            ApiError::DatabaseError(_) => "DATABASE_ERROR",
            ApiError::ExternalServiceError(_) => "EXTERNAL_SERVICE_ERROR",
            ApiError::ValidationError(_) => "VALIDATION_ERROR",
        }
    }

    /// Get the HTTP status code
    pub fn status_code(&self) -> StatusCode {
        match self {
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            ApiError::Forbidden(_) => StatusCode::FORBIDDEN,
            ApiError::Conflict(_) => StatusCode::CONFLICT,
            ApiError::UnprocessableEntity(_) => StatusCode::UNPROCESSABLE_ENTITY,
            ApiError::TooManyRequests => StatusCode::TOO_MANY_REQUESTS,
            ApiError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            ApiError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::ExternalServiceError(_) => StatusCode::BAD_GATEWAY,
            ApiError::ValidationError(_) => StatusCode::BAD_REQUEST,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let error_code = self.error_code();
        let message = self.to_string();

        // Log server errors
        match &self {
            ApiError::InternalError(_)
            | ApiError::DatabaseError(_)
            | ApiError::ServiceUnavailable(_) => {
                tracing::error!(error = %message, code = %error_code, "Server error occurred");
            }
            _ => {
                tracing::debug!(error = %message, code = %error_code, "Client error occurred");
            }
        }

        let body = ErrorResponse {
            error: ErrorDetails {
                code: error_code.to_string(),
                message,
                details: None,
            },
        };

        (status, Json(body)).into_response()
    }
}

// Convenience conversions from common error types

impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => ApiError::NotFound("Resource not found".to_string()),
            _ => ApiError::DatabaseError(err.to_string()),
        }
    }
}

impl From<validator::ValidationErrors> for ApiError {
    fn from(err: validator::ValidationErrors) -> Self {
        ApiError::ValidationError(err.to_string())
    }
}

impl From<reqwest::Error> for ApiError {
    fn from(err: reqwest::Error) -> Self {
        ApiError::ExternalServiceError(err.to_string())
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        ApiError::BadRequest(format!("Invalid JSON: {}", err))
    }
}

/// Result type alias using ApiError
pub type ApiResult<T> = Result<T, ApiError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        assert_eq!(
            ApiError::NotFound("test".to_string()).error_code(),
            "NOT_FOUND"
        );
        assert_eq!(
            ApiError::BadRequest("test".to_string()).error_code(),
            "BAD_REQUEST"
        );
        assert_eq!(
            ApiError::Unauthorized("test".to_string()).error_code(),
            "UNAUTHORIZED"
        );
        assert_eq!(ApiError::TooManyRequests.error_code(), "TOO_MANY_REQUESTS");
    }

    #[test]
    fn test_status_codes() {
        assert_eq!(
            ApiError::NotFound("test".to_string()).status_code(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            ApiError::BadRequest("test".to_string()).status_code(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            ApiError::TooManyRequests.status_code(),
            StatusCode::TOO_MANY_REQUESTS
        );
        assert_eq!(
            ApiError::InternalError("test".to_string()).status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }
}

//! Request tracing middleware

use axum::{extract::Request, middleware::Next, response::Response};
use std::time::Instant;

/// Middleware for logging request information with timing
pub async fn request_tracing(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let path = uri.path().to_string();

    // Extract client IP if available
    let client_ip = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or(s).trim().to_string())
        .or_else(|| {
            request
                .headers()
                .get("x-real-ip")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string())
        });

    let start = Instant::now();

    // Log request
    tracing::info!(
        method = %method,
        path = %path,
        client_ip = ?client_ip,
        "Request started"
    );

    // Execute the request
    let response = next.run(request).await;

    let duration = start.elapsed();
    let status = response.status();

    // Log response
    if status.is_server_error() {
        tracing::error!(
            method = %method,
            path = %path,
            status = %status.as_u16(),
            duration_ms = %duration.as_millis(),
            "Request completed with error"
        );
    } else if status.is_client_error() {
        tracing::warn!(
            method = %method,
            path = %path,
            status = %status.as_u16(),
            duration_ms = %duration.as_millis(),
            "Request completed with client error"
        );
    } else {
        tracing::info!(
            method = %method,
            path = %path,
            status = %status.as_u16(),
            duration_ms = %duration.as_millis(),
            "Request completed"
        );
    }

    response
}

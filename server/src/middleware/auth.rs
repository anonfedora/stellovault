//! Authentication middleware
//!
//! Middleware for JWT token verification and user extraction.

use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::auth::{verify_token, AuthService};
use crate::models::UserRole;

/// Authenticated user extracted from JWT token
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: Uuid,
    pub wallet_address: String,
    pub role: UserRole,
    pub jti: String,
}

/// Error response for authentication failures
#[derive(Debug, Serialize)]
struct AuthError {
    error: AuthErrorDetails,
}

#[derive(Debug, Serialize)]
struct AuthErrorDetails {
    code: String,
    message: String,
}

impl AuthError {
    fn new(code: &str, message: &str) -> Self {
        Self {
            error: AuthErrorDetails {
                code: code.to_string(),
                message: message.to_string(),
            },
        }
    }
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        (StatusCode::UNAUTHORIZED, Json(self)).into_response()
    }
}

/// Extractor for authenticated users
///
/// This extractor verifies the JWT token from the Authorization header
/// and extracts the authenticated user information.
///
/// # Example
///
/// ```rust,ignore
/// async fn protected_handler(user: AuthenticatedUser) -> impl IntoResponse {
///     format!("Hello, user {}", user.user_id)
/// }
/// ```
#[async_trait]
impl<S> FromRequestParts<S> for AuthenticatedUser
where
    Arc<AuthService>: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // Extract the Authorization header
        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state)
                .await
                .map_err(|_| {
                    AuthError::new(
                        "MISSING_TOKEN",
                        "Authorization header with Bearer token required",
                    )
                    .into_response()
                })?;

        // Get the auth service from state
        let auth_service = Arc::<AuthService>::from_ref(state);

        // Verify the token
        let claims = verify_token(bearer.token(), auth_service.jwt_secret()).map_err(|e| {
            let (code, message) = match e.to_string().as_str() {
                s if s.contains("expired") => ("TOKEN_EXPIRED", "Token has expired"),
                _ => ("INVALID_TOKEN", "Invalid token"),
            };
            AuthError::new(code, message).into_response()
        })?;

        // Check token type is access
        if claims.token_type != "access" {
            return Err(
                AuthError::new("INVALID_TOKEN_TYPE", "Expected access token").into_response(),
            );
        }

        // Parse user ID
        let user_id = Uuid::parse_str(&claims.sub).map_err(|_| {
            AuthError::new("INVALID_TOKEN", "Invalid user ID in token").into_response()
        })?;

        // Parse role
        let role = match claims.role.as_str() {
            "buyer" => UserRole::Buyer,
            "seller" => UserRole::Seller,
            "oracle" => UserRole::Oracle,
            "admin" => UserRole::Admin,
            _ => {
                return Err(AuthError::new("INVALID_TOKEN", "Invalid role in token").into_response())
            }
        };

        // Verify session is still valid (not revoked)
        auth_service
            .verify_session(&claims.jti)
            .await
            .map_err(|_| {
                AuthError::new("SESSION_REVOKED", "Session has been revoked").into_response()
            })?;

        Ok(AuthenticatedUser {
            user_id,
            wallet_address: claims.wallet,
            role,
            jti: claims.jti,
        })
    }
}

/// Optional authenticated user extractor
///
/// This extractor attempts to authenticate but doesn't fail if no token is present.
#[derive(Debug, Clone)]
pub struct OptionalUser(pub Option<AuthenticatedUser>);

#[async_trait]
impl<S> FromRequestParts<S> for OptionalUser
where
    Arc<AuthService>: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        match AuthenticatedUser::from_request_parts(parts, state).await {
            Ok(user) => Ok(OptionalUser(Some(user))),
            Err(_) => Ok(OptionalUser(None)),
        }
    }
}

/// Middleware to require admin role
pub struct AdminUser(pub AuthenticatedUser);

#[async_trait]
impl<S> FromRequestParts<S> for AdminUser
where
    Arc<AuthService>: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let user = AuthenticatedUser::from_request_parts(parts, state).await?;

        if !matches!(user.role, UserRole::Admin) {
            return Err(AuthError::new("FORBIDDEN", "Admin access required").into_response());
        }

        Ok(AdminUser(user))
    }
}

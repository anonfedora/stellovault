//! JWT token generation and validation
//!
//! Handles creation and verification of access and refresh tokens.

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::models::{User, UserRole};

/// JWT-related errors
#[derive(Error, Debug)]
pub enum JwtError {
    #[error("Token encoding failed: {0}")]
    EncodingFailed(String),

    #[error("Token decoding failed: {0}")]
    DecodingFailed(String),

    #[error("Token expired")]
    TokenExpired,

    #[error("Invalid token: {0}")]
    InvalidToken(String),
}

/// JWT claims for access tokens
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,
    /// Primary wallet address
    pub wallet: String,
    /// User role
    pub role: String,
    /// JWT ID (for revocation)
    pub jti: String,
    /// Issued at (Unix timestamp)
    pub iat: i64,
    /// Expiration (Unix timestamp)
    pub exp: i64,
    /// Token type (access or refresh)
    pub token_type: String,
}

/// Token type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    Access,
    Refresh,
}

impl TokenType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TokenType::Access => "access",
            TokenType::Refresh => "refresh",
        }
    }
}

/// Generate an access token for a user
///
/// # Arguments
/// * `user` - The authenticated user
/// * `jti` - Unique token identifier for revocation
/// * `secret` - JWT signing secret
/// * `ttl_seconds` - Token time-to-live in seconds
pub fn generate_access_token(
    user: &User,
    jti: &str,
    secret: &str,
    ttl_seconds: i64,
) -> Result<String, JwtError> {
    generate_token(user, jti, secret, ttl_seconds, TokenType::Access)
}

/// Generate a refresh token for a user
///
/// # Arguments
/// * `user` - The authenticated user
/// * `jti` - Unique token identifier for revocation
/// * `secret` - JWT signing secret
/// * `ttl_days` - Token time-to-live in days
pub fn generate_refresh_token(
    user: &User,
    jti: &str,
    secret: &str,
    ttl_days: i64,
) -> Result<String, JwtError> {
    let ttl_seconds = ttl_days * 24 * 60 * 60;
    generate_token(user, jti, secret, ttl_seconds, TokenType::Refresh)
}

/// Internal function to generate tokens
fn generate_token(
    user: &User,
    jti: &str,
    secret: &str,
    ttl_seconds: i64,
    token_type: TokenType,
) -> Result<String, JwtError> {
    let now = Utc::now();
    let exp = now + Duration::seconds(ttl_seconds);

    let role = match user.role {
        UserRole::Buyer => "buyer",
        UserRole::Seller => "seller",
        UserRole::Oracle => "oracle",
        UserRole::Admin => "admin",
    };

    let claims = Claims {
        sub: user.id.to_string(),
        wallet: user.primary_wallet_address.clone(),
        role: role.to_string(),
        jti: jti.to_string(),
        iat: now.timestamp(),
        exp: exp.timestamp(),
        token_type: token_type.as_str().to_string(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| JwtError::EncodingFailed(e.to_string()))
}

/// Verify and decode a JWT token
///
/// # Arguments
/// * `token` - The JWT token string
/// * `secret` - JWT signing secret
///
/// # Returns
/// * `Ok(Claims)` if token is valid
/// * `Err(JwtError)` if validation fails
pub fn verify_token(token: &str, secret: &str) -> Result<Claims, JwtError> {
    let mut validation = Validation::default();
    validation.validate_exp = true;

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map_err(|e| {
        if e.to_string().contains("ExpiredSignature") {
            JwtError::TokenExpired
        } else {
            JwtError::DecodingFailed(e.to_string())
        }
    })?;

    Ok(token_data.claims)
}

/// Extract user ID from claims
#[allow(dead_code)]
pub fn get_user_id_from_claims(claims: &Claims) -> Result<Uuid, JwtError> {
    Uuid::parse_str(&claims.sub).map_err(|e| JwtError::InvalidToken(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_user() -> User {
        User {
            id: Uuid::new_v4(),
            primary_wallet_address: "GAAZI4TCR3TY5OJHCTJC2A4QSY6CJWJH5IAJTGKIN2ER7LBNVKOCCWN7"
                .to_string(),
            email: Some("test@example.com".to_string()),
            name: Some("Test User".to_string()),
            role: UserRole::Buyer,
            risk_score: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_generate_access_token() {
        let user = create_test_user();
        let jti = Uuid::new_v4().to_string();
        let secret = "test-secret-key";

        let token = generate_access_token(&user, &jti, secret, 900).unwrap();
        assert!(!token.is_empty());

        // Verify the token
        let claims = verify_token(&token, secret).unwrap();
        assert_eq!(claims.sub, user.id.to_string());
        assert_eq!(claims.wallet, user.primary_wallet_address);
        assert_eq!(claims.token_type, "access");
    }

    #[test]
    fn test_generate_refresh_token() {
        let user = create_test_user();
        let jti = Uuid::new_v4().to_string();
        let secret = "test-secret-key";

        let token = generate_refresh_token(&user, &jti, secret, 7).unwrap();
        assert!(!token.is_empty());

        let claims = verify_token(&token, secret).unwrap();
        assert_eq!(claims.token_type, "refresh");
    }

    #[test]
    fn test_invalid_token() {
        let secret = "test-secret-key";
        let result = verify_token("invalid.token.here", secret);
        assert!(result.is_err());
    }

    #[test]
    fn test_wrong_secret() {
        let user = create_test_user();
        let jti = Uuid::new_v4().to_string();

        let token = generate_access_token(&user, &jti, "secret1", 900).unwrap();
        let result = verify_token(&token, "secret2");
        assert!(result.is_err());
    }
}

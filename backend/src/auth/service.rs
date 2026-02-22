//! Authentication service
//!
//! Core business logic for wallet-based authentication.

use chrono::{Duration, Utc};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use thiserror::Error;
use uuid::Uuid;

use crate::models::{
    AuthNonce, AuthSession, AuthTokensResponse, ChallengeResponse, User, UserRole, Wallet,
};

use super::crypto::{verify_stellar_signature, CryptoError};
use super::jwt::{generate_access_token, generate_refresh_token, verify_token, JwtError};

/// Auth service errors
#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Invalid wallet address: {0}")]
    InvalidWalletAddress(String),

    #[error("Nonce not found or expired")]
    NonceNotFound,

    #[error("Nonce already used")]
    NonceAlreadyUsed,

    #[error("Nonce expired")]
    NonceExpired,

    #[error("Invalid signature: {0}")]
    InvalidSignature(String),

    #[error("User not found")]
    UserNotFound,

    #[error("Session not found or revoked")]
    SessionNotFound,

    #[error("Token error: {0}")]
    TokenError(String),

    #[error("Invalid refresh token")]
    InvalidRefreshToken,

    #[error("Wallet already linked to another user")]
    WalletAlreadyLinked,

    #[error("Cannot remove primary wallet")]
    CannotRemovePrimaryWallet,

    #[error("User must have at least one wallet")]
    MustHaveOneWallet,
}

impl From<sqlx::Error> for AuthError {
    fn from(e: sqlx::Error) -> Self {
        AuthError::DatabaseError(e.to_string())
    }
}

impl From<CryptoError> for AuthError {
    fn from(e: CryptoError) -> Self {
        AuthError::InvalidSignature(e.to_string())
    }
}

impl From<JwtError> for AuthError {
    fn from(e: JwtError) -> Self {
        AuthError::TokenError(e.to_string())
    }
}

/// Authentication service
#[derive(Clone)]
pub struct AuthService {
    db_pool: PgPool,
    jwt_secret: String,
    nonce_ttl_seconds: i64,
    access_token_ttl_seconds: i64,
    refresh_token_ttl_days: i64,
}

impl AuthService {
    /// Create a new AuthService
    pub fn new(
        db_pool: PgPool,
        jwt_secret: String,
        nonce_ttl_seconds: i64,
        access_token_ttl_seconds: i64,
        refresh_token_ttl_days: i64,
    ) -> Self {
        Self {
            db_pool,
            jwt_secret,
            nonce_ttl_seconds,
            access_token_ttl_seconds,
            refresh_token_ttl_days,
        }
    }

    /// Generate a nonce challenge for wallet authentication
    pub async fn generate_challenge(
        &self,
        wallet_address: &str,
    ) -> Result<ChallengeResponse, AuthError> {
        // Validate wallet address format (starts with G for public keys)
        if !wallet_address.starts_with('G') || wallet_address.len() != 56 {
            return Err(AuthError::InvalidWalletAddress(
                "Invalid Stellar address format".to_string(),
            ));
        }

        // Generate secure random nonce
        let nonce = generate_secure_nonce();
        let expires_at = Utc::now() + Duration::seconds(self.nonce_ttl_seconds);

        // Create human-readable message to sign
        let message = format!(
            "Sign this message to authenticate with StelloVault:\n\nNonce: {}\nWallet: {}\nExpires: {}",
            nonce,
            wallet_address,
            expires_at.format("%Y-%m-%d %H:%M:%S UTC")
        );

        // Store nonce in database
        sqlx::query(
            r#"
            INSERT INTO auth_nonces (id, nonce, wallet_address, message, expires_at)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(&nonce)
        .bind(wallet_address)
        .bind(&message)
        .bind(expires_at)
        .execute(&self.db_pool)
        .await?;

        Ok(ChallengeResponse {
            nonce,
            message,
            expires_at,
        })
    }

    /// Verify a signed message and issue tokens
    pub async fn verify_signature(
        &self,
        wallet_address: &str,
        nonce: &str,
        signature: &str,
        device_info: Option<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<AuthTokensResponse, AuthError> {
        // Fetch and validate nonce
        let auth_nonce: AuthNonce = sqlx::query_as(
            r#"
            SELECT id, nonce, wallet_address, message, expires_at, used, used_at, created_at
            FROM auth_nonces
            WHERE nonce = $1 AND wallet_address = $2
            "#,
        )
        .bind(nonce)
        .bind(wallet_address)
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or(AuthError::NonceNotFound)?;

        // Check if nonce is already used (replay attack prevention)
        if auth_nonce.used {
            return Err(AuthError::NonceAlreadyUsed);
        }

        // Check if nonce is expired
        if auth_nonce.expires_at < Utc::now() {
            return Err(AuthError::NonceExpired);
        }

        // Verify the signature
        verify_stellar_signature(wallet_address, &auth_nonce.message, signature)?;

        // Mark nonce as used immediately (atomic operation for replay prevention)
        let rows_affected = sqlx::query(
            r#"
            UPDATE auth_nonces
            SET used = TRUE, used_at = NOW()
            WHERE id = $1 AND used = FALSE
            "#,
        )
        .bind(auth_nonce.id)
        .execute(&self.db_pool)
        .await?
        .rows_affected();

        // If no rows were affected, another request already used this nonce
        if rows_affected == 0 {
            return Err(AuthError::NonceAlreadyUsed);
        }

        // Get or create user
        let user = self.get_or_create_user(wallet_address).await?;

        // Generate tokens
        let jti = Uuid::new_v4().to_string();
        let access_token =
            generate_access_token(&user, &jti, &self.jwt_secret, self.access_token_ttl_seconds)?;

        let refresh_jti = Uuid::new_v4().to_string();
        let refresh_token = generate_refresh_token(
            &user,
            &refresh_jti,
            &self.jwt_secret,
            self.refresh_token_ttl_days,
        )?;

        // Hash refresh token for storage
        let refresh_token_hash = hash_token(&refresh_token);

        // Calculate session expiration (refresh token lifetime)
        let session_expires_at = Utc::now() + Duration::days(self.refresh_token_ttl_days);

        // Create session
        sqlx::query(
            r#"
            INSERT INTO auth_sessions (id, user_id, jti, refresh_token_hash, device_info, ip_address, user_agent, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(user.id)
        .bind(&jti)
        .bind(&refresh_token_hash)
        .bind(&device_info)
        .bind(&ip_address)
        .bind(&user_agent)
        .bind(session_expires_at)
        .execute(&self.db_pool)
        .await?;

        Ok(AuthTokensResponse {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: self.access_token_ttl_seconds,
            user: user.into(),
        })
    }

    /// Get or create a user by wallet address
    async fn get_or_create_user(&self, wallet_address: &str) -> Result<User, AuthError> {
        // Try to find existing user
        let existing_user: Option<User> = sqlx::query_as(
            r#"
            SELECT id, primary_wallet_address, email, name, role, risk_score, created_at, updated_at
            FROM users
            WHERE primary_wallet_address = $1
            "#,
        )
        .bind(wallet_address)
        .fetch_optional(&self.db_pool)
        .await?;

        if let Some(user) = existing_user {
            return Ok(user);
        }

        // Also check wallets table for linked wallets
        let linked_wallet: Option<Wallet> = sqlx::query_as(
            r#"
            SELECT id, user_id, wallet_address, is_primary, label, verified_at, created_at, updated_at
            FROM wallets
            WHERE wallet_address = $1
            "#,
        )
        .bind(wallet_address)
        .fetch_optional(&self.db_pool)
        .await?;

        if let Some(wallet) = linked_wallet {
            // Return the user associated with this wallet
            let user: User = sqlx::query_as(
                r#"
                SELECT id, primary_wallet_address, email, name, role, risk_score, created_at, updated_at
                FROM users
                WHERE id = $1
                "#,
            )
            .bind(wallet.user_id)
            .fetch_one(&self.db_pool)
            .await?;
            return Ok(user);
        }

        // Create new user
        let user_id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO users (id, primary_wallet_address, role, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(user_id)
        .bind(wallet_address)
        .bind(UserRole::Buyer)
        .bind(now)
        .bind(now)
        .execute(&self.db_pool)
        .await?;

        // Also create a wallet entry
        sqlx::query(
            r#"
            INSERT INTO wallets (id, user_id, wallet_address, is_primary, verified_at, created_at, updated_at)
            VALUES ($1, $2, $3, TRUE, $4, $5, $6)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(wallet_address)
        .bind(now)
        .bind(now)
        .bind(now)
        .execute(&self.db_pool)
        .await?;

        Ok(User {
            id: user_id,
            primary_wallet_address: wallet_address.to_string(),
            email: None,
            name: None,
            role: UserRole::Buyer,
            risk_score: None,
            created_at: now,
            updated_at: now,
        })
    }

    /// Refresh tokens using a valid refresh token
    pub async fn refresh_tokens(
        &self,
        refresh_token: &str,
    ) -> Result<AuthTokensResponse, AuthError> {
        // Verify the refresh token
        let claims = verify_token(refresh_token, &self.jwt_secret)?;

        if claims.token_type != "refresh" {
            return Err(AuthError::InvalidRefreshToken);
        }

        // Hash the refresh token to find the session
        let refresh_token_hash = hash_token(refresh_token);

        // Find the session and verify it's not revoked
        let session: AuthSession = sqlx::query_as(
            r#"
            SELECT id, user_id, jti, refresh_token_hash, device_info, ip_address, user_agent, expires_at, revoked, revoked_at, created_at, updated_at
            FROM auth_sessions
            WHERE refresh_token_hash = $1 AND revoked = FALSE AND expires_at > NOW()
            "#,
        )
        .bind(&refresh_token_hash)
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or(AuthError::SessionNotFound)?;

        // Get the user
        let user = self.get_user_by_id(session.user_id).await?;

        // Generate new tokens
        let jti = Uuid::new_v4().to_string();
        let access_token =
            generate_access_token(&user, &jti, &self.jwt_secret, self.access_token_ttl_seconds)?;

        let refresh_jti = Uuid::new_v4().to_string();
        let new_refresh_token = generate_refresh_token(
            &user,
            &refresh_jti,
            &self.jwt_secret,
            self.refresh_token_ttl_days,
        )?;

        let new_refresh_token_hash = hash_token(&new_refresh_token);
        let session_expires_at = Utc::now() + Duration::days(self.refresh_token_ttl_days);

        // Update the session with new refresh token
        sqlx::query(
            r#"
            UPDATE auth_sessions
            SET jti = $1, refresh_token_hash = $2, expires_at = $3, updated_at = NOW()
            WHERE id = $4
            "#,
        )
        .bind(&jti)
        .bind(&new_refresh_token_hash)
        .bind(session_expires_at)
        .bind(session.id)
        .execute(&self.db_pool)
        .await?;

        Ok(AuthTokensResponse {
            access_token,
            refresh_token: new_refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: self.access_token_ttl_seconds,
            user: user.into(),
        })
    }

    /// Revoke a session (logout)
    pub async fn revoke_session(&self, jti: &str) -> Result<(), AuthError> {
        let rows_affected = sqlx::query(
            r#"
            UPDATE auth_sessions
            SET revoked = TRUE, revoked_at = NOW()
            WHERE jti = $1 AND revoked = FALSE
            "#,
        )
        .bind(jti)
        .execute(&self.db_pool)
        .await?
        .rows_affected();

        if rows_affected == 0 {
            return Err(AuthError::SessionNotFound);
        }

        Ok(())
    }

    /// Revoke all sessions for a user
    pub async fn revoke_all_sessions(&self, user_id: Uuid) -> Result<u64, AuthError> {
        let rows_affected = sqlx::query(
            r#"
            UPDATE auth_sessions
            SET revoked = TRUE, revoked_at = NOW()
            WHERE user_id = $1 AND revoked = FALSE
            "#,
        )
        .bind(user_id)
        .execute(&self.db_pool)
        .await?
        .rows_affected();

        Ok(rows_affected)
    }

    /// Get a user by ID
    pub async fn get_user_by_id(&self, user_id: Uuid) -> Result<User, AuthError> {
        sqlx::query_as(
            r#"
            SELECT id, primary_wallet_address, email, name, role, risk_score, created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or(AuthError::UserNotFound)
    }

    /// Verify a session is valid (not revoked)
    pub async fn verify_session(&self, jti: &str) -> Result<AuthSession, AuthError> {
        sqlx::query_as(
            r#"
            SELECT id, user_id, jti, refresh_token_hash, device_info, ip_address, user_agent, expires_at, revoked, revoked_at, created_at, updated_at
            FROM auth_sessions
            WHERE jti = $1 AND revoked = FALSE AND expires_at > NOW()
            "#,
        )
        .bind(jti)
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or(AuthError::SessionNotFound)
    }

    /// Get wallets for a user
    pub async fn get_user_wallets(&self, user_id: Uuid) -> Result<Vec<Wallet>, AuthError> {
        let wallets: Vec<Wallet> = sqlx::query_as(
            r#"
            SELECT id, user_id, wallet_address, is_primary, label, verified_at, created_at, updated_at
            FROM wallets
            WHERE user_id = $1
            ORDER BY is_primary DESC, created_at ASC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.db_pool)
        .await?;

        Ok(wallets)
    }

    /// Link a new wallet to a user (after signature verification)
    pub async fn link_wallet(
        &self,
        user_id: Uuid,
        wallet_address: &str,
        nonce: &str,
        signature: &str,
        label: Option<String>,
    ) -> Result<Wallet, AuthError> {
        // Check if wallet is already linked
        let existing: Option<Wallet> = sqlx::query_as(
            r#"
            SELECT id, user_id, wallet_address, is_primary, label, verified_at, created_at, updated_at
            FROM wallets
            WHERE wallet_address = $1
            "#,
        )
        .bind(wallet_address)
        .fetch_optional(&self.db_pool)
        .await?;

        if existing.is_some() {
            return Err(AuthError::WalletAlreadyLinked);
        }

        // Validate nonce and signature
        let auth_nonce: AuthNonce = sqlx::query_as(
            r#"
            SELECT id, nonce, wallet_address, message, expires_at, used, used_at, created_at
            FROM auth_nonces
            WHERE nonce = $1 AND wallet_address = $2 AND used = FALSE AND expires_at > NOW()
            "#,
        )
        .bind(nonce)
        .bind(wallet_address)
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or(AuthError::NonceNotFound)?;

        // Verify signature
        verify_stellar_signature(wallet_address, &auth_nonce.message, signature)?;

        // Mark nonce as used
        sqlx::query(
            r#"
            UPDATE auth_nonces
            SET used = TRUE, used_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(auth_nonce.id)
        .execute(&self.db_pool)
        .await?;

        // Create wallet
        let wallet_id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO wallets (id, user_id, wallet_address, is_primary, label, verified_at, created_at, updated_at)
            VALUES ($1, $2, $3, FALSE, $4, $5, $6, $7)
            "#,
        )
        .bind(wallet_id)
        .bind(user_id)
        .bind(wallet_address)
        .bind(&label)
        .bind(now)
        .bind(now)
        .bind(now)
        .execute(&self.db_pool)
        .await?;

        Ok(Wallet {
            id: wallet_id,
            user_id,
            wallet_address: wallet_address.to_string(),
            is_primary: false,
            label,
            verified_at: now,
            created_at: now,
            updated_at: now,
        })
    }

    /// Unlink a wallet from a user
    pub async fn unlink_wallet(&self, user_id: Uuid, wallet_id: Uuid) -> Result<(), AuthError> {
        // Get the wallet
        let wallet: Wallet = sqlx::query_as(
            r#"
            SELECT id, user_id, wallet_address, is_primary, label, verified_at, created_at, updated_at
            FROM wallets
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(wallet_id)
        .bind(user_id)
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or(AuthError::UserNotFound)?;

        // Cannot remove primary wallet
        if wallet.is_primary {
            return Err(AuthError::CannotRemovePrimaryWallet);
        }

        // Check user has more than one wallet
        let wallet_count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM wallets WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.db_pool)
        .await?;

        if wallet_count <= 1 {
            return Err(AuthError::MustHaveOneWallet);
        }

        // Delete the wallet
        sqlx::query(
            r#"
            DELETE FROM wallets WHERE id = $1
            "#,
        )
        .bind(wallet_id)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    /// Set a wallet as primary
    pub async fn set_primary_wallet(
        &self,
        user_id: Uuid,
        wallet_id: Uuid,
    ) -> Result<Wallet, AuthError> {
        // Get the wallet
        let wallet: Wallet = sqlx::query_as(
            r#"
            SELECT id, user_id, wallet_address, is_primary, label, verified_at, created_at, updated_at
            FROM wallets
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(wallet_id)
        .bind(user_id)
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or(AuthError::UserNotFound)?;

        // Begin transaction
        let mut tx = self.db_pool.begin().await?;

        // Unset all primary wallets for this user
        sqlx::query(
            r#"
            UPDATE wallets SET is_primary = FALSE WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

        // Set this wallet as primary
        sqlx::query(
            r#"
            UPDATE wallets SET is_primary = TRUE, updated_at = NOW() WHERE id = $1
            "#,
        )
        .bind(wallet_id)
        .execute(&mut *tx)
        .await?;

        // Update user's primary wallet address
        sqlx::query(
            r#"
            UPDATE users SET primary_wallet_address = $1, updated_at = NOW() WHERE id = $2
            "#,
        )
        .bind(&wallet.wallet_address)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(Wallet {
            is_primary: true,
            updated_at: Utc::now(),
            ..wallet
        })
    }

    /// Get JWT secret (for middleware access)
    pub fn jwt_secret(&self) -> &str {
        &self.jwt_secret
    }

    /// Get database pool (for handler access)
    pub fn db_pool(&self) -> &PgPool {
        &self.db_pool
    }
}

/// Generate a cryptographically secure nonce
fn generate_secure_nonce() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let bytes: [u8; 32] = rng.gen();
    hex::encode(bytes)
}

/// Hash a token for storage
fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

// We need hex crate for encoding
mod hex {
    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        bytes
            .as_ref()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect()
    }
}

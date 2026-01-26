//! Configuration management for StelloVault
//!
//! This module handles loading and validating configuration from environment variables,
//! with support for different environments (development, staging, production).

use std::env;
use thiserror::Error;

/// Configuration errors
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Missing required environment variable: {0}")]
    MissingEnvVar(String),

    #[error("Invalid environment value: {0}")]
    InvalidValue(String),

    #[error("Invalid port number: {0}")]
    InvalidPort(String),
}

/// Application environment
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Environment {
    Development,
    Staging,
    Production,
}

#[allow(dead_code)]
impl Environment {
    /// Parse environment from string
    pub fn from_str(s: &str) -> Result<Self, ConfigError> {
        match s.to_lowercase().as_str() {
            "dev" | "development" => Ok(Environment::Development),
            "staging" => Ok(Environment::Staging),
            "prod" | "production" => Ok(Environment::Production),
            _ => Err(ConfigError::InvalidValue(format!(
                "Invalid environment: '{}'. Expected: dev, staging, or prod",
                s
            ))),
        }
    }

    /// Check if this is a production environment
    pub fn is_production(&self) -> bool {
        matches!(self, Environment::Production)
    }

    /// Get the environment name as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Development => "development",
            Environment::Staging => "staging",
            Environment::Production => "production",
        }
    }
}

impl Default for Environment {
    fn default() -> Self {
        Environment::Development
    }
}

/// Application configuration
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Config {
    /// Database connection URL
    pub database_url: String,

    /// Soroban RPC URL for blockchain interactions
    pub soroban_rpc_url: String,

    /// Stellar Horizon API URL
    pub horizon_url: String,

    /// Network passphrase for Stellar network
    pub network_passphrase: String,

    /// Smart contract ID
    pub contract_id: String,

    /// Current environment
    pub environment: Environment,

    /// Server port
    pub port: u16,

    /// Maximum database connections
    pub db_max_connections: u32,

    /// Rate limit: requests per second per IP
    pub rate_limit_rps: u32,

    /// Webhook secret for external integrations
    pub webhook_secret: Option<String>,

    /// CORS allowed origins
    pub cors_allowed_origins: Option<String>,

    /// Log level (RUST_LOG)
    pub log_level: String,

    /// JWT secret for token signing
    pub jwt_secret: String,

    /// Access token TTL in seconds (default: 900 = 15 minutes)
    pub jwt_access_token_ttl_seconds: i64,

    /// Refresh token TTL in days (default: 7)
    pub jwt_refresh_token_ttl_days: i64,

    /// Auth nonce TTL in seconds (default: 300 = 5 minutes)
    pub auth_nonce_ttl_seconds: i64,
}

#[allow(dead_code)]
impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, ConfigError> {
        // Load .env file if present (ignore errors)
        dotenvy::dotenv().ok();

        let environment = env::var("ENVIRONMENT")
            .map(|s| Environment::from_str(&s))
            .unwrap_or(Ok(Environment::Development))?;

        let database_url = env::var("DATABASE_URL")
            .map_err(|_| ConfigError::MissingEnvVar("DATABASE_URL".to_string()))?;

        let soroban_rpc_url = env::var("SOROBAN_RPC_URL")
            .unwrap_or_else(|_| "https://soroban-testnet.stellar.org".to_string());

        let horizon_url = env::var("HORIZON_URL")
            .unwrap_or_else(|_| "https://horizon-testnet.stellar.org".to_string());

        let network_passphrase = env::var("NETWORK_PASSPHRASE")
            .unwrap_or_else(|_| "Test SDF Network ; September 2015".to_string());

        let contract_id =
            env::var("CONTRACT_ID").unwrap_or_else(|_| "STELLOVAULT_CONTRACT_ID".to_string());

        let port = env::var("PORT")
            .unwrap_or_else(|_| "3001".to_string())
            .parse::<u16>()
            .map_err(|_| ConfigError::InvalidPort("PORT must be a valid number".to_string()))?;

        let db_max_connections = env::var("DB_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "5".to_string())
            .parse::<u32>()
            .unwrap_or(5);

        let rate_limit_rps = env::var("RATE_LIMIT_RPS")
            .unwrap_or_else(|_| "100".to_string())
            .parse::<u32>()
            .unwrap_or(100);

        let webhook_secret = env::var("WEBHOOK_SECRET").ok();

        let cors_allowed_origins = env::var("CORS_ALLOWED_ORIGINS").ok();

        let log_level = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());

        // JWT and Auth configuration
        let jwt_secret = env::var("JWT_SECRET")
            .unwrap_or_else(|_| "development-secret-change-in-production".to_string());

        let jwt_access_token_ttl_seconds = env::var("JWT_ACCESS_TOKEN_TTL_SECONDS")
            .unwrap_or_else(|_| "900".to_string())
            .parse::<i64>()
            .unwrap_or(900);

        let jwt_refresh_token_ttl_days = env::var("JWT_REFRESH_TOKEN_TTL_DAYS")
            .unwrap_or_else(|_| "7".to_string())
            .parse::<i64>()
            .unwrap_or(7);

        let auth_nonce_ttl_seconds = env::var("AUTH_NONCE_TTL_SECONDS")
            .unwrap_or_else(|_| "300".to_string())
            .parse::<i64>()
            .unwrap_or(300);

        Ok(Config {
            database_url,
            soroban_rpc_url,
            horizon_url,
            network_passphrase,
            contract_id,
            environment,
            port,
            db_max_connections,
            rate_limit_rps,
            webhook_secret,
            cors_allowed_origins,
            log_level,
            jwt_secret,
            jwt_access_token_ttl_seconds,
            jwt_refresh_token_ttl_days,
            auth_nonce_ttl_seconds,
        })
    }

    /// Get database URL (useful for logging masked version)
    pub fn database_url_masked(&self) -> String {
        // Mask password in database URL for logging
        if let Some(at_pos) = self.database_url.find('@') {
            if let Some(colon_pos) = self.database_url[..at_pos].rfind(':') {
                let prefix = &self.database_url[..colon_pos + 1];
                let suffix = &self.database_url[at_pos..];
                return format!("{}****{}", prefix, suffix);
            }
        }
        self.database_url.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_from_str() {
        assert_eq!(
            Environment::from_str("dev").unwrap(),
            Environment::Development
        );
        assert_eq!(
            Environment::from_str("development").unwrap(),
            Environment::Development
        );
        assert_eq!(
            Environment::from_str("staging").unwrap(),
            Environment::Staging
        );
        assert_eq!(
            Environment::from_str("prod").unwrap(),
            Environment::Production
        );
        assert_eq!(
            Environment::from_str("production").unwrap(),
            Environment::Production
        );

        // Case insensitive
        assert_eq!(
            Environment::from_str("DEV").unwrap(),
            Environment::Development
        );
        assert_eq!(
            Environment::from_str("PROD").unwrap(),
            Environment::Production
        );

        // Invalid
        assert!(Environment::from_str("invalid").is_err());
    }

    #[test]
    fn test_environment_is_production() {
        assert!(!Environment::Development.is_production());
        assert!(!Environment::Staging.is_production());
        assert!(Environment::Production.is_production());
    }

    #[test]
    fn test_environment_as_str() {
        assert_eq!(Environment::Development.as_str(), "development");
        assert_eq!(Environment::Staging.as_str(), "staging");
        assert_eq!(Environment::Production.as_str(), "production");
    }

    #[test]
    fn test_config_database_url_masked() {
        let config = Config {
            database_url: "postgresql://user:secret_password@localhost/db".to_string(),
            soroban_rpc_url: String::new(),
            horizon_url: String::new(),
            network_passphrase: String::new(),
            contract_id: String::new(),
            environment: Environment::Development,
            port: 3001,
            db_max_connections: 5,
            rate_limit_rps: 100,
            webhook_secret: None,
            cors_allowed_origins: None,
            log_level: "info".to_string(),
            jwt_secret: "test-secret".to_string(),
            jwt_access_token_ttl_seconds: 900,
            jwt_refresh_token_ttl_days: 7,
            auth_nonce_ttl_seconds: 300,
        };

        let masked = config.database_url_masked();
        assert!(masked.contains("****"));
        assert!(!masked.contains("secret_password"));
    }

    #[test]
    fn test_config_error_types() {
        // Test that ConfigError::MissingEnvVar can be created and displayed
        let err = ConfigError::MissingEnvVar("DATABASE_URL".to_string());
        assert!(err.to_string().contains("DATABASE_URL"));

        // Test that ConfigError::InvalidPort can be created and displayed
        let err = ConfigError::InvalidPort("invalid".to_string());
        assert!(err.to_string().contains("invalid"));
    }
}

//! Database connection and pool management for StelloVault
//!
//! This module handles PostgreSQL connection pooling and migrations.

use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

use crate::config::Config;

/// Database connection error
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("Failed to connect to database: {0}")]
    ConnectionError(String),

    #[error("Failed to run migrations: {0}")]
    MigrationError(String),

    #[error("Database health check failed: {0}")]
    HealthCheckError(String),
}

/// Create a database connection pool
pub async fn create_pool(config: &Config) -> Result<PgPool, DbError> {
    tracing::info!("Connecting to database at {}", config.database_url_masked());

    let pool = PgPoolOptions::new()
        .max_connections(config.db_max_connections)
        .acquire_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(600))
        .connect(&config.database_url)
        .await
        .map_err(|e| DbError::ConnectionError(e.to_string()))?;

    tracing::info!("Database connection pool created successfully");

    Ok(pool)
}

/// Run database migrations
pub async fn run_migrations(pool: &PgPool) -> Result<(), DbError> {
    tracing::info!("Running database migrations...");

    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .map_err(|e| DbError::MigrationError(e.to_string()))?;

    tracing::info!("Database migrations completed successfully");

    Ok(())
}

/// Check database connectivity (for health checks)
pub async fn check_health(pool: &PgPool) -> Result<(), DbError> {
    sqlx::query("SELECT 1")
        .fetch_one(pool)
        .await
        .map_err(|e| DbError::HealthCheckError(e.to_string()))?;

    Ok(())
}

/// Database pool wrapper for use in application state
#[derive(Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    /// Create a new database wrapper
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get a reference to the connection pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Check database health
    pub async fn is_healthy(&self) -> bool {
        check_health(&self.pool).await.is_ok()
    }
}

impl std::ops::Deref for Database {
    type Target = PgPool;

    fn deref(&self) -> &Self::Target {
        &self.pool
    }
}

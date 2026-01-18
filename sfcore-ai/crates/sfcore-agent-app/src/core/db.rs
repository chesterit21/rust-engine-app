//! Database Connection Pool
//! 
//! Proper connection pool dengan SQLx - initialize ONCE, share everywhere.

use sqlx::{postgres::PgPoolOptions, PgPool};
use std::{env, time::Duration};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("DATABASE_URL environment variable not set")]
    MissingUrl,
    #[error("Failed to connect to database: {0}")]
    ConnectionFailed(#[from] sqlx::Error),
}

/// Create connection pool - call ONCE at app startup
pub async fn create_pool() -> Result<PgPool, DbError> {
    let database_url = env::var("DATABASE_URL").map_err(|_| DbError::MissingUrl)?;

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .min_connections(2)
        .acquire_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .connect(&database_url)
        .await?;

    Ok(pool)
}

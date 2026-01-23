use crate::config::DatabaseConfig;
use anyhow::Result;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;

#[derive(Clone)]
pub struct DbPool {
    pool: PgPool,
}

impl DbPool {
    pub async fn new(config: &DatabaseConfig) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(config.pool_max_size)
            .acquire_timeout(Duration::from_secs(config.pool_timeout_seconds))
            .connect(&config.url)
            .await?;
        
        // Test connection
        sqlx::query("SELECT 1").execute(&pool).await?;
        
        Ok(Self { pool })
    }
    
    pub fn get_pool(&self) -> &PgPool {
        &self.pool
    }
    
    pub async fn close(&self) {
        self.pool.close().await;
    }
}

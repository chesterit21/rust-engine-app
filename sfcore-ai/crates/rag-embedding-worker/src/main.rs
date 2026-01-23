use anyhow::Result;
use tracing::{info, error};

mod config;
mod database;
mod document;
mod embedding;
mod utils;
mod worker;

use config::Settings;
use database::DbPool;
use worker::Worker;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    utils::logger::init_logger()?;
    
    info!("🚀 Starting RAG Embedding Worker...");
    
    // Load configuration
    let settings = Settings::load()?;
    info!("✅ Configuration loaded");
    
    // Initialize database pool
    let db_pool = DbPool::new(&settings.database).await?;
    info!("✅ Database connection established");
    
    // Create worker instance
    let worker = Worker::new(settings, db_pool).await?;
    info!("✅ Worker initialized");
    
    // Run worker (akan block sampai error atau shutdown signal)
    match worker.run().await {
        Ok(_) => info!("Worker stopped gracefully"),
        Err(e) => error!("Worker error: {}", e),
    }
    
    Ok(())
}

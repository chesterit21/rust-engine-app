use anyhow::Result;

use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{
    fmt,
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

pub fn init_logger() -> Result<()> {
    // Get log level from environment (default: info)
    let log_level = std::env::var("RUST_LOG")
        .unwrap_or_else(|_| "info,rag_embedding_worker=debug".to_string());
    
    // Get log format from environment (default: pretty)
    let log_format = std::env::var("LOG_FORMAT")
        .unwrap_or_else(|_| "pretty".to_string());
    
    // Create file appender (logs/app.log, daily rotation)
    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("app")
        .filename_suffix("log")
        .build("logs")?;
    
    // Create filter
    let filter = EnvFilter::try_new(&log_level)?;
    
    // Setup subscriber
    match log_format.as_str() {
        "json" => {
            // JSON format untuk production
            tracing_subscriber::registry()
                .with(filter)
                .with(
                    fmt::layer()
                        .json()
                        .with_writer(std::io::stdout)
                        .with_target(true)
                        .with_level(true)
                        .with_thread_ids(true)
                )
                .with(
                    fmt::layer()
                        .json()
                        .with_writer(file_appender)
                        .with_target(true)
                        .with_level(true)
                        .with_thread_ids(true)
                )
                .init();
        }
        _ => {
            // Pretty format untuk development
            tracing_subscriber::registry()
                .with(filter)
                .with(
                    fmt::layer()
                        .pretty()
                        .with_writer(std::io::stdout)
                        .with_target(true)
                        .with_level(true)
                        .with_thread_ids(false)
                )
                .with(
                    fmt::layer()
                        .with_writer(file_appender)
                        .with_target(true)
                        .with_level(true)
                        .with_ansi(false) // No colors in file
                )
                .init();
        }
    }
    
    Ok(())
}

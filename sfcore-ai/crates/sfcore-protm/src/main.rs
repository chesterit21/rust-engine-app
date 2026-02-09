
use sfcore_protm::config::Config;
use sfcore_protm::AppState;
use sfcore_protm::handler;
use sqlx::sqlite::SqlitePoolOptions;
use std::sync::Arc;
use std::str::FromStr;
use std::time::Duration;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// mod migration; // Deleted as per user request
pub mod dto;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "sfcore_protm=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::init();

    // Database connection
    // Ensure the DB exists, otherwise CREATE it? 
    // The user said the file is at /home/sfcore/server-db/SFCoreProTM.db
    // We assume it exists.
    // Database connection
    let db_url = if config.db_path.starts_with("sqlite://") {
        config.db_path.clone()
    } else {
        format!("sqlite://{}", config.db_path)
    };

    tracing::info!("Connecting to database at: {}", db_url);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::from_str(&db_url)?
                .busy_timeout(std::time::Duration::from_secs(5))
                .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        )
        .await
        .expect("Failed to connect to database");

    // migration::fix_tech_stacks_constraint(&pool).await?;
    // migration::create_master_personas_table(&pool).await?;
    // migration::upgrade_architecture_patterns_v2(&pool).await?;

    let state = Arc::new(AppState { db: pool });

    // Build router
    let app = handler::create_router(state);

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

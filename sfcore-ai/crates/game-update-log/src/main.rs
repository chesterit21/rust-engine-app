use anyhow::Result;
use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use game_update_log::App;

#[derive(Parser)]
#[command(name = "game-update-log")]
#[command(about = "Game result scraper and logger", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the main update loop
    Run,
    
    /// Correct missing logs
    Correct,
    
    /// Validate last logs
    Validate,
    
    /// Run a single update cycle (for testing)
    Update,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenv().ok();

    // Setup tracing
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    let formatting_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_level(true)
        .with_thread_ids(false)
        .with_thread_names(false);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(formatting_layer)
        .init();

    // Get database URL from environment
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env file");

    // Create app instance
    let app = App::new(&database_url).await?;

    // Parse CLI arguments
    let cli = Cli::parse();

    // Execute command
    match cli.command {
        Commands::Run => {
            tracing::info!("Starting game update log application");
            app.run().await?;
        }
        Commands::Correct => {
            tracing::info!("Running log correction");
            app.correct_logs().await?;
        }
        Commands::Validate => {
            tracing::info!("Validating last logs");
            app.validate_last_log_games().await?;
        }
        Commands::Update => {
            tracing::info!("Running single update cycle");
            app.start_update_log().await?;
        }
    }

    Ok(())
}
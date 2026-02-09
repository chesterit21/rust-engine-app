use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod routes;
mod handlers;
mod services;
mod dtos;
mod state;
mod errors;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env file
    dotenvy::dotenv().ok();

    // Setup tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "game_matrix=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("🚀 Starting Game Matrix API Server...");

    // Load DB path from env (default: games_matrix.db)
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "games_matrix.db".to_string());

    tracing::info!("🗄️  Connecting to SQLite DB: {}", database_url);

    // Init app state
    let state = state::AppState::new(&database_url)
        .await
        .expect("Failed to initialize app state");

    // Ensure indexes exist (Batch 2 Requirement)
    state.repositories.create_indexes(&state.db_pool)
        .await
        .expect("Failed to create database indexes");

    let app_state = Arc::new(state);

    // Build router
    let app = routes::create_router(app_state.clone())
        .layer(
            CorsLayer::new()
                .allow_origin(tower_http::cors::Any)
                .allow_methods(tower_http::cors::Any)
                .allow_headers(tower_http::cors::Any),
        );

    // Bind to address
    let addr = SocketAddr::from(([0, 0, 0, 0], 3001));
    tracing::info!("🚀 Server running on http://{}", addr);

    // Start server using tokio::net::TcpListener (axum 0.8+ API)
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
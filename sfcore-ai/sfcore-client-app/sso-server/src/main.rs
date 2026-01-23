use axum::{routing::get, routing::post, Router};
use std::net::SocketAddr;
use tower_http::{cors::CorsLayer, services::{ServeDir, ServeFile}};
use tracing::{info, error};
use std::sync::Arc;

use sso_api::{handlers::{auth, health}, state::AppState};
use sso_shared::config::AppConfig;
use sso_infrastructure::database::connection;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env
    dotenvy::dotenv().ok();

    // Initialize telemetry
    sso_shared::telemetry::init_telemetry();
    
    info!("SSO Server starting...");

    // Load configuration
    let config = match AppConfig::load() {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            std::process::exit(1);
        }
    };

    // Connect to Database
    info!("Connecting to database at {}...", config.database.url);
    let pool = connection::create_pool(&config.database.url, config.database.max_connections).await?;
    info!("Database connection established.");

    // Create App State
    let state = AppState {
        db: pool,
        config: config.clone(),
    };

    // Build router
    let app = Router::new()
        // Health check
        .route("/health", get(health::health_check))
        // Static Assets
        .nest_service("/assets", ServeDir::new("static/assets"))
        // serve login page
        .route_service("/login", ServeFile::new("static/login.html"))
        // Auth routes
        .route("/api/v1/auth/login", post(auth::login))
        .route("/api/v1/auth/logout", post(auth::logout))
        // Add State
        .with_state(state)
        // Add CORS
        .layer(
            CorsLayer::new()
                .allow_origin("http://localhost:5173".parse::<axum::http::HeaderValue>().unwrap())
                .allow_methods([
                    axum::http::Method::GET,
                    axum::http::Method::POST,
                    axum::http::Method::PUT,
                    axum::http::Method::DELETE,
                    axum::http::Method::OPTIONS,
                ])
                .allow_headers([axum::http::header::CONTENT_TYPE, axum::http::header::AUTHORIZATION]),
        );

    // Bind address
    let host: std::net::IpAddr = config.app.host.parse()?;
    let addr = SocketAddr::from((host, config.app.port));
    info!("Listening on {}", addr);

    // Start server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

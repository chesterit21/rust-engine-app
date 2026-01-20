//! SSO Server - Main Application Entry Point

use axum::{routing::get, routing::post, Router};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tracing::info;

use sso_api::handlers::{auth, health};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize telemetry
    sso_shared::telemetry::init_telemetry();
    
    info!("SSO Server starting...");

    // Build router
    let app = Router::new()
        // Health check
        .route("/health", get(health::health_check))
        // Auth routes
        .route("/api/v1/auth/login", post(auth::login))
        .route("/api/v1/auth/logout", post(auth::logout))
        // Add CORS
        .layer(CorsLayer::permissive());

    // Bind address
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    info!("Listening on {}", addr);

    // Start server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

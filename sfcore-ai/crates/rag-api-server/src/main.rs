use anyhow::Result;
use axum::{
    extract::DefaultBodyLimit,
    middleware,
    routing::{get, post},
    Extension, Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::{
    cors::CorsLayer,
    trace::{DefaultMakeSpan, TraceLayer},
};
use tracing::info;
use services::DocumentService;

mod config;
mod database;
mod handlers;
mod security;
mod services;
mod utils;
mod document;
mod models;

use config::Settings;
use database::{DbPool, Repository};
use security::{CustomHeaderValidator, DocumentAuthorization, IpWhitelist};
use services::{EmbeddingService, LlmService, RagService};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "info,rag_api_server=debug".to_string()),
        )
        .with_target(true)
        .with_thread_ids(true)
        .json()
        .init();
    
    info!("🚀 Starting RAG API Server...");
    
    // Load configuration
    let settings = Settings::load()?;
    info!("✅ Configuration loaded");
    
    // Initialize database pool
    let db_pool = DbPool::new(&settings.database).await?;
    info!("✅ Database connection established");
    
    // Initialize repository
    let repository = Arc::new(Repository::new(db_pool));
    
    // Initialize services
    let embedding_service = Arc::new(EmbeddingService::new(
        settings.llm.base_url.clone(),
        settings.embedding.clone(),
    ));
    
    let document_service = Arc::new(DocumentService::new(
        repository.clone(),
        embedding_service.clone(),
    ));

    let llm_service = Arc::new(LlmService::new(settings.llm.clone()));
    
    let rag_service = Arc::new(RagService::new(
        repository.clone(),
        embedding_service.clone(),
        llm_service.clone(),
        settings.rag.clone(),
    ));
    
    // Initialize security
    let ip_whitelist = Arc::new(IpWhitelist::new(
        settings.config_path(),
        settings.security.allowed_ips.clone(),
    )?);
    
    // Start file watcher untuk hot-reload IP whitelist
    // Clone inner value because start_watcher takes ownership of self (not Arc)
    (*ip_whitelist).clone().start_watcher()?;
    info!("✅ IP whitelist watcher started");
    
    let header_validator = Arc::new(CustomHeaderValidator::new(
        settings.security.custom_headers.app_id.clone(),
        settings.security.custom_headers.api_key.clone(),
        settings.security.custom_headers.request_signature == "enabled",
        settings.security.custom_headers.timestamp_tolerance,
    ));
    
    let document_auth = Arc::new(DocumentAuthorization::new(repository.clone()));
    
    // Build router
    let app = build_router(
        rag_service,
        embedding_service,
        document_auth,
        document_service,
        ip_whitelist,
        header_validator,
    );
    
    // Server address
    let addr = SocketAddr::from((
        settings.server.host.parse::<std::net::IpAddr>()?,
        settings.server.port,
    ));
    
    info!("🎯 Server listening on {}", addr);
    
    // Start server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;
    
    Ok(())
}

fn build_router(
    rag_service: Arc<RagService>,
    embedding_service: Arc<EmbeddingService>,
    document_auth: Arc<DocumentAuthorization>,
    document_service: Arc<DocumentService>, 
    ip_whitelist: Arc<IpWhitelist>,
    header_validator: Arc<CustomHeaderValidator>,
) -> Router {
    // Public routes (no security)
    let public_routes = Router::new()
        .route("/health", get(handlers::health::health_check))
        .route("/health/ready", get(handlers::health::readiness_check));
    
    // Protected routes (dengan security middleware)
    let protected_routes = Router::new()
        // .route("/api/chat", post(handlers::chat::chat_handler))
        .route("/api/chat/stream", post(handlers::chat::chat_stream_handler))
        .route("/api/search", post(handlers::search::search_handler))
        .route("/api/upload", post(handlers::upload::upload_handler))
        .route("/api/documents", get(handlers::search::list_documents_handler))
        .layer(middleware::from_fn(security::middleware::security_middleware))
        .layer(Extension(rag_service.clone()))      // CLONE HERE
        .layer(Extension(embedding_service.clone())) // CLONE HERE
        .layer(Extension(ip_whitelist))
        .layer(Extension(header_validator))
        .layer(Extension(document_service)) 
        .layer(Extension(document_auth.clone()));
    
    // Combine routes
    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        // Shared state
        .layer(Extension(rag_service))
        .layer(Extension(embedding_service))
        .layer(Extension(document_auth))
        // CORS
        .layer(
            CorsLayer::permissive()
                .allow_origin(tower_http::cors::Any)
                .allow_methods(tower_http::cors::Any)
                .allow_headers(tower_http::cors::Any),
        )
        // Tracing
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        )
        // Body limit (untuk upload - max 100MB)
        .layer(DefaultBodyLimit::max(100 * 1024 * 1024))
}

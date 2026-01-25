use anyhow::Result;
use axum::{
    extract::{DefaultBodyLimit, FromRef},
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

use rag_api_server::{
    config, database, document, handlers, models, security, services, state, utils,
};

use config::Settings;
use database::{DbPool, Repository};
use security::{CustomHeaderValidator, DocumentAuthorization, IpWhitelist};
use services::{
    conversation::ConversationManager,
    EmbeddingService, LlmService, RagService, DocumentService
};
use state::AppState;

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
    
    info!("🚀 Starting RAG API Server with Conversation Memory...");
    
    // Load configuration
    let settings = Settings::load()?;
    info!("✅ Configuration loaded");
    
    // Initialize database pool
    let db_pool = DbPool::new(&settings.database).await?;
    info!("✅ Database connection established");
    
    // Initialize repository
    let repository = Arc::new(Repository::new(db_pool.clone()));
    
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
    
    // Initialize conversation manager
    // We need to pass Boxed trait objects. 
    // Since services are wrapped in Arc, we dereference and clone the inner struct (which must derive Clone).
    // EmbeddingService, RagService, LlmService all derive Clone.
    let conversation_manager = Arc::new(ConversationManager::new(
        Box::new((*embedding_service).clone()),
        Box::new((*rag_service).clone()),
        Box::new((*llm_service).clone()),
    ));
    info!("✅ Conversation manager initialized");
    
    // Initialize security
    let ip_whitelist = Arc::new(IpWhitelist::new(
        settings.config_path(),
        settings.security.allowed_ips.clone(),
    )?);
    
    // Start file watcher untuk hot-reload IP whitelist
    (*ip_whitelist).clone().start_watcher()?;
    info!("✅ IP whitelist watcher started");
    
    let header_validator = Arc::new(CustomHeaderValidator::new(
        settings.security.custom_headers.app_id.clone(),
        settings.security.custom_headers.api_key.clone(),
        settings.security.custom_headers.request_signature == "enabled",
        settings.security.custom_headers.timestamp_tolerance,
    ));
    
    let document_auth = Arc::new(DocumentAuthorization::new(repository.clone()));
    
    // Build application state
    let app_state = AppState {
        db_pool: db_pool.clone(),
        embedding_service: embedding_service.clone(),
        rag_service: rag_service.clone(),
        llm_service: llm_service.clone(),
        conversation_manager: conversation_manager.clone(),
        settings: settings.clone(),
        document_service: document_service.clone(),
        document_auth: document_auth.clone(),
        ip_whitelist: ip_whitelist.clone(),
        header_validator: header_validator.clone(),
    };

    // Build router
    let app = build_router(Arc::new(app_state));
    
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

fn build_router(app_state: Arc<AppState>) -> Router {
    // Extract services for Extension injection (legacy support)
    // We need clones for the extensions
    let rag_service = app_state.rag_service.clone();
    let embedding_service = app_state.embedding_service.clone();
    let document_auth = app_state.document_auth.clone();
    let document_service = app_state.document_service.clone();
    let ip_whitelist = app_state.ip_whitelist.clone();
    let header_validator = app_state.header_validator.clone();
    let repository = Arc::new(Repository::new(app_state.db_pool.clone())); // Re-create repo wrapper or store in state? Store logic implies repo was made earlier.
    // Repo is used by upload handler via Extension. 
    // DocumentService uses repo.
    // Handlers use: Extension(Arc<Repository>), Extension(Arc<EmbeddingService>), Extension(Arc<DocumentAuthorization>)
    // So we must provide these.

    // Public routes (no security)
    let public_routes = Router::new()
        .route("/health", get(handlers::health::health_check))
        .route("/health/ready", get(handlers::health::readiness_check));
    
    // Protected routes (dengan security middleware)
    let protected_routes = Router::new()
        // Chat Endpoints
        .route("/api/chat/stream", post(handlers::chat::chat_stream_handler))
        .route("/api/chat/session/new", post(handlers::chat::new_session_handler))
        .route("/api/chat/stats", get(handlers::chat::cache_stats_handler))
        .route("/api/chat/cleanup", post(handlers::chat::cleanup_sessions_handler))
        
        // Existing Endpoints
        .route("/api/search", post(handlers::search::search_handler))
        .route("/api/upload", post(handlers::upload::upload_handler))
        .route("/api/documents", get(handlers::search::list_documents_handler))
        
        .layer(middleware::from_fn(security::middleware::security_middleware))
        
        // Inject State (for new handlers)
        // State is injected via .with_state() on the whole router or merge.
        // But duplicate injection is tricky?
        // Actually, if we use .with_state(app_state), methods using State<AppState> will work.
        // Methods using Extension will ignore State and look for Extension.
        // We must provide Extension layers for them.
        
        .layer(Extension(rag_service))
        .layer(Extension(embedding_service))
        .layer(Extension(ip_whitelist))
        .layer(Extension(header_validator))
        .layer(Extension(document_service)) 
        .layer(Extension(document_auth))
        .layer(Extension(repository)); // For upload handler
    
    // Combine routes
    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .with_state(app_state) // Provide State for all routes
        .layer(
            CorsLayer::permissive()
                .allow_origin(tower_http::cors::Any)
                .allow_methods(tower_http::cors::Any)
                .allow_headers(tower_http::cors::Any),
        )
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        )
        .layer(DefaultBodyLimit::max(100 * 1024 * 1024))
}

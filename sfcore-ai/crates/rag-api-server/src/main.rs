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
    logging,
};

use config::Settings;
use database::{DbPool, Repository};
use security::{CustomHeaderValidator, DocumentAuthorization, IpWhitelist};
use services::{
    conversation::ConversationManager,
    EmbeddingService, LlmService, RagService, DocumentService, EventBus
};
use state::AppState;
use logging::{ActivityLogger, LoggerConfig};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    // Initialize logging with non-blocking file writer to prevent console freeze
    // 1. File Appender (Daily rolling, logs/ directory)
    let file_appender = tracing_appender::rolling::daily("logs", "rag-server.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // 2. Archive stdout (optional, minimal output to console)
    // We filter stdout to WARN only to prevent "QuickEdit" freeze on Windows
    let stdout_log = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_filter(tracing_subscriber::filter::LevelFilter::WARN);

    // 3. File Log (Full Debug/Info)
    let file_log = tracing_subscriber::fmt::layer()
        .with_writer(non_blocking)
        .with_ansi(false)
        .json()
        .with_target(true)
        .with_thread_ids(true);

    // 4. Registry
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
    use tracing_subscriber::Layer; // Import Layer trait

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,rag_api_server=debug".into())
        )
        .with(stdout_log)
        .with(file_log)
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
    repository.ensure_processing_table().await?;
    repository.ensure_indices().await?;
    info!("✅ Repository and tables initialized");
    
    // Ensure chat history tables (NEW)
    repository.ensure_chat_history_tables().await?;
    info!("✅ Chat history tables ensured");

    // Initialize Activity Logger
    let logger = ActivityLogger::new(
        db_pool.get_pool().clone(),
        LoggerConfig::default(),
    );
    info!("✅ Activity logger initialized");
    
    // Initialize Limiters
    // Initialize concurrency limiters
    let limiters = Arc::new(utils::limiters::Limiters::new(&settings.limits));
    info!("✅ Global concurrency limiters initialized");

    // OVERRIDE: Check for Gemini Configuration
    let mut final_embedding_config = settings.embedding.clone();
    let mut final_llm_config = settings.llm.clone();

    if let Some(gemini) = &settings.gemini {
        if gemini.enabled {
            info!("♊ Gemini Configuration Detected & ENABLED! Overriding LLM and Embedding settings.");
            
            // Override Embedding Config for Gemini
        final_embedding_config.base_url = gemini.base_url.clone().unwrap_or_else(|| "https://generativelanguage.googleapis.com/v1beta/openai".to_string());
        final_embedding_config.model = gemini.embedding_model.clone().unwrap_or_else(|| "text-embedding-004".to_string());
        final_embedding_config.dimension = 768; // Standard for text-embedding-004
        final_embedding_config.api_key = Some(gemini.api_key.clone());
        
        // Override LLM Config for Gemini
        final_llm_config.base_url = gemini.base_url.clone().unwrap_or_else(|| "https://generativelanguage.googleapis.com/v1beta/openai".to_string());
        final_llm_config.api_key = Some(gemini.api_key.clone());
        final_llm_config.max_tokens = 8192; // Default for flash
        final_llm_config.model = Some(gemini.model.clone().unwrap_or_else(|| "gemini-1.5-flash".to_string()));
        }
    }

    // Initialize services
    let embedding_service = Arc::new(EmbeddingService::new(
        final_embedding_config.base_url.clone(),
        final_embedding_config.clone(),
        limiters.clone(),
        settings.limits.embedding_batch_size,
    ));

    let llm_service = Arc::new(LlmService::new(
        final_llm_config.clone(),
        settings.prompts.context_extraction_system_prompt.clone(),
        limiters.clone(),
    ));

    let document_service = Arc::new(DocumentService::new(
        repository.clone(),
        embedding_service.clone(),
        llm_service.clone(),
        &settings.rag,
        &settings.limits, // NEW
    ));
    
    let rag_service = Arc::new(RagService::new(
        repository.clone(),
        embedding_service.clone(),
        llm_service.clone(),
        settings.rag.clone(),
        limiters.clone(),
    ));
    
    // Initialize conversation manager
    let conversation_manager = Arc::new(ConversationManager::new(
        Box::new((*embedding_service).clone()),
        Box::new((*rag_service).clone()),
        Box::new((*llm_service).clone()),
        logger.clone(),
        settings.llm.stream_response,
        settings.prompts.main_system_prompt.clone(),
        settings.prompts.deep_scan_system_prompt.clone(),
        settings.rag.clone(),
    ));
    info!("✅ Conversation manager initialized");

    // Initialize EventBus
    let event_bus = Arc::new(EventBus::new(4096)); // Increased capacity to prevent dropped events
    info!("✅ EventBus initialized");
    
    // Initialize security
    let ip_whitelist = Arc::new(IpWhitelist::new(
        settings.config_path(),
        settings.security.allowed_ips.clone(),
    )?);
    
    // Start file watcher
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
        conversation_manager,
        settings: settings.clone(),
        document_service,
        document_auth,
        ip_whitelist,
        header_validator,
        event_bus,
        limiters, // NEW
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
    let rag_service = app_state.rag_service.clone();
    let embedding_service = app_state.embedding_service.clone();
    let document_auth = app_state.document_auth.clone();
    let document_service = app_state.document_service.clone();
    let ip_whitelist = app_state.ip_whitelist.clone();
    let header_validator = app_state.header_validator.clone();
    let repository = Arc::new(Repository::new(app_state.db_pool.clone()));

    // Public routes
    let public_routes = Router::new()
        .route("/health", get(handlers::health::health_check))
        .route("/health/ready", get(handlers::health::readiness_check));
    
    // Protected routes
    let mut protected_routes = Router::new()
        // Chat Endpoints
        .route("/api/chat/session/new", post(handlers::chat::new_session_handler))
        .route("/api/chat/init", post(handlers::chat::init_handler))
        .route("/api/chat/events", get(handlers::chat::events_handler))
        .route("/api/chat/stats", get(handlers::chat::cache_stats_handler))
        .route("/api/chat/logs/stats", get(handlers::chat::logger_stats_handler))
        .route("/api/chat/cleanup", post(handlers::chat::cleanup_sessions_handler))
        
        // Existing Endpoints
        .route("/api/search", post(handlers::search::search_handler))
        .route("/api/documents", post(handlers::search::list_documents_handler));

    // ==================================================================================
    // 🔀 ROUTING SWITCH (Toggle Mode)
    // Controlled by [gemini] enabled = true/false in settings.toml
    // ==================================================================================

    let mut use_gemini = false;
    if let Some(gemini) = &app_state.settings.gemini {
        if gemini.enabled {
             use_gemini = true;
        }
    }

    if use_gemini {
        // Mode 1: GEMINI
        info!("🔀 Mode: GEMINI (Enabled via settings)");
        protected_routes = route_for_gemini(protected_routes);
    } else {
        // Mode 2: LEGACY (Ollama/Standard)
        info!("🔀 Mode: LEGACY/OLLAMA (Default/Gemini Disabled)");
        protected_routes = route_for_legacy(protected_routes);
    }

    // FOR MANUAL FORCE (Example):
    // protected_routes = route_for_legacy(protected_routes); // Uncomment this to force Legacy
    // protected_routes = route_for_gemini(protected_routes); // Uncomment this to force Gemini

/// ♊ Configure Routes for GEMINI (Upload -> Gemini Handler)
fn route_for_gemini(router: Router<Arc<AppState>>) -> Router<Arc<AppState>> {
    use handlers::{gemini, chat};
    router
        .route("/api/upload", post(gemini::upload_handler_gemini))
        // Note: Chat stream uses standard handler but with Gemini-configured LLM Service
        .route("/api/chat/stream", post(chat::chat_stream_handler))
}

/// 🦙 Configure Routes for LEGACY/OLLAMA (Upload -> Standard Handler)
fn route_for_legacy(router: Router<Arc<AppState>>) -> Router<Arc<AppState>> {
    use handlers::{upload, chat};
    router
        .route("/api/upload", post(upload::upload_handler))
        .route("/api/chat/stream", post(chat::chat_stream_handler))
}

    protected_routes = protected_routes
        .layer(middleware::from_fn(security::middleware::security_middleware))
        
        // Inject Extensions
        .layer(Extension(rag_service))
        .layer(Extension(embedding_service))
        .layer(Extension(ip_whitelist))
        .layer(Extension(header_validator))
        .layer(Extension(document_service)) 
        .layer(Extension(document_auth))
        .layer(Extension(repository));
    
    // Combine routes
    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .with_state(app_state)
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
        .layer(tower_http::catch_panic::CatchPanicLayer::new())
        .layer(DefaultBodyLimit::max(100 * 1024 * 1024))
}

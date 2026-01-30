use anyhow::{Context, Result};
use clap::Parser;
use log::{error, info, warn};
use sfcore_ai_engine::{LlamaCppEngine, LlamaCppOptions};
use std::sync::Arc;
use tokio::net::UnixListener;
mod tcp_handler;

mod handler;
mod config;
mod error;
mod auth;
mod http_handler;

use config::Config;

#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[derive(Parser, Debug)]
#[command(name = "sfcore-ai-server", version, about = "SFCore AI Inference Server")]
struct Args {
    /// Path to GGUF model file (overrides config)
    #[arg(long)]
    model: Option<String>,
    
    /// Transport mode: uds, http-sse, tcp (overrides config)
    #[arg(long)]
    transport: Option<String>,
    
    // === Context & Memory ===
    /// Context size (-c, --ctx-size)
    #[arg(long, alias = "ctx-size")]
    context_length: Option<u32>,
    
    /// Logical batch size for prompt processing
    #[arg(long)]
    batch_size: Option<usize>,
    
    /// Physical batch size per step
    #[arg(long)]
    ubatch_size: Option<usize>,
    
    // === Threading ===
    /// Number of threads for decoding (generation)
    #[arg(long)]
    threads: Option<u32>,
    
    /// Number of threads for batch processing
    #[arg(long)]
    threads_batch: Option<u32>,
    
    // === Parallel Processing ===
    /// Number of parallel sequences
    #[arg(long)]
    parallel: Option<u32>,
    
    // === Caching & Memory ===
    /// Disable prompt caching (saves memory)
    #[arg(long)]
    no_cache_prompt: bool,
    
    /// Lock model in RAM (prevent swapping)
    #[arg(long)]
    mlock: Option<bool>,
    
    // === Token Management ===
    /// Number of tokens to keep from initial prompt (-1 = all, 0 = none)
    #[arg(long)]
    keep: Option<i32>,
    
    // === Sampling Parameters ===
    /// Random seed for reproducibility
    #[arg(long)]
    seed: Option<u32>,
    
    /// Sampling temperature (0.0 = greedy, >1.0 = creative)
    #[arg(long)]
    temperature: Option<f32>,
    
    /// Top-K sampling
    #[arg(long)]
    top_k: Option<i32>,
    
    /// Top-P (nucleus) sampling
    #[arg(long)]
    top_p: Option<f32>,
    
    /// Min-P sampling
    #[arg(long)]
    min_p: Option<f32>,
    
    // === Penalties ===
    /// Repetition penalty (1.0 = disabled)
    #[arg(long)]
    repeat_penalty: Option<f32>,
    
    /// Last N tokens to consider for repetition penalty
    #[arg(long)]
    repeat_last_n: Option<i32>,
    
    /// Frequency penalty (additive)
    #[arg(long)]
    frequency_penalty: Option<f32>,
    
    /// Presence penalty (additive)
    #[arg(long)]
    presence_penalty: Option<f32>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Force single threaded BLAS
    unsafe {
        std::env::set_var("OPENBLAS_NUM_THREADS", "1");
        std::env::set_var("MKL_NUM_THREADS", "1");
    }
    
    env_logger::init();
    
    // Load config
    let config_path = "server_config.toml";
    let config = Config::from_file(config_path)?;
    let args = Args::parse();
    
    info!("SFCore AI Server starting...");
    
    // Resolve model path
    let model_path = args.model
        .or(config.model.clone())
        .ok_or_else(|| anyhow::anyhow!("Model path required (--model or config)"))?;
    
    // Resolve transport
    let transport = args.transport
        .or(config.transport.clone())
        .unwrap_or_else(|| "uds".to_string());
    
    info!("Model: {}", model_path);
    info!("Transport: {}", transport);
    
    // Build engine options with precedence: CLI > Config > Defaults
    let engine_cfg = config.engine.as_ref();
    
    let opts = LlamaCppOptions {
        // Context & Memory
        context_length: args.context_length
            .or(engine_cfg.and_then(|e| e.context_length))
            .unwrap_or(4096),
        
        batch_size: args.batch_size
            .or(engine_cfg.and_then(|e| e.batch_size))
            .unwrap_or(2048),
        
        ubatch_size: args.ubatch_size
            .or(engine_cfg.and_then(|e| e.ubatch_size))
            .unwrap_or(1024),
        
        // Threading
        threads: Some(
            args.threads
                .or(engine_cfg.and_then(|e| e.threads))
                .unwrap_or(4) as i32
        ),
        
        threads_batch: Some(
            args.threads_batch
                .or(engine_cfg.and_then(|e| e.threads_batch))
                .unwrap_or(4) as i32
        ),
        
        // Memory
        use_mlock: args.mlock
            .or(engine_cfg.and_then(|e| e.mlock))
            .unwrap_or(true),
        
        no_cache_prompt: args.no_cache_prompt
            || engine_cfg.and_then(|e| e.no_cache_prompt).unwrap_or(false),
        
        // Sampling
        seed: args.seed
            .or(engine_cfg.and_then(|e| e.seed))
            .unwrap_or(1234),
        
        temperature: args.temperature
            .or(engine_cfg.and_then(|e| e.temperature))
            .unwrap_or(0.5),
        
        top_k: args.top_k
            .or(engine_cfg.and_then(|e| e.top_k))
            .unwrap_or(40),
        
        top_p: args.top_p
            .or(engine_cfg.and_then(|e| e.top_p))
            .unwrap_or(0.9),
        
        min_p: args.min_p
            .or(engine_cfg.and_then(|e| e.min_p))
            .unwrap_or(0.05),
        
        // Penalties
        repeat_penalty: args.repeat_penalty
            .or(engine_cfg.and_then(|e| e.repeat_penalty))
            .unwrap_or(1.0),
        
        repeat_last_n: args.repeat_last_n
            .or(engine_cfg.and_then(|e| e.repeat_last_n))
            .unwrap_or(64),
        
        frequency_penalty: args.frequency_penalty
            .or(engine_cfg.and_then(|e| e.frequency_penalty))
            .unwrap_or(0.0),
        
        presence_penalty: args.presence_penalty
            .or(engine_cfg.and_then(|e| e.presence_penalty))
            .unwrap_or(0.0),
    };
    
    // Log active configuration
    info!("=== Engine Configuration ===");
    info!("Context Length: {}", opts.context_length);
    info!("Batch Size: {}", opts.batch_size);
    info!("Ubatch Size: {}", opts.ubatch_size);
    info!("Threads: {:?}", opts.threads);
    info!("Threads Batch: {:?}", opts.threads_batch);
    info!("Use mlock: {}", opts.use_mlock);
    info!("No cache prompt: {}", opts.no_cache_prompt); // <-- NEW
    info!("Temperature: {}", opts.temperature);
    info!("Top-K: {}, Top-P: {}, Min-P: {}", opts.top_k, opts.top_p, opts.min_p);
    info!("===========================");
    
    // Initialize engine
    let mut engine = LlamaCppEngine::new(opts)?;
    engine.load_gguf(&model_path)?;
    let engine = Arc::new(engine);
    
    // Dispatch based on transport
    match transport.as_str() {
        "uds" => run_uds_server(engine, &config).await,
        "http-sse" => run_http_server(engine, &config).await,
        "tcp" => run_tcp_server(engine, &config).await,
        _ => Err(anyhow::anyhow!("Unknown transport: {}", transport)),
    }
}

/// Run UDS server (existing)
async fn run_uds_server(engine: Arc<LlamaCppEngine>, config: &Config) -> Result<()> {
    let socket_path = config
        .uds
        .as_ref()
        .map(|u| u.socket.clone())
        .unwrap_or_else(|| "/tmp/sfcore-ai.sock".to_string());
    
    if std::path::Path::new(&socket_path).exists() {
        if let Err(e) = std::fs::remove_file(&socket_path) {
            warn!("Failed to remove existing socket: {}", e);
        }
    }
    
    let listener = UnixListener::bind(&socket_path).context("Failed to bind UDS")?;
    info!("UDS listening on {}", socket_path);
    
    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                let engine_ref = engine.clone();
                tokio::spawn(async move {
                    if let Err(e) = handler::handle_connection(stream, engine_ref).await {
                        error!("Connection error: {}", e);
                    }
                });
            }
            Err(e) => {
                error!("Accept failed: {}", e);
            }
        }
    }
}

/// Run HTTP-SSE server
async fn run_http_server(engine: Arc<LlamaCppEngine>, config: &Config) -> Result<()> {
    use axum::{
        routing::{get, post},
        Router,
        middleware,
    };
    use tower_http::trace::TraceLayer;
    
    let http_config = config
        .http
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Missing [http] config"))?;
    
    let auth_config = config
        .auth
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Missing [auth] config for HTTP transport"))?;
    
    let rate_limit_config = config
        .rate_limit
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Missing [rate_limit] config"))?;
    
    // Create shared states
    let app_state = http_handler::AppState {
        engine: engine.clone(),
    };
    
    let auth_state = auth::AuthState::new(auth_config, rate_limit_config.requests_per_minute);
    
    // Build router
    let app = Router::new()
        .route("/v1/inference", post(http_handler::inference_handler))
        .route("/health", get(http_handler::health_handler))
        .layer(middleware::from_fn_with_state(
            auth_state.clone(),
            auth::auth_middleware,
        ))
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);
    
    let addr = format!("{}:{}", http_config.host, http_config.port);
    info!("HTTP-SSE listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}

/// Run TCP server (Windows)
/// Run TCP server (cross-platform: Linux, Windows, macOS, Android)
async fn run_tcp_server(engine: Arc<LlamaCppEngine>, config: &Config) -> Result<()> {
    let tcp_config = config
        .tcp
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Missing [tcp] config"))?;
    
    let addr = format!("{}:{}", tcp_config.host, tcp_config.port);
    
    // Security warning for 0.0.0.0 binding
    if tcp_config.host == "0.0.0.0" {
        warn!("⚠️  TCP server binding to 0.0.0.0 (all interfaces)");
        warn!("⚠️  This exposes inference to network without encryption!");
        warn!("⚠️  Recommended: Use SSH tunnel or bind to 127.0.0.1 for local only");
    }
    
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .with_context(|| format!("Failed to bind TCP socket: {}", addr))?;
    
    info!("TCP listening on {} (cross-platform mode)", addr);
    info!("Compatible with: Linux, Windows, macOS, Android, iOS, etc.");
    
    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                info!("Accepted connection from: {}", addr);
                let engine_ref = engine.clone();
                tokio::spawn(async move {
                    if let Err(e) = tcp_handler::handle_connection(stream, engine_ref).await {
                        error!("[{}] Connection error: {}", addr, e);
                    }
                });
            }
            Err(e) => {
                error!("Accept failed: {}", e);
            }
        }
    }
}

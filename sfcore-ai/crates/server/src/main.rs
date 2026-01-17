use anyhow::{Context, Result};
use clap::Parser;
use log::{error, info, warn};
use serde::Deserialize;
use sfcore_ai_engine::{LlamaCppEngine, LlamaCppOptions};
use std::fs;
use std::sync::Arc;
use tokio::net::UnixListener;

mod handler;

#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

/// Configuration file structure (config.toml)
#[derive(Debug, Deserialize)]
struct Config {
    model: Option<String>,
    socket: Option<String>,
    threads: Option<i32>,
    threads_batch: Option<i32>,
    context_length: Option<u32>,
    batch_size: Option<usize>,
    ubatch_size: Option<usize>,
    mlock: Option<bool>,
}

#[derive(Parser, Debug)]
#[command(name = "sfcore-ai-server", version)]
struct Args {
    /// Path to GGUF model file (overrides config)
    #[arg(long)]
    model: Option<String>,

    /// Unix Domain Socket path (overrides config)
    #[arg(long)]
    socket: Option<String>,

    // === Engine Options ===
    #[arg(long)]
    threads: Option<i32>,

    #[arg(long)]
    threads_batch: Option<i32>,

    #[arg(long)]
    context_length: Option<u32>,

    #[arg(long)]
    batch_size: Option<usize>,

    #[arg(long)]
    ubatch_size: Option<usize>,

    #[arg(long)]
    mlock: Option<bool>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Force single threaded BLAS to avoid contention
    std::env::set_var("OPENBLAS_NUM_THREADS", "1");
    std::env::set_var("MKL_NUM_THREADS", "1");

    env_logger::init();
    
    // 1. Load Config File (if exists)
    let config_path = "server_config.toml";
    let config: Config = if std::path::Path::new(config_path).exists() {
        let content = fs::read_to_string(config_path).context("Failed to read config file")?;
        toml::from_str(&content).context("Failed to parse config file")?
    } else {
        Config {
            model: None,
            socket: None, // Default fallback later
            threads: None,
            threads_batch: None,
            context_length: None,
            batch_size: None,
            ubatch_size: None,
            mlock: None,
        }
    };

    let args = Args::parse();
    
    info!("SFCore AI Server starting...");

    // 2. Resolve Parameters (Args > Config > Defaults)
    let model_path = args.model
        .or(config.model.clone())
        .ok_or_else(|| anyhow::anyhow!("Model path must be provided via --model or server_config.toml"))?;
        
    let socket_path = args.socket
        .or(config.socket.clone())
        .unwrap_or_else(|| "/tmp/sfcore-ai.sock".to_string());
        
    let threads = args.threads.or(config.threads).unwrap_or(4);
    let threads_batch = args.threads_batch.or(config.threads_batch).unwrap_or(4);
    let context_length = args.context_length.or(config.context_length).unwrap_or(4096);
    let batch_size = args.batch_size.or(config.batch_size).unwrap_or(2048);
    let ubatch_size = args.ubatch_size.or(config.ubatch_size).unwrap_or(1024);
    let mlock = args.mlock.or(config.mlock).unwrap_or(true);

    info!("Model: {}", model_path);
    info!("Socket: {}", socket_path);

    // 3. Initialize Engine
    let opts = LlamaCppOptions {
        threads: Some(threads),
        threads_batch: Some(threads_batch),
        context_length,
        batch_size,
        ubatch_size,
        use_mlock: mlock,
        ..Default::default()
    };

    let mut engine = LlamaCppEngine::new(opts)?;
    engine.load_gguf(&model_path)?;
    
    // Wrap in Arc for shared access across tasks
    let engine = Arc::new(engine);

    // 4. Bind UDS Listener
    if std::path::Path::new(&socket_path).exists() {
        if let Err(e) = std::fs::remove_file(&socket_path) {
            warn!("Failed to remove existing socket: {}", e);
        }
    }

    let listener = UnixListener::bind(&socket_path).context("Failed to bind UDS")?;
    info!("Listening on {}", socket_path);

    // 5. Accept Loop
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

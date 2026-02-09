// main.rs — SFCore AI Inference Server
//
// Windows: runs as a native Windows Service (install/start/stop/remove via CLI flags)
// Linux:   runs as a normal binary (use systemd to manage)

use anyhow::{Context, Result};
use clap::Parser;
use log::{error, info, warn};
use sfcore_ai_engine::{LlamaCppEngine, LlamaCppOptions};
use std::sync::Arc;

// UDS is Linux-only
#[cfg(unix)]
use tokio::net::UnixListener;

mod tcp_handler;
mod handler;
mod config;
mod error;
mod auth;
mod http_handler;

use config::Config;

// ============================================================
// Jemalloc – Linux / macOS only (MSVC = Windows)
// ============================================================
#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

// ============================================================
// Windows Service glue (kompilasi hanya di Windows)
// ============================================================
#[cfg(windows)]
mod windows_service_glue {
    use std::sync::mpsc;
    use std::time::Duration;

    use windows_service::service::{
        ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus, ServiceType,
    };
    use windows_service::service_control_handler::{self, ServiceControlHandlerResult};

    pub const SERVICE_NAME: &str = "SFCoreAIService";
    pub const SERVICE_DISPLAY_NAME: &str = "SFCore AI Inference Server";
    pub const SERVICE_DESCRIPTION: &str = "High-performance local LLM inference service";

    /// Jalankan closure `run_server` sebagai Windows Service.
    /// Menangani control events (Stop) dari SCM.
    pub fn run_as_service(
        run_server: impl FnOnce() -> anyhow::Result<()> + Send + 'static,
    ) -> anyhow::Result<()> {
        let (shutdown_tx, shutdown_rx) = mpsc::channel::<()>();

        // Control handler: SCM kirim Stop → kita kirim shutdown signal
        let control_handler = move |control: ServiceControl| -> ServiceControlHandlerResult {
            match control {
                ServiceControl::Stop => {
                    log::info!("[WinService] Received STOP signal");
                    let _ = shutdown_tx.send(());
                    ServiceControlHandlerResult::NoError
                }
                _ => ServiceControlHandlerResult::NoError,
            }
        };

        let status_handle =
            service_control_handler::register(SERVICE_NAME, control_handler)
                .map_err(|e| anyhow::anyhow!("Failed to register control handler: {}", e))?;

        // Report: Running
        status_handle
            .set_service_status(ServiceStatus {
                service_type: ServiceType::OWN_PROCESS,
                current_state: ServiceState::Running,
                controls_accepted: ServiceControlAccept::STOP,
                exit_code: ServiceExitCode::Win32(0),
                checkpoint: 0,
                wait_hint: Duration::from_secs(0),
                process_id: None,
            })
            .map_err(|e| anyhow::anyhow!("Failed to report Running: {}", e))?;

        log::info!("[WinService] Status: RUNNING");

        // Jalankan server di thread terpisah
        let _server_handle = std::thread::spawn(move || {
            if let Err(e) = run_server() {
                log::error!("[WinService] Server error: {}", e);
            }
        });

        // Tunggu sinyal shutdown dari control handler
        match shutdown_rx.recv() {
            Ok(()) => log::info!("[WinService] Shutdown signal received"),
            Err(_) => log::warn!("[WinService] Shutdown channel closed"),
        }

        // Report: Stopped
        let _ = status_handle.set_service_status(ServiceStatus {
            service_type: ServiceType::OWN_PROCESS,
            current_state: ServiceState::Stopped,
            controls_accepted: ServiceControlAccept::empty(),
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::from_secs(0),
            process_id: None,
        });

        log::info!("[WinService] Status: STOPPED");
        Ok(())
    }
}

// ============================================================
// CLI Args
// ============================================================
#[derive(Parser, Debug)]
#[command(name = "sfcore-ai-server", version, about = "SFCore AI Inference Server")]
struct Args {
    #[arg(long)]
    model: Option<String>,

    #[arg(long)]
    transport: Option<String>,

    /// Load model as embedding model instead of LLM
    #[arg(long, help = "Load model as embedding model (for vector embeddings)")]
    embedding: bool,

    // === Windows Service Management ===
    #[cfg(windows)]
    #[arg(long, help = "Install as Windows Service")]
    install: bool,

    #[cfg(windows)]
    #[arg(long, help = "Remove Windows Service")]
    remove: bool,

    #[cfg(windows)]
    #[arg(long, help = "Start Windows Service")]
    start: bool,

    #[cfg(windows)]
    #[arg(long, help = "Stop Windows Service")]
    stop: bool,

    /// Run as Windows Service (called internally by SCM, do not use manually)
    #[cfg(windows)]
    #[arg(long, help = "Run as Windows Service (internal, used by SCM)")]
    service: bool,

    // === Context & Memory ===
    #[arg(long, alias = "ctx-size")]
    context_length: Option<u32>,

    #[arg(long)]
    batch_size: Option<usize>,

    #[arg(long)]
    ubatch_size: Option<usize>,

    // === Threading ===
    #[arg(long)]
    threads: Option<u32>,

    #[arg(long)]
    threads_batch: Option<u32>,

    // === Parallel ===
    #[arg(long)]
    parallel: Option<u32>,

    // === Caching & Memory ===
    #[arg(long)]
    no_cache_prompt: bool,

    #[arg(long)]
    mlock: Option<bool>,

    // === Token Management ===
    #[arg(long)]
    keep: Option<i32>,

    // === Sampling ===
    #[arg(long)]
    seed: Option<u32>,

    #[arg(long)]
    temperature: Option<f32>,

    #[arg(long)]
    top_k: Option<i32>,

    #[arg(long)]
    top_p: Option<f32>,

    #[arg(long)]
    min_p: Option<f32>,

    // === Penalties ===
    #[arg(long)]
    repeat_penalty: Option<f32>,

    #[arg(long)]
    repeat_last_n: Option<i32>,

    #[arg(long)]
    frequency_penalty: Option<f32>,

    #[arg(long)]
    presence_penalty: Option<f32>,
}

// ============================================================
// Windows: install / remove / start / stop helpers
// ============================================================
#[cfg(windows)]
fn handle_windows_service_commands(args: &Args) -> Result<bool> {
    use std::ffi::OsString;
    use std::path::PathBuf;
    use windows_service::service::{
        ServiceAccess, ServiceErrorControl, ServiceStartType, ServiceType, ServiceInfo,
    };
    use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};
    use windows_service_glue::{SERVICE_DISPLAY_NAME, SERVICE_NAME};

    let is_service_cmd = args.install || args.remove || args.start || args.stop || args.service;
    if !is_service_cmd {
        return Ok(false);
    }

    // ---- INSTALL ----
    if args.install {
        let exe_path = std::env::current_exe().context("Failed to get current exe path")?;

        let manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE)
            .context("Failed to connect to Service Manager")?;

        let service_info = ServiceInfo {
            name: OsString::from(SERVICE_NAME),
            display_name: OsString::from(SERVICE_DISPLAY_NAME),
            service_type: ServiceType::OWN_PROCESS,
            start_type: ServiceStartType::AutoStart,
            error_control: ServiceErrorControl::Normal,
            executable_path: PathBuf::from(&exe_path),
            launch_arguments: vec![],
            dependencies: vec![],
            account_name: None,
            account_password: None,
        };

        manager
            .create_service(&service_info, ServiceAccess::all())
            .context("Failed to create service")?;

        info!("✅ Service '{}' installed", SERVICE_NAME);
        info!("   Exe : {}", exe_path.display());
        info!("   Type: Automatic start");
        info!("   Run : sc start {}", SERVICE_NAME);
        return Ok(true);
    }

    // ---- REMOVE ----
    if args.remove {
        let manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)
            .context("Failed to connect to Service Manager")?;

        let service = manager
            .open_service(SERVICE_NAME, ServiceAccess::DELETE)
            .context("Failed to open service — is it installed?")?;

        service.delete().context("Failed to delete service")?;
        info!("✅ Service '{}' removed", SERVICE_NAME);
        return Ok(true);
    }

    // ---- START ----
    if args.start {
        let manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)
            .context("Failed to connect to Service Manager")?;

        let service = manager
            .open_service(SERVICE_NAME, ServiceAccess::START)
            .context("Failed to open service")?;

        let args: Vec<std::ffi::OsString> = Vec::new();
        service.start(&args).context("Failed to start service")?;
        info!("✅ Service '{}' started", SERVICE_NAME);
        return Ok(true);
    }

    // ---- STOP ----
    if args.stop {
        let manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)
            .context("Failed to connect to Service Manager")?;

        let service = manager
            .open_service(SERVICE_NAME, ServiceAccess::STOP)
            .context("Failed to open service")?;

        service.stop().context("Failed to stop service")?;
        info!("✅ Service '{}' stopped", SERVICE_NAME);
        return Ok(true);
    }

    // ---- SERVICE (called by SCM) ----
    if args.service {
        info!("[WinService] Launched by SCM — running as Windows Service");

        windows_service_glue::run_as_service(|| {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
            rt.block_on(run_server_async())
        })?;

        return Ok(true);
    }

    Ok(false)
}

// ============================================================
// Build Engine Options: CLI > Config > Defaults
// ============================================================
fn build_engine_options(args: &Args, config: &Config) -> LlamaCppOptions {
    let cfg = config.engine.as_ref();

    let mut seed = args.seed.or(cfg.and_then(|e| e.seed)).unwrap_or(1234);
    
    // If seed is 0, generate a random one
    if seed == 0 {
        use std::time::{SystemTime, UNIX_EPOCH};
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        seed = since_the_epoch.as_millis() as u32;
        log::info!("🎲 Random Seed Generated: {}", seed);
    } else {
        log::info!("🔒 Fixed Seed: {}", seed);
    }

    LlamaCppOptions {
        context_length: args.context_length.or(cfg.and_then(|e| e.context_length)).unwrap_or(4096),
        batch_size:     args.batch_size.or(cfg.and_then(|e| e.batch_size)).unwrap_or(2048),
        ubatch_size:    args.ubatch_size.or(cfg.and_then(|e| e.ubatch_size)).unwrap_or(1024),

        threads:        Some(args.threads.or(cfg.and_then(|e| e.threads)).unwrap_or(4) as i32),
        threads_batch:  Some(args.threads_batch.or(cfg.and_then(|e| e.threads_batch)).unwrap_or(4) as i32),

        use_mlock:      args.mlock.or(cfg.and_then(|e| e.mlock)).unwrap_or(true),
        no_cache_prompt: args.no_cache_prompt || cfg.and_then(|e| e.no_cache_prompt).unwrap_or(false),

        seed,
        temperature:      args.temperature.or(cfg.and_then(|e| e.temperature)).unwrap_or(0.5),
        top_k:            args.top_k.or(cfg.and_then(|e| e.top_k)).unwrap_or(40),
        top_p:            args.top_p.or(cfg.and_then(|e| e.top_p)).unwrap_or(0.9),
        min_p:            args.min_p.or(cfg.and_then(|e| e.min_p)).unwrap_or(0.05),
        repeat_penalty:   args.repeat_penalty.or(cfg.and_then(|e| e.repeat_penalty)).unwrap_or(1.0),
        repeat_last_n:    args.repeat_last_n.or(cfg.and_then(|e| e.repeat_last_n)).unwrap_or(64),
        frequency_penalty: args.frequency_penalty.or(cfg.and_then(|e| e.frequency_penalty)).unwrap_or(0.0),
        presence_penalty:  args.presence_penalty.or(cfg.and_then(|e| e.presence_penalty)).unwrap_or(0.0),
    }
}

// ============================================================
// Core server logic (shared across all platforms)
// ============================================================
async fn run_server_async() -> Result<()> {
    unsafe {
        std::env::set_var("OPENBLAS_NUM_THREADS", "1");
        std::env::set_var("MKL_NUM_THREADS", "1");
    }

    let config = Config::from_file("server_config.toml")?;
    let args  = Args::parse();

    let model_path = args.model.clone()
        .or(config.model.clone())
        .ok_or_else(|| anyhow::anyhow!("Model path required (--model or config)"))?;

    let transport = args.transport.clone()
        .or(config.transport.clone())
        .unwrap_or_else(|| "tcp".to_string());

    info!("Model: {}", model_path);
    info!("Transport: {}", transport);

    let opts = build_engine_options(&args, &config);

    // Determine if we're loading an embedding model
    let is_embedding = args.embedding || config.is_embedding.unwrap_or(false);

    info!("=== Engine Configuration ===");
    info!("Context Length : {}", opts.context_length);
    info!("Batch / UBatch : {} / {}", opts.batch_size, opts.ubatch_size);
    info!("Threads        : {:?} / {:?}", opts.threads, opts.threads_batch);
    info!("mlock          : {}", opts.use_mlock);
    info!("no_cache_prompt: {}", opts.no_cache_prompt);
    info!("Temp / TopK / TopP / MinP: {} / {} / {} / {}", opts.temperature, opts.top_k, opts.top_p, opts.min_p);
    info!("Model Mode     : {}", if is_embedding { "EMBEDDING" } else { "LLM" });
    info!("===========================");

    let mut engine = LlamaCppEngine::new(opts)?;
    
    if is_embedding {
        info!("🔗 Loading embedding model...");
        engine.load_gguf_embedding(&model_path)?;
    } else {
        info!("💬 Loading LLM model...");
        engine.load_gguf(&model_path)?;
    }
    
    let engine = Arc::new(engine);

    match transport.as_str() {
        #[cfg(unix)]
        "uds" => run_uds_server(engine, &config).await,

        "http-sse" => run_http_server(engine, &config).await,
        "tcp"      => run_tcp_server(engine, &config).await,

        #[cfg(windows)]
        "uds" => Err(anyhow::anyhow!("UDS is not supported on Windows. Use 'tcp' or 'http-sse'.")),

        _ => Err(anyhow::anyhow!("Unknown transport: {}", transport)),
    }
}

// ============================================================
// main()
// ============================================================
#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    // Windows: proses service management commands dulu
    #[cfg(windows)]
    {
        let args = Args::parse();
        if handle_windows_service_commands(&args)? {
            return Ok(());
        }
    }

    // Normal standalone run
    info!("SFCore AI Server starting (standalone mode)...");
    run_server_async().await
}

// ============================================================
// Transport: UDS (Linux only)
// ============================================================
#[cfg(unix)]
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
                let eng = engine.clone();
                tokio::spawn(async move {
                    if let Err(e) = handler::handle_connection(stream, eng).await {
                        error!("UDS connection error: {}", e);
                    }
                });
            }
            Err(e) => error!("UDS accept failed: {}", e),
        }
    }
}

// ============================================================
// Transport: HTTP-SSE
// ============================================================
async fn run_http_server(engine: Arc<LlamaCppEngine>, config: &Config) -> Result<()> {
    use axum::{
        middleware,
        routing::{get, post},
        Router,
    };
    use tower_http::trace::TraceLayer;

    let http_cfg = config.http.as_ref()
        .ok_or_else(|| anyhow::anyhow!("Missing [http] config"))?;
    let auth_cfg = config.auth.as_ref()
        .ok_or_else(|| anyhow::anyhow!("Missing [auth] config"))?;
    let rl_cfg = config.rate_limit.as_ref()
        .ok_or_else(|| anyhow::anyhow!("Missing [rate_limit] config"))?;

    let app_state  = http_handler::AppState { engine: engine.clone() };
    let auth_state = auth::AuthState::new(auth_cfg, rl_cfg.requests_per_minute);

    let app = Router::new()
        .route("/v1/inference", post(http_handler::inference_handler))
        .route("/health", get(http_handler::health_handler))
        .layer(middleware::from_fn_with_state(auth_state, auth::auth_middleware))
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    let addr = format!("{}:{}", http_cfg.host, http_cfg.port);
    info!("HTTP-SSE listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

// ============================================================
// Transport: TCP
// ============================================================
async fn run_tcp_server(engine: Arc<LlamaCppEngine>, config: &Config) -> Result<()> {
    let tcp_cfg = config.tcp.as_ref()
        .ok_or_else(|| anyhow::anyhow!("Missing [tcp] config"))?;

    let addr = format!("{}:{}", tcp_cfg.host, tcp_cfg.port);

    if tcp_cfg.host == "0.0.0.0" {
        warn!("⚠️  TCP binding to 0.0.0.0 — exposed without encryption!");
        warn!("⚠️  Recommended: bind to 127.0.0.1 + SSH tunnel for production");
    }

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .with_context(|| format!("Failed to bind TCP: {}", addr))?;

    info!("TCP listening on {}", addr);

    loop {
        match listener.accept().await {
            Ok((stream, peer)) => {
                info!("TCP connection from: {}", peer);
                let eng = engine.clone();
                tokio::spawn(async move {
                    if let Err(e) = tcp_handler::handle_connection(stream, eng).await {
                        error!("[{}] TCP error: {}", peer, e);
                    }
                });
            }
            Err(e) => error!("TCP accept failed: {}", e),
        }
    }
}
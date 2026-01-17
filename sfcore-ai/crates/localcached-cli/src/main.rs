mod app;
mod event;
mod theme;
mod ui;

use app::App;
use ratatui::DefaultTerminal;
use clap::{Parser, Subcommand};
use std::process::Command;
use std::path::Path;
use std::fs;

#[derive(Parser)]
#[command(name = "localcached-cli")]
#[command(about = "CLI & TUI Manager for LocalCached")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Path to the socket file
    #[arg(short, long, env = "LOCALCACHED_SOCKET", default_value = "/run/localcached.sock")]
    socket: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the localcached server in background
    Start {
         /// Path to server binary. If not provided, searches in standard paths or PATH.
         #[arg(long, env = "LOCALCACHED_BIN")]
         bin: Option<String>,
    },
    /// Stop the localcached server
    Stop,
    /// Open the TUI (Terminal User Interface) - Default
    Monitor,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    color_eyre::install().ok();
    let cli = Cli::parse();
    
    let socket_path = cli.socket.clone();

    match cli.command.unwrap_or(Commands::Monitor) {
        Commands::Start { bin } => handle_start(bin).await?,
        Commands::Stop => handle_stop(&socket_path).await?,
        Commands::Monitor => start_tui(socket_path).await?,
    }

    Ok(())
}

async fn handle_start(bin_path: Option<String>) -> anyhow::Result<()> {
    let binary = if let Some(b) = bin_path {
        b
    } else {
        find_server_binary()
            .ok_or_else(|| anyhow::anyhow!("Count not find 'localcached-server' binary. Please set LOCALCACHED_BIN or put it in PATH."))?
    };

    println!("Starting server: {}", binary);
    // Use nohup-like behavior or just spawn?
    // Spawn checks:
    let child = Command::new(binary)
        .spawn()
        .map_err(|e| anyhow::anyhow!("Failed to spawn server: {}", e))?;
    
    println!("Server started with PID: {}", child.id());
    Ok(())
}

fn find_server_binary() -> Option<String> {
    // 1. Check relative target/release (dev context)
    let p = Path::new("target/release/localcached-server");
    if p.exists() { return Some(p.to_string_lossy().to_string()); }
    
    // 2. Check ../target/release (if running from crate dir)
    let p = Path::new("../target/release/localcached-server");
    if p.exists() { return Some(p.to_string_lossy().to_string()); }

    // 3. Check PATH using 'which' command
    if let Ok(output) = Command::new("which").arg("localcached-server").output() {
        if output.status.success() {
             // Clean the output string
             let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
             if !path.is_empty() {
                 return Some(path);
             }
        }
    }

    // 4. Try current dir
    let p = Path::new("./localcached-server");
    if p.exists() { return Some("./localcached-server".to_string()); }

    None
}

async fn handle_stop(socket_path: &str) -> anyhow::Result<()> {
    let pid_path = derive_pid_path(socket_path);
    if !Path::new(&pid_path).exists() {
        // Fallback: Check process list? Too complex.
        // Assuming if socket exists but no PID, maybe we can't stop it cleanly via PID.
        println!("PID file not found at {}. Cannot stop reliably.", pid_path);
        return Err(anyhow::anyhow!("PID file missing"));
    }

    let pid_str = fs::read_to_string(&pid_path)?;
    let pid = pid_str.trim().parse::<i32>()?;
    
    println!("Stopping server PID: {}", pid);
    
    // Send SIGTERM
    let status = Command::new("kill")
        .arg(pid.to_string())
        .status()?;
    
    if status.success() {
        println!("Server stopped.");
         // Clean up PID file? Server writes it, but processes killed might not clean it.
         let _ = fs::remove_file(pid_path);
    } else {
        println!("Failed to kill process.");
    }

    Ok(())
}

fn derive_pid_path(socket_path: &str) -> String {
    // If LOCALCACHED_PID_FILE env is set, use it (CLI should be aware? Env passed to this proc)
    if let Ok(v) = std::env::var("LOCALCACHED_PID_FILE") {
        return v;
    }
    
    let path = Path::new(socket_path);
    if let Some(parent) = path.parent() {
         let stem = path.file_stem().unwrap_or_default();
         let mut pid_name = stem.to_os_string();
         pid_name.push(".pid");
         return parent.join(pid_name).to_string_lossy().to_string();
    }
    // Fallback
    "/run/localcached.pid".to_string()
}

async fn start_tui(socket_path: String) -> anyhow::Result<()> {
    // Initialize terminal (ratatui::init handles raw mode + alternate screen)
    let terminal = ratatui::init();

    // Run app
    let result = run_app(terminal, socket_path).await;

    // Restore terminal (always, even on error)
    ratatui::restore();

    result
}

// ... existing run_app ...
async fn run_app(mut terminal: DefaultTerminal, socket_path: String) -> anyhow::Result<()> {
    let mut app = App::new(socket_path);

    loop {
        // Draw
        terminal.draw(|frame| ui::render(frame, &app))?;

        // Check quit flag
        if app.should_quit {
            break;
        }

        // Poll events (non-blocking with 100ms timeout)
        if let Some(evt) = event::poll_event(100)? {
            event::handle_event(&mut app, evt).await?;
        }
    }
    Ok(())
}

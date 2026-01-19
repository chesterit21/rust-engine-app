//! Engine ViewModel
//!
//! Manages SFCore AI Engine server state and chat interaction.

use crate::core::config::AppConfig;
use crate::core::uds_client::UdsClient;
use crate::events::AppEvent;
use std::process::Stdio;
use std::time::{Duration, Instant};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;

/// Server status enum
#[derive(Debug, Clone, PartialEq)]
pub enum ServerStatus {
    Stopped,
    Starting,
    WarmingUp,  // Socket exists but model still loading
    Running,    // Fully ready to accept requests
    Error(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
}

#[derive(Debug, Clone, Default)]
pub struct InferenceMetrics {
    pub tokens_generated: i32,
    pub speed_tokens_sec: f32,
    pub total_time_ms: u128,
}

/// EngineViewModel - manages server process and chat
pub struct EngineViewModel {
    pub status: ServerStatus,
    server_process: Option<Child>,
    pub chat_input: String,
    pub messages: Vec<ChatMessage>,
    pub logs: Vec<String>,
    pub is_loading: bool,
    pub current_metrics: Option<InferenceMetrics>,
    event_tx: mpsc::UnboundedSender<AppEvent>,
    config: AppConfig,
}

impl EngineViewModel {
    pub fn new(event_tx: mpsc::UnboundedSender<AppEvent>) -> Self {
        Self {
            status: ServerStatus::Stopped,
            server_process: None,
            chat_input: String::new(),
            messages: Vec::with_capacity(100),  // PERFORMANCE: Pre-allocate
            logs: Vec::with_capacity(50),       // PERFORMANCE: Pre-allocate
            is_loading: false,
            current_metrics: None,
            event_tx,
            config: AppConfig::load(),
        }
    }

    pub fn is_running(&self) -> bool {
        matches!(self.status, ServerStatus::Running)
    }
    
    /// Returns true if server is ready to accept chat requests
    pub fn is_ready(&self) -> bool {
        matches!(self.status, ServerStatus::Running)
    }
    
    /// Clear all chat messages and logs
    pub fn clear_chat(&mut self) {
        self.messages.clear();
        self.logs.clear();
        self.current_metrics = None;
    }

    pub fn toggle_server(&mut self) {
        if self.is_running() || matches!(self.status, ServerStatus::Starting) {
            self.stop_server();
        } else {
            self.start_server();
        }
    }

    pub fn start_server(&mut self) {
        if self.server_process.is_some() {
            return;
        }

        self.status = ServerStatus::Starting;
        self.add_system_message("⏳ Starting server...");

        // Use release binary (faster and more stable)
        let child_result = Command::new("./target/release/sfcore-ai-server")
            .current_dir("/home/sfcore/SFCoreAIApps/AIRustTools/root-app/sfcore-ai")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn();

        match child_result {
            Ok(child) => {
                self.server_process = Some(child);
                // Don't set Running yet! Wait for socket confirmation
                self.status = ServerStatus::Starting; 

                // Async wait for socket
                let tx = self.event_tx.clone();
                let socket_path = self.config.server.socket_path.clone(); // Use config
                let timeout_secs = self.config.server.timeout_seconds;

                tokio::spawn(async move {
                    let start = Instant::now();
                    let timeout = Duration::from_secs(timeout_secs); 

                    // Phase 1: Wait for socket file to appear
                    while start.elapsed() < timeout {
                        if std::path::Path::new(&socket_path).exists() {
                            let _ = tx.send(AppEvent::EngineResponse(
                                "🔍 [DEBUG] Socket found, warming up model...".to_string(),
                            ));
                            let _ = tx.send(AppEvent::ServerStatusChange("WarmingUp".to_string()));
                            break;
                        }
                        tokio::time::sleep(Duration::from_millis(500)).await;
                    }
                    
                    // Phase 2: Ping socket to verify it's accepting connections
                    let mut ready = false;
                    while start.elapsed() < timeout && !ready {
                        match tokio::net::UnixStream::connect(&socket_path).await {
                            Ok(_) => {
                                ready = true;
                                let _ = tx.send(AppEvent::EngineResponse(
                                    "✅ Server ready! Accepting connections.".to_string(),
                                ));
                                let _ = tx.send(AppEvent::ServerStatusChange("Running".to_string()));
                            }
                            Err(_) => {
                                tokio::time::sleep(Duration::from_millis(1000)).await;
                            }
                        }
                    }
                    
                    if !ready {
                        let _ = tx.send(AppEvent::EngineResponse("❌ Error: Server start timeout".to_string()));
                        let _ = tx.send(AppEvent::ServerStatusChange("Error".to_string()));
                    }
                });
            }
            Err(e) => {
                self.status = ServerStatus::Error(format!("Failed to start: {}", e));
                self.add_system_message(&format!("❌ Failed to start server: {}", e));
            }
        }
    }

    pub fn stop_server(&mut self) {
        if let Some(mut child) = self.server_process.take() {
            self.status = ServerStatus::Stopped;
            self.add_system_message("🛑 Stopping server...");
            
            // Spawn async shutdown task
            tokio::spawn(async move {
                // Graceful shutdown with SIGTERM
                if let Some(id) = child.id() {
                    unsafe { libc::kill(id as i32, libc::SIGTERM) };
                }

                // Wait for exit with timeout
                match tokio::time::timeout(Duration::from_secs(5), child.wait()).await {
                    Ok(_) => {}, // Exited gracefully
                    Err(_) => {
                        // Timeout, force kill
                        let _ = child.kill().await;
                    }
                }
                
                // Cleanup socket
                let _ = std::fs::remove_file("/tmp/sfcore-ai.sock");
            });
            
            self.add_system_message("🛑 Server stopped.");
        } else {
            self.status = ServerStatus::Stopped;
        }
    }

    pub fn send_message(&mut self) {
        if self.chat_input.trim().is_empty() {
            return;
        }

        if !self.is_running() {
            self.add_system_message("⚠️ Server is not running. Click play to start.");
            return;
        }

        self.is_loading = true;
        self.current_metrics = None; // Reset metrics
        let prompt = self.chat_input.clone();
        
        self.messages.push(ChatMessage {
            role: MessageRole::User,
            content: prompt.clone(),
        });
        
        self.chat_input.clear();
        
        let tx_event = self.event_tx.clone();
        let socket_path = self.config.server.socket_path.clone();

        tokio::spawn(async move {
            // let _ = tx_event.send(AppEvent::EngineResponse(format!("🔍 [DEBUG] Connecting to {}", socket_path)));
            let client = UdsClient::new(&socket_path);
            let (tx_stream, mut rx_stream) = mpsc::unbounded_channel();
            
            // Clone tx_event for the streaming task
            let tx_event_stream = tx_event.clone();
            
            // Start streaming in background task (don't await here!)
            let stream_handle = tokio::spawn(async move {
                client.stream_chat(&prompt, 1024, tx_stream).await
            });
            
            // PERFORMANCE IMPROVEMENT: Token batching
            // Buffer tokens and send in batches to reduce UI updates
            let mut buffer = String::with_capacity(256);
            let mut last_update = Instant::now();
            let batch_interval = Duration::from_millis(50); // 20 updates/sec max
            
            let mut chunk_count = 0;
            while let Some(chunk) = rx_stream.recv().await {
                chunk_count += 1;
                buffer.push_str(&chunk);
                
                // Send batch if:
                // 1. Buffer >= 10 chars, OR
                // 2. Interval elapsed (50ms)
                if buffer.len() >= 10 || last_update.elapsed() >= batch_interval {
                    if !buffer.is_empty() {
                        // PERFORMANCE: Use take() to avoid clone allocation
                        let _ = tx_event_stream.send(AppEvent::EngineResponse(
                            std::mem::take(&mut buffer)
                        ));
                        last_update = Instant::now();
                    }
                }
            }
            
            // Flush remaining buffer at end of stream
            if !buffer.is_empty() {
                let _ = tx_event_stream.send(AppEvent::EngineResponse(
                    std::mem::take(&mut buffer)
                ));
            }
            
            // Wait for streaming task to complete and check for errors
            if let Ok(result) = stream_handle.await {
                if let Err(e) = result {
                    let _ = tx_event.send(AppEvent::EngineResponse(format!("❌ Error: {}", e)));
                }
            }
            
            let _ = tx_event.send(AppEvent::EngineResponse(format!("🔍 [DEBUG] Stream finished. Total chunks: {}", chunk_count)));
            // Signal streaming is done - reset loading state
            let _ = tx_event.send(AppEvent::StreamEnd);
        });
    }
    
    pub fn append_response(&mut self, text: &str) {
        // Handle system messages
        if text.starts_with("✅") || text.starts_with("❌") || text.starts_with("⏳") || text.starts_with("🛑") || text.starts_with("⚠️") {
             if text.contains("Server ready") {
                self.status = ServerStatus::Running;
            } else if text.contains("Error: Server start timeout") {
                self.status = ServerStatus::Error("Timeout".to_string());
                self.server_process = None;
            }
            self.add_system_message(text);
            self.is_loading = false;
            return;
        }
        
        // Debug logs
        if text.starts_with("🔍") {
            self.log_message(text);
            return;
        }

        // Logic for Assistant response streaming
        if self.is_loading {
             let last_is_assistant = self.messages.last().map_or(false, |m| matches!(m.role, MessageRole::Assistant));
             if !last_is_assistant {
                 self.messages.push(ChatMessage {
                     role: MessageRole::Assistant,
                     content: String::new(),
                 });
             }
             // Don't set is_loading = false yet for streaming, wait for completion?
             // Actually for streaming, we want to stay "loading" (showing cursor/typing) until done.
             // But our current architecture sends raw strings. 
             // Ideally we need an explicit Done event.
             // For now, let's keep is_loading = true until we decide stream is over. 
             // But we don't have a "StreamEnd" event yet.
             // Let's modify UdsClient to send encoded events or handle this better.
             // Hack: For now, keep is_loading true. But when do we stop?
             // Improve: Add StreamEnd event.
        }
        
        if let Some(last_msg) = self.messages.last_mut() {
            if matches!(last_msg.role, MessageRole::Assistant) {
                last_msg.content.push_str(text);
                
                // Update metrics (simple estimation)
                let tokens = last_msg.content.split_whitespace().count() as i32; // Rough est
                // Real implementation would parse metrics from `stream_chat` metrics payload when done.
                if let Some(metrics) = &mut self.current_metrics {
                     metrics.tokens_generated = tokens;
                } else {
                    self.current_metrics = Some(InferenceMetrics {
                        tokens_generated: tokens,
                        speed_tokens_sec: 0.0, // Calculate delta?
                        total_time_ms: 0,
                    });
                }
            } else {
                 self.messages.push(ChatMessage {
                     role: MessageRole::Assistant,
                     content: text.to_string(),
                 });
            }
        }
        
        // Ensure UI repaints for streaming
        // (AppEvent handling usually triggers repaint)
    }
    
    fn log_message(&mut self, text: &str) {
        self.logs.push(format!("[{}] {}", chrono::Local::now().format("%H:%M:%S"), text));
    }

    fn add_system_message(&mut self, text: &str) {
        self.log_message(text);
        
        self.messages.push(ChatMessage {
            role: MessageRole::System,
            content: text.to_string(),
        });
    }
}

impl Drop for EngineViewModel {
    fn drop(&mut self) {
        if self.server_process.is_some() {
             self.stop_server();
        }
    }
}

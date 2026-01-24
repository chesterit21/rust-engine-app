use super::{EmbeddingProvider, EmbeddingRequest, EmbeddingResponse};
use crate::config::LlamaServerConfig;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Serialize};
use std::process::{Child, Command, Stdio};
use std::time::Duration;
use sysinfo::System;
use tokio::time::{sleep, timeout};
use tracing::{debug, error, info, warn};

#[derive(Debug, Serialize)]
struct LlamaEmbeddingRequest {
    content: String,
}



pub struct LlamaServerManager {
    config: LlamaServerConfig,
    client: Client,
    process: Option<Child>,
    base_url: String,
}

impl LlamaServerManager {
    pub fn new(config: LlamaServerConfig) -> Self {
        let base_url = format!("http://{}:{}", config.host, config.port);
        
        Self {
            config,
            client: Client::builder()
                .timeout(Duration::from_secs(300))
                .build()
                .expect("Failed to create HTTP client"),
            process: None,
            base_url,
        }
    }
    
    /// Check available system memory
    pub fn check_memory(&self) -> Result<u64> {
        let mut sys = System::new_all();
        sys.refresh_memory();
        
        let available_mb = sys.available_memory() / 1024 / 1024;
        info!("Available memory: {} MB", available_mb);
        
        if available_mb < 2048 {
            anyhow::bail!("Not enough memory to start embedding server (< 2GB available)");
        }
        
        Ok(available_mb)
    }
    
    /// Start llama-server process
    pub async fn start(&mut self) -> Result<()> {
        // Check if already running
        if self.is_running().await {
            info!("Llama-server already running");
            return Ok(());
        }
        
        // Check available memory
        self.check_memory()?;
        
        info!("🚀 Starting llama-server...");
        
        // Build command
        let mut cmd = Command::new(&self.config.binary_path);
        
        cmd.arg("--model")
            .arg(&self.config.model_path)
            .arg("--host")
            .arg(&self.config.host)
            .arg("--port")
            .arg(self.config.port.to_string())
            .arg("--ctx-size")
            .arg(self.config.ctx_size.to_string())
            .arg("--threads")
            .arg(self.config.threads.to_string());
        
        // Embedding-only mode (no generation)
        if self.config.embedding_only {
            cmd.arg("--embedding");
        }
        
        // Suppress verbose output
        cmd.stdout(Stdio::null())
            .stderr(Stdio::piped());
        
        // Spawn process
        let child = cmd.spawn()
            .map_err(|e| anyhow!("Failed to spawn llama-server: {}", e))?;
        
        self.process = Some(child);
        
        info!("Process spawned, waiting for server to be ready...");
        
        // Wait for server to be ready
        let ready = timeout(
            Duration::from_secs(self.config.startup_timeout_seconds),
            self.wait_until_ready(),
        )
        .await;
        
        match ready {
            Ok(Ok(_)) => {
                info!("✅ Llama-server ready");
                Ok(())
            }
            Ok(Err(e)) => {
                self.stop().await?;
                Err(anyhow!("Server failed to start: {}", e))
            }
            Err(_) => {
                self.stop().await?;
                Err(anyhow!("Server startup timeout"))
            }
        }
    }
    
    /// Check if server is running and responding
    async fn is_running(&self) -> bool {
        match self.client
            .get(&format!("{}/health", self.base_url))
            .send()
            .await
        {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }
    
    /// Wait until server is ready
    async fn wait_until_ready(&self) -> Result<()> {
        let mut attempts = 0;
        let max_attempts = 60; // 60 attempts * 1 second = 1 minute
        
        loop {
            if self.is_running().await {
                return Ok(());
            }
            
            attempts += 1;
            if attempts >= max_attempts {
                return Err(anyhow!("Server not responding after {} attempts", max_attempts));
            }
            
            sleep(Duration::from_secs(1)).await;
        }
    }
    
    /// Stop llama-server process
    pub async fn stop(&mut self) -> Result<()> {
        if let Some(mut child) = self.process.take() {
            info!("🛑 Stopping llama-server...");
            
            // Try graceful shutdown first
            match child.kill() {
                Ok(_) => {
                    // Wait for process to exit
                    let wait_result = timeout(
                        Duration::from_secs(self.config.shutdown_timeout_seconds),
                        tokio::task::spawn_blocking(move || child.wait()),
                    )
                    .await;
                    
                    match wait_result {
                        Ok(Ok(Ok(status))) => {
                            info!("Llama-server stopped with status: {}", status);
                        }
                        _ => {
                            warn!("Failed to wait for process exit");
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to kill process: {}", e);
                }
            }
        }
        
        Ok(())
    }
    
    /// Embed single text
    async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        let request = LlamaEmbeddingRequest {
            content: text.to_string(),
        };
        
        let response = self
            .client
            .post(&format!("{}/embedding", self.base_url))
            .json(&request)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Embedding request failed: {} - {}", status, body);
        }
        
        // Dynamic parsing to separate legacy vs new llama-server format
        let json: serde_json::Value = response.json().await?;
        
        // DEBUG: Print full response
        tracing::debug!("Raw Llama Server response: {}", serde_json::to_string(&json).unwrap_or_default());
        
        let embedding: Vec<f32> = if json.is_array() {
            let arr = json.as_array().unwrap();
            if arr.is_empty() {
                anyhow::bail!("Received empty array from llama-server");
            }
            
            let first_item = &arr[0];
            tracing::debug!("First item structure: {}", serde_json::to_string(first_item).unwrap_or_default());
            
            // New Format: [{"index":0, "embedding":[...]}]
            let emb_field = first_item["embedding"]
                .as_array()
                .ok_or_else(|| anyhow!("Missing 'embedding' field in new format API response"))?;
            
            tracing::debug!("Embedding field length: {}", emb_field.len());
            
            // Check for double nesting [[...]] which sometimes happens with batch requests
            if !emb_field.is_empty() && emb_field[0].is_array() {
                tracing::debug!("Detected nested embedding format [[...]]");
                emb_field[0]
                    .as_array()
                    .ok_or_else(|| anyhow!("Invalid nested embedding format"))?
                    .iter()
                    .filter_map(|v| v.as_f64().map(|f| f as f32))
                    .collect()
            } else {
                tracing::debug!("Detected flat embedding format [...]");
                emb_field
                    .iter()
                    .filter_map(|v| v.as_f64().map(|f| f as f32))
                    .collect()
            }
        } else {
            // Legacy Format: {"embedding":[...]}
            tracing::debug!("Using legacy format parsing");
            json["embedding"]
                .as_array()
                .ok_or_else(|| anyhow!("Missing 'embedding' field in legacy API response"))?
                .iter()
                .filter_map(|v| v.as_f64().map(|f| f as f32))
                .collect()
        };
        
        if embedding.is_empty() {
             anyhow::bail!("Parsed embedding vector is empty!");
        }
        
        tracing::debug!("Successfully parsed embedding vector, length: {}", embedding.len());
        
        Ok(embedding)
    }
}

#[async_trait]
impl EmbeddingProvider for LlamaServerManager {
    async fn embed(&self, request: EmbeddingRequest) -> Result<EmbeddingResponse> {
        let mut embeddings = Vec::with_capacity(request.texts.len());
        
        for (i, text) in request.texts.iter().enumerate() {
            debug!("Embedding text {}/{}", i + 1, request.texts.len());
            
            let embedding = self.embed_text(text).await?;
            embeddings.push(embedding);
        }
        
        Ok(EmbeddingResponse { embeddings })
    }
    
    async fn embed_single(&self, text: String) -> Result<Vec<f32>> {
        self.embed_text(&text).await
    }
}

impl Drop for LlamaServerManager {
    fn drop(&mut self) {
        // Cleanup on drop
        if let Some(mut child) = self.process.take() {
            let _ = child.kill();
        }
    }
}
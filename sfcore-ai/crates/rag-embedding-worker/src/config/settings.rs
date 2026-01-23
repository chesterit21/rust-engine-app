use anyhow::Result;
use config::{Config, ConfigError, Environment, File};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Settings {
    pub database: DatabaseConfig,
    pub embedding: EmbeddingConfig,
    pub chunking: ChunkingConfig,
    pub worker: WorkerConfig,
    pub llama_server: LlamaServerConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub pool_max_size: u32,
    pub pool_timeout_seconds: u64,
    pub listen_channel: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EmbeddingConfig {
    pub model: String,  // model name untuk llama-server
    pub dimension: usize,  // 384 untuk AllMiniLML6V2, 1536 untuk OpenAI
    pub batch_size: usize,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ChunkingConfig {
    pub size: usize,
    pub overlap: usize,
    #[serde(default = "default_strategy")]
    pub strategy: ChunkStrategy,
}

fn default_strategy() -> ChunkStrategy {
    ChunkStrategy::Semantic
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ChunkStrategy {
    Semantic,  // Semantic splitting (best untuk RAG)
    Fixed,     // Fixed size chunks
    Recursive, // Recursive character splitting
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WorkerConfig {
    pub threads: usize,
    pub bulk_batch_size: usize,
    pub processing_timeout_seconds: u64,
    pub document_root_path: PathBuf,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LlamaServerConfig {
    pub binary_path: PathBuf,  // path ke llama-server binary
    pub model_path: PathBuf,    // path ke model embedding
    pub host: String,
    pub port: u16,
    pub startup_timeout_seconds: u64,
    pub shutdown_timeout_seconds: u64,
    #[serde(default = "default_embedding_flag")]
    pub embedding_only: bool,
    #[serde(default = "default_ctx_size")]
    pub ctx_size: u32,
    #[serde(default = "default_threads")]
    pub threads: i32,
}

fn default_embedding_flag() -> bool {
    true  // embedding-only mode by default
}

fn default_ctx_size() -> u32 {
    2048
}

fn default_threads() -> i32 {
    4
}

impl Settings {
    pub fn load() -> Result<Self> {
        // Load from environment first
        dotenvy::dotenv().ok();
        
        let config = Config::builder()
            // Load from config file
            .add_source(File::with_name("config/settings").required(false))
            // Override with environment variables (prefix: APP)
            // Example: APP_DATABASE__URL=postgres://...
            .add_source(
                Environment::with_prefix("APP")
                    .separator("__")
                    .try_parsing(true)
            )
            .build()?;
        
        let settings: Settings = config.try_deserialize()?;
        
        // Validate settings
        settings.validate()?;
        
        Ok(settings)
    }
    
    fn validate(&self) -> Result<()> {
        // Validate llama-server binary exists
        if !self.llama_server.binary_path.exists() {
            anyhow::bail!(
                "llama-server binary not found at: {:?}",
                self.llama_server.binary_path
            );
        }
        
        // Validate model exists
        if !self.llama_server.model_path.exists() {
            anyhow::bail!(
                "Embedding model not found at: {:?}",
                self.llama_server.model_path
            );
        }
        
        // Validate document root path
        if !self.worker.document_root_path.exists() {
            anyhow::bail!(
                "Document root path not found: {:?}",
                self.worker.document_root_path
            );
        }
        
        Ok(())
    }
}

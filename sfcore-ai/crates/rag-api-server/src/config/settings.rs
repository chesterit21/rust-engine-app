use anyhow::Result;
use config::{Config, ConfigError, Environment, File};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Settings {
    pub server: ServerConfig,
    pub security: SecurityConfig,
    pub database: DatabaseConfig,
    pub embedding: EmbeddingConfig,
    pub llm: LlmConfig,
    pub rag: RagConfig,
    pub prompts: PromptsConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub max_connections: usize,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SecurityConfig {
    pub allowed_ips: Vec<String>,
    pub custom_headers: CustomHeadersConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CustomHeadersConfig {
    pub app_id: String,
    pub api_key: String,
    pub request_signature: String,
    pub timestamp_tolerance: i64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub pool_max_size: u32,
    pub pool_timeout_seconds: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EmbeddingConfig {
    pub model: String,
    pub base_url: String, // Added base_url for embedding server
    pub dimension: usize,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LlmConfig {
    pub base_url: String,
    pub timeout_seconds: u64,
    pub max_tokens: usize,
    pub stream_response: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RagConfig {
    pub retrieval_top_k: usize,
    pub chunk_size: usize,
    pub chunk_overlap_percentage: f32,
    pub rerank_enabled: bool,
    pub max_context_length: usize,  // Keep for backward compat
    pub max_context_tokens: usize,  // NEW: token-based limit
    pub document_path: String,
}


#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PromptsConfig {
    pub main_system_prompt: String,
    pub context_extraction_system_prompt: String,
}

impl Settings {
    pub fn load() -> Result<Self> {
        dotenvy::dotenv().ok();
        
        let config = Config::builder()
            .add_source(File::with_name("config/settings").required(true))
            .add_source(
                Environment::with_prefix("APP")
                    .separator("__")
                    .try_parsing(true)
            )
            .build()?;
        
        let settings: Settings = config.try_deserialize()?;
        Ok(settings)
    }
    
    pub fn config_path(&self) -> PathBuf {
        PathBuf::from("config/settings.toml")
    }
}

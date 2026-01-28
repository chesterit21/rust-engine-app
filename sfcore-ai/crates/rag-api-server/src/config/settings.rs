use anyhow::Result;
use config::{Config, Environment, File};
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
    pub limits: LimitsConfig,
    pub gemini: Option<GeminiConfig>, // NEW
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GeminiConfig {
    pub enabled: bool, // NEW
    pub api_key: String,
    pub model: Option<String>,
    pub embedding_model: Option<String>,
    pub base_url: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LimitsConfig {
    pub embedding_concurrency: usize,
    pub db_search_concurrency: usize,
    pub llm_generate_concurrency: usize,
    pub llm_stream_concurrency: usize,
    pub acquire_timeout_ms: u64,
    pub embedding_batch_size: usize, // NEW
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
    pub base_url: String,
    pub dimension: usize,
    pub api_key: Option<String>, // NEW
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LlmConfig {
    pub base_url: String,
    pub timeout_seconds: u64,
    pub max_tokens: usize,
    pub stream_response: bool,
    pub api_key: Option<String>,
    pub model: Option<String>, // NEW: Required for Gemini/OpenAI
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RagConfig {
    pub retrieval_top_k: usize,
    pub chunk_size: usize,
    pub chunk_overlap_percentage: f32,
    pub rerank_enabled: bool,
    pub max_context_length: usize,
    pub max_context_tokens: usize,
    pub deep_scan_batch_tokens: usize, // NEW
    pub document_path: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PromptsConfig {
    pub local: PromptSet,
    pub gemini: PromptSet,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PromptSet {
    pub main_system_prompt: String,
    pub context_extraction_system_prompt: String,
    pub rag_query_system_prompt: String,
    pub deep_scan_system_prompt: String,
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
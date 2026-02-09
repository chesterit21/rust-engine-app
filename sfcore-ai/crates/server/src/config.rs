//! Configuration management with transport selection

use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub model: Option<String>,
    pub transport: Option<String>,
    pub is_embedding: Option<bool>,
    
    // Auth config
    pub auth: Option<AuthConfig>,
    
    // Rate limit config
    pub rate_limit: Option<RateLimitConfig>,
    
    // Transport-specific configs
    pub uds: Option<UdsConfig>,
    pub http: Option<HttpConfig>,
    pub tcp: Option<TcpConfig>,
    
    // Engine params (NEW: All in engine section)
    pub engine: Option<EngineConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    pub api_key: String,
    pub allowed_clients: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UdsConfig {
    pub socket: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HttpConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TcpConfig {
    pub host: String,
    pub port: u16,
}

/// Complete engine configuration
#[derive(Debug, Clone, Deserialize)]
pub struct EngineConfig {
    // Context & Memory
    pub context_length: Option<u32>,
    pub batch_size: Option<usize>,
    pub ubatch_size: Option<usize>,
    
    // Threading
    pub threads: Option<u32>,
    pub threads_batch: Option<u32>,
    
    // Parallel Processing
    pub parallel: Option<u32>,
    
    // Caching & Memory
    pub no_cache_prompt: Option<bool>,
    pub mlock: Option<bool>,
    
    // Token Management
    pub keep_tokens: Option<i32>,
    
    // Sampling
    pub seed: Option<u32>,
    pub temperature: Option<f32>,
    pub top_k: Option<i32>,
    pub top_p: Option<f32>,
    pub min_p: Option<f32>,
    
    // Penalties
    pub repeat_penalty: Option<f32>,
    pub repeat_last_n: Option<i32>,
    pub frequency_penalty: Option<f32>,
    pub presence_penalty: Option<f32>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path))?;
        
        toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path))
    }
    
    pub fn get_transport(&self) -> String {
        self.transport.clone().unwrap_or_else(|| "uds".to_string())
    }
}

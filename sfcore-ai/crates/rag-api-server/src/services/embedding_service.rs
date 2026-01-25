use crate::config::EmbeddingConfig;
use crate::utils::error::ApiError;
use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::debug;
// Import trait from conversation manager
use crate::services::conversation::manager::EmbeddingProvider;

#[derive(Debug, Serialize)]
struct EmbeddingRequest {
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    input: Option<String>,
}

#[derive(Debug, Deserialize)]
struct EmbeddingResponse {
    embedding: Vec<f32>,
}

#[derive(Clone)] // Clone derives needed for Arc usage
pub struct EmbeddingService {
    client: Client,
    base_url: String,
    dimension: usize,
}

impl EmbeddingService {
    pub fn new(llm_base_url: String, config: EmbeddingConfig) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(60))
                .build()
                .unwrap_or_else(|_| Client::new()),
            base_url: llm_base_url,
            dimension: config.dimension,
        }
    }
    
    /// Generate embedding untuk single text (Existing Public API)
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>, ApiError> {
        self.embed_internal(text).await.map_err(|e| ApiError::LlmError(e.to_string()))
    }

    /// Internal method returning anyhow::Result
    async fn embed_internal(&self, text: &str) -> Result<Vec<f32>> {
        debug!("Generating embedding for {} chars", text.len());
        
        let request = EmbeddingRequest {
            content: text.to_string(),
            input: Some(text.to_string()), // Send both for compatibility
        };
        
        // Try /embedding first
        let url = format!("{}/embedding", self.base_url);
        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to connect to embedding server")?;
        
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Embedding API error ({}): {}", status, body);
        }
        
        let json_value: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse embedding response as JSON")?;
        
        // Robust parsing logic
        let embedding = if json_value.is_array() {
            // OpenAI format: [{"embedding": [...]}] or just [...]
            let arr = json_value.as_array().unwrap();
            if arr.is_empty() {
                anyhow::bail!("Empty array returned from embedding server");
            }
            
            if arr[0].is_object() && arr[0]["embedding"].is_array() {
                arr[0]["embedding"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter_map(|v: &serde_json::Value| v.as_f64().map(|f| f as f32))
                    .collect::<Vec<f32>>()
            } else {
                // Direct array of floats
                arr.iter()
                    .filter_map(|v: &serde_json::Value| v.as_f64().map(|f| f as f32))
                    .collect::<Vec<f32>>()
            }
        } else if json_value.is_object() && json_value["embedding"].is_array() {
            // Standard llama.cpp format: {"embedding": [...]}
            json_value["embedding"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(|v: &serde_json::Value| v.as_f64().map(|f| f as f32))
                .collect::<Vec<f32>>()
        } else if json_value.is_object() && json_value["data"].is_array() {
             // OpenAI data format: {"data": [{"embedding": [...]}]}
             let data = json_value["data"].as_array().unwrap();
             if !data.is_empty() && data[0]["embedding"].is_array() {
                 data[0]["embedding"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter_map(|v: &serde_json::Value| v.as_f64().map(|f| f as f32))
                    .collect::<Vec<f32>>()
             } else {
                 anyhow::bail!("Unrecognized embedding response format: {}", json_value);
             }
        } else {
            anyhow::bail!("Unrecognized embedding response format: {}", json_value);
        };
        
        if embedding.is_empty() {
            anyhow::bail!("Generated embedding is empty");
        }

        if embedding.len() != self.dimension {
            anyhow::bail!(
                "Embedding dimension mismatch: expected {}, got {}",
                self.dimension,
                embedding.len()
            );
        }
        
        Ok(embedding)
    }
    
    /// Generate embeddings untuk batch texts
    pub async fn embed_batch(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>, ApiError> {
        let mut embeddings = Vec::with_capacity(texts.len());
        
        for text in texts {
            let embedding = self.embed(&text).await?;
            embeddings.push(embedding);
        }
        
        Ok(embeddings)
    }

    /// Embed with weights (Internal logic for trait)
    async fn embed_weighted_internal(
        &self,
        current_text: &str,
        context_text: &str,
        current_weight: f32,
        history_weight: f32,
    ) -> Result<Vec<f32>> {
        // Embed current message
        let current_embedding = self.embed_internal(current_text).await?;
        
        // Embed full context (current + history)
        let context_embedding = self.embed_internal(context_text).await?;
        
        // Weighted average
        let weighted = current_embedding
            .iter()
            .zip(context_embedding.iter())
            .map(|(curr, ctx)| {
                current_weight * curr + history_weight * ctx
            })
            .collect();
        
        Ok(weighted)
    }
}

// Implement trait
#[async_trait::async_trait]
impl EmbeddingProvider for EmbeddingService {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        self.embed_internal(text).await
    }

    async fn embed_weighted(
        &self,
        current_text: &str,
        context_text: &str,
        current_weight: f32,
        history_weight: f32,
    ) -> Result<Vec<f32>> {
        self.embed_weighted_internal(current_text, context_text, current_weight, history_weight).await
    }
}

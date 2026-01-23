use crate::config::EmbeddingConfig;
use crate::utils::error::ApiError;
use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Debug, Serialize)]
struct EmbeddingRequest {
    content: String,
}

#[derive(Debug, Deserialize)]
struct EmbeddingResponse {
    embedding: Vec<f32>,
}

pub struct EmbeddingService {
    client: Client,
    base_url: String,
    dimension: usize,
}

impl EmbeddingService {
    pub fn new(llm_base_url: String, config: EmbeddingConfig) -> Self {
        Self {
            client: Client::new(),
            base_url: llm_base_url,
            dimension: config.dimension,
        }
    }
    
    /// Generate embedding untuk single text
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>, ApiError> {
        debug!("Generating embedding for {} chars", text.len());
        
        let request = EmbeddingRequest {
            content: text.to_string(),
        };
        
        let response = self
            .client
            .post(&format!("{}/embedding", self.base_url))
            .json(&request)
            .send()
            .await
            .map_err(|e| ApiError::LlmError(format!("Failed to call embedding API: {}", e)))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ApiError::LlmError(format!(
                "Embedding API error: {} - {}",
                status, body
            )));
        }
        
        let embedding_response: EmbeddingResponse = response
            .json()
            .await
            .map_err(|e| ApiError::LlmError(format!("Failed to parse embedding response: {}", e)))?;
        
        if embedding_response.embedding.len() != self.dimension {
            return Err(ApiError::LlmError(format!(
                "Embedding dimension mismatch: expected {}, got {}",
                self.dimension,
                embedding_response.embedding.len()
            )));
        }
        
        Ok(embedding_response.embedding)
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
}

use crate::config::EmbeddingConfig;
use crate::utils::error::ApiError;
use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::debug;
// Import trait from conversation manager
use crate::services::conversation::manager::EmbeddingProvider;

#[derive(Debug, Serialize)]
struct EmbeddingRequest {
    content: String, // "content" or "input"? Existing uses "content". Previous fix code used "input" for v1/embeddings compatible?
    // My existing code (Step 807) used "content". I'll checks config/settings or logs.
    // Existing code:
    // struct EmbeddingRequest { content: String }
    // url: base_url + "/embedding"
    //
    // The Fix code (Step 7) uses:
    // struct EmbedRequest { input: String }
    // url: base_url + "/v1/embeddings"
    //
    // This suggests I should perhaps support standard OpenAI format or existing format.
    // The user's prompt implies "rag-api-server" existing custom logic?
    // The "Fix" code might be generic.
    // I should stick to EXISTING logic ("content", "/embedding") unless Fix implies changing LLM backend compatibility.
    // The settings.toml has [llm] base_url...
    // I'll stick to EXISTING logic to avoid breaking connection with `rag-embedding-worker` or whatever LLM server.
    // I will keep existing struct/logic but wrap in trait.
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
            client: Client::new(),
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
            anyhow::bail!("Embedding API error: {} - {}", status, body);
        }
        
        let embedding_response: EmbeddingResponse = response
            .json()
            .await?;
        
        if embedding_response.embedding.len() != self.dimension {
            anyhow::bail!(
                "Embedding dimension mismatch: expected {}, got {}",
                self.dimension,
                embedding_response.embedding.len()
            );
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

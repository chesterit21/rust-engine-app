/// embedding_service.rs

use crate::config::EmbeddingConfig;
use crate::utils::error::ApiError;
use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::debug;
// Import trait from conversation manager
use crate::services::conversation::manager::EmbeddingProvider;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Serialize)]
struct EmbeddingRequest {
    input: String,
    model: String,
}

#[derive(Debug, Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
}

#[derive(Debug, Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
}

use crate::utils::limiters::Limiters;
use std::time::Instant;

#[derive(Clone)] // Clone derives needed for Arc usage
pub struct EmbeddingService {
    client: Client,
    base_url: String,
    pub dimension: usize,
    model_name: String,
    cache: Arc<RwLock<HashMap<String, Vec<f32>>>>, // Cache embeddings
    limiters: Arc<Limiters>, // NEW
    batch_size: usize, // NEW
    api_key: Option<String>, // NEW
}

impl EmbeddingService {
    pub fn new(llm_base_url: String, config: EmbeddingConfig, limiters: Arc<Limiters>, batch_size: usize) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(60))
                .build()
                .unwrap_or_else(|_| Client::new()),
            base_url: llm_base_url,
            dimension: config.dimension,
            model_name: config.model,
            cache: Arc::new(RwLock::new(HashMap::new())),
            limiters, // NEW
            batch_size,
            api_key: config.api_key,
        }
    }
    
    /// Generate embedding untuk single text (Existing Public API)
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>, ApiError> {
        self.embed_internal(text).await.map_err(|e| ApiError::LlmError(e.to_string()))
    }

    /// Internal method returning anyhow::Result
    async fn embed_internal(&self, text: &str) -> Result<Vec<f32>> {
        // 1. Check Cache
        {
            let cache = self.cache.read().await;
            if let Some(embedding) = cache.get(text) {
                debug!("Cache HIT for embedding ({:.20}...) - skipping API call", text);
                return Ok(embedding.clone());
            }
        }

        // 2. Limiter acquire (only on cache MISS)
        let (_permit, wait) = Limiters::acquire_timed(
            self.limiters.embedding.clone(),
            self.limiters.acquire_timeout,
            "embedding",
        )
        .await?;

        debug!(wait_ms = wait.as_millis() as u64, op = "embedding", "wait_queue");

        let exec_start = Instant::now();

        debug!("Generating embedding for {} chars using model {}", text.len(), self.model_name);
        
        let request = EmbeddingRequest {
            input: text.to_string(),
            model: self.model_name.clone(),
        };
        
        // Use standard /v1/embeddings endpoint
        let url = format!("{}/v1/embeddings", self.base_url);
        
        let mut request_builder = self.client.post(&url);
        
        if let Some(key) = &self.api_key {
            request_builder = request_builder.header("Authorization", format!("Bearer {}", key));
        }

        let response = request_builder
            .json(&request)
            .send()
            .await
            .context("Failed to connect to embedding server")?;
         
        debug!(exec_ms = exec_start.elapsed().as_millis() as u64, op = "embedding", "exec");
   
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Embedding API error ({}): {}", status, body);
        }
        
        // Parse standard OpenAI response format
        let body_text = response.text().await.context("Failed to read response body")?;
        
        // DEBUG: Log the raw response to see what Llama Server is sending
        // Truncate if too long to avoid huge logs, but keep enough to see structure
        let debug_body = if body_text.len() > 500 {
            format!("{}...", &body_text[..500]) 
        } else {
            body_text.clone()
        };
        debug!("Raw Embedding Response: {}", debug_body);

        let response_body: EmbeddingResponse = serde_json::from_str(&body_text)
            .context(format!("Failed to parse embedding response (expected OpenAI format). Raw: {}", debug_body))?;
            
        if response_body.data.is_empty() {
            anyhow::bail!("Empty data array returned from embedding server");
        }
        
        let embedding = &response_body.data[0].embedding;
        
        if embedding.is_empty() {
            anyhow::bail!("Generated embedding vector is empty");
        }

        if embedding.len() != self.dimension {
            anyhow::bail!(
                "Embedding dimension mismatch: expected {}, got {}",
                self.dimension,
                embedding.len()
            );
        }

        // 3. Store in Cache
        {
            let mut cache = self.cache.write().await;
            cache.insert(text.to_string(), embedding.clone());
        }
        
        Ok(embedding.clone())
    }
    
    /// Generate embeddings untuk batch texts (Parallel Optimized)
    /// Generate embeddings untuk batch texts (Serialized Batching)
    pub async fn embed_batch(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>, ApiError> {
        use futures::future::join_all;
        
        debug!("Generating batch embeddings for {} texts (batch_size={})", texts.len(), self.batch_size);

        let mut all_results = Vec::with_capacity(texts.len());
        
        // Process in chunks (serial batches) to prevent semaphore flooding
        for chunk_batch in texts.chunks(self.batch_size) {
            let futures: Vec<_> = chunk_batch.iter()
                .map(|text| {
                    let service = self.clone();
                    let t = text.clone();
                    async move {
                        service.embed(&t).await
                    }
                })
                .collect();
            
            let results = join_all(futures).await;
            
            // If any error, bail
            for res in results {
                match res {
                    Ok(emb) => all_results.push(emb),
                    Err(e) => return Err(e),
                }
            }
        }
        
        Ok(all_results)
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

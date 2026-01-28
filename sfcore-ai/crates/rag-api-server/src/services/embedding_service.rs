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
    input: Vec<String>,
    model: String,
}

#[derive(Debug, Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
}

#[derive(Debug, Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
    index: Option<usize>, // Add index for safety sorting if needed
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
        // Optimization: Use the batch logic for single item to unify code path
        // parse result from vector
        let res = self.embed_batch(vec![text.to_string()]).await?;
        res.into_iter().next().ok_or_else(|| ApiError::LlmError("No embedding returned".to_string()))
    }

    /// Internal method returning anyhow::Result (Legacy wrapper if needed, but we used embed_batch now)
    async fn embed_internal(&self, text: &str) -> Result<Vec<f32>> {
         // This is now redundant if we redirect to embed_batch, but keeping for compatibility if any internal call uses it specific logic.
         // Let's implement it via direct request to match previous logic but with Array payload.
         
         // 1. Check Cache
        {
            let cache = self.cache.read().await;
            if let Some(embedding) = cache.get(text) {
                debug!("Cache HIT for embedding ({:.20}...) - skipping API call", text);
                return Ok(embedding.clone());
            }
        }
        
        let batch_res = self.send_embedding_request(vec![text.to_string()]).await?;
        let embedding = batch_res.into_iter().next().ok_or_else(|| anyhow::anyhow!("No data returned"))?;
        
        // 3. Store in Cache
        {
            let mut cache = self.cache.write().await;
            cache.insert(text.to_string(), embedding.clone());
        }
        
        Ok(embedding)
    }
    
    /// Helper to send actual HTTP request
    async fn send_embedding_request(&self, inputs: Vec<String>) -> Result<Vec<Vec<f32>>> {
         // 2. Limiter acquire
        let (_permit, wait) = Limiters::acquire_timed(
            self.limiters.embedding.clone(),
            self.limiters.acquire_timeout,
            "embedding",
        )
        .await?;

        debug!(wait_ms = wait.as_millis() as u64, op = "embedding", "wait_queue");
        let exec_start = Instant::now();

        let request = EmbeddingRequest {
            input: inputs,
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
        
        let body_text = response.text().await.context("Failed to read response body")?;
        
        let response_body: EmbeddingResponse = serde_json::from_str(&body_text)
            .context(format!("Failed to parse embedding response. Data sample: {:.200}", body_text))?;
            
        if response_body.data.is_empty() {
             return Ok(vec![]);
        }
        
        // Map data. Sort by index if present? Usually returned in order.
        let results: Vec<Vec<f32>> = response_body.data.into_iter().map(|d| d.embedding).collect();
        
        if results.len() != request.input.len() {
             // Warn?
             debug!("Requested {} embeddings, got {}", request.input.len(), results.len());
        }
        
        Ok(results)
    }

    /// Generate embeddings untuk batch texts (True Batching)
    pub async fn embed_batch(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>, ApiError> {
        if texts.is_empty() {
            return Ok(vec![]);
        }

        debug!("Generating batch embeddings for {} texts", texts.len());
        
        // Check cache for ALL items first?
        // Implementing partial cache hit logic is complex for batching (need to only request missing).
        // For now, let's just send the request. Optimized cache later.
        
        // Split into chunks if texts > self.batch_size (safety)
        // Note: document_service already batches, but this is a library method.
        let mut all_results = Vec::with_capacity(texts.len());
        
        for chunk_batch in texts.chunks(self.batch_size) {
             let chunk_vec = chunk_batch.to_vec();
             let chunk_results = self.send_embedding_request(chunk_vec).await
                 .map_err(|e| ApiError::LlmError(e.to_string()))?;
             all_results.extend(chunk_results);
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

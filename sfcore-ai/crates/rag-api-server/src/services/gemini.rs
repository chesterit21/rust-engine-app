use crate::config::GeminiConfig;
use crate::utils::error::ApiError;
use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, error};

// Re-use limiters if needed, or build new ones.
// We will reuse the global limiters for concurrency safety.
use crate::utils::limiters::Limiters;

#[derive(Clone)]
pub struct GeminiService {
    client: Client,
    config: GeminiConfig,
    limiters: Arc<Limiters>,
}

#[derive(Serialize)]
struct GeminiEmbeddingRequest {
    model: String,
    content: GeminiContent,
}

#[derive(Serialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Serialize)]
struct GeminiPart {
    text: String,
}

// Minimal OpenAI-compatible Request (since we use the v1beta/openai endpoint)
#[derive(Serialize)]
struct OpenAiEmbeddingRequest {
    input: String,
    model: String,
}

#[derive(Serialize)]
struct OpenAiChatRequest {
    model: String,
    messages: Vec<OpenAiMessage>,
    stream: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OpenAiMessage {
    pub role: String,
    pub content: String,
}

// Response Structures
#[derive(Deserialize)]
struct OpenAiEmbeddingResponse {
    data: Vec<OpenAiEmbeddingData>,
}

#[derive(Deserialize)]
struct OpenAiEmbeddingData {
    embedding: Vec<f32>,
}

impl GeminiService {
    pub fn new(config: GeminiConfig, limiters: Arc<Limiters>) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(60))
                .build()
                .unwrap_or_else(|_| Client::new()),
            config,
            limiters,
        }
    }

    /// Generate Embedding using Gemini (via OpenAI Compatible Endpoint)
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>, ApiError> {
        // Acquire Limiter
        let (_permit, _wait) = Limiters::acquire_timed(
            self.limiters.embedding.clone(),
            self.limiters.acquire_timeout,
            "gemini_embedding",
        )
        .await
        .map_err(|e| ApiError::LlmError(e.to_string()))?;

        let url = "https://generativelanguage.googleapis.com/v1beta/openai/embeddings";
        
        let request = OpenAiEmbeddingRequest {
            input: text.to_string(),
            model: "text-embedding-004".to_string(),
        };

        let response = self.client.post(url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| ApiError::LlmError(format!("Gemini Network Error: {}", e)))?;

        if !response.status().is_success() {
             let status = response.status();
             let text = response.text().await.unwrap_or_default();
             return Err(ApiError::LlmError(format!("Gemini API Error ({}): {}", status, text)));
        }

        let body: OpenAiEmbeddingResponse = response.json().await
            .map_err(|e| ApiError::LlmError(format!("Failed to parse Gemini Embedding: {}", e)))?;

        if let Some(data) = body.data.first() {
            Ok(data.embedding.clone())
        } else {
            Err(ApiError::LlmError("Gemini returned no embedding data".to_string()))
        }
    }

    /// Generate Chat Completion (Stream)
    // NOTE: For simplicity in this specific class, we might return a raw stream or reuse the logic.
    // Given 'Strict Separation', let's implement a direct method.
    pub async fn chat_stream(
        &self,
        messages: Vec<OpenAiMessage>,
    ) -> Result<reqwest::Response, ApiError> {
        let url = "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions";
        
        let request = OpenAiChatRequest {
            model: "gemini-1.5-flash".to_string(),
            messages,
            stream: true,
        };

        let response = self.client.post(url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| ApiError::LlmError(format!("Gemini Chat Error: {}", e)))?;

        if !response.status().is_success() {
             let status = response.status();
             let text = response.text().await.unwrap_or_default();
             return Err(ApiError::LlmError(format!("Gemini Chat API Error ({}): {}", status, text)));
        }

        Ok(response)
    }
}

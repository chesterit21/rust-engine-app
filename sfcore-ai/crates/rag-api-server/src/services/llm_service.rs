use crate::config::LlmConfig;
use crate::utils::error::ApiError;
use futures::stream::Stream;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use tracing::debug;

#[derive(Debug, Serialize)]
pub struct ChatCompletionRequest {
    pub messages: Vec<ChatMessage>,
    pub max_tokens: usize,
    pub temperature: f32,
    pub stream: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct ChatCompletionChunk {
    pub choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
pub struct Choice {
    pub delta: Delta,
}

#[derive(Debug, Deserialize)]
pub struct Delta {
    pub content: Option<String>,
}

pub struct LlmService {
    client: Client,
    config: LlmConfig,
}

impl LlmService {
    pub fn new(config: LlmConfig) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(config.timeout_seconds))
                .build()
                .expect("Failed to create HTTP client"),
            config,
        }
    }
    
    /// Generate completion dengan streaming
    pub async fn chat_stream(
        &self,
        messages: Vec<ChatMessage>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String, ApiError>> + Send>>, ApiError> {
        debug!("Starting chat stream with {} messages", messages.len());
        
        let request = ChatCompletionRequest {
            messages,
            max_tokens: self.config.max_tokens,
            temperature: 0.7,
            stream: true,
        };
        
        let response = self
            .client
            .post(&format!("{}/v1/chat/completions", self.config.base_url))
            .json(&request)
            .send()
            .await
            .map_err(|e| ApiError::LlmError(format!("Failed to call LLM API: {}", e)))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ApiError::LlmError(format!(
                "LLM API error: {} - {}",
                status, body
            )));
        }
        
        // Convert response stream to text stream
        let stream = response.bytes_stream();
        
        // Parse SSE stream
        let parsed_stream = futures::stream::unfold(stream, |mut stream| async move {
            use futures::StreamExt;
            
            match stream.next().await {
                Some(Ok(bytes)) => {
                    // Parse SSE format: "data: {...}\n\n"
                    let text = String::from_utf8_lossy(&bytes);
                    
                    for line in text.lines() {
                        if line.starts_with("data: ") {
                            let json_str = line.strip_prefix("data: ").unwrap_or("");
                            
                            if json_str == "[DONE]" {
                                return None;
                            }
                            
                            if let Ok(chunk) = serde_json::from_str::<ChatCompletionChunk>(json_str) {
                                if let Some(content) = chunk.choices.first()
                                    .and_then(|c| c.delta.content.as_ref())
                                {
                                    return Some((Ok(content.clone()), stream));
                                }
                            }
                        }
                    }
                    
                    Some((Ok(String::new()), stream))
                }
                Some(Err(e)) => {
                    Some((Err(ApiError::LlmError(format!("Stream error: {}", e))), stream))
                }
                None => None,
            }
        });
        
        Ok(Box::pin(parsed_stream))
    }
}

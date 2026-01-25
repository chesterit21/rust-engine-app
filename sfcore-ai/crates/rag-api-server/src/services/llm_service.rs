use crate::config::LlmConfig;
use crate::utils::error::ApiError;
use futures::stream::Stream;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use tracing::debug;
use anyhow::Result;

// Use shared ChatMessage
use crate::models::chat::ChatMessage;
// Import trait and types from manager
use crate::services::conversation::manager::{LlmProvider, RetrievalChunk};

#[derive(Debug, Serialize)]
pub struct ChatCompletionRequest {
    pub messages: Vec<ChatMessage>,
    pub max_tokens: usize,
    pub temperature: f32,
    pub stream: bool,
}

// Local response structs
#[derive(Debug, Deserialize)]
pub struct ChatCompletionChunk {
    pub choices: Vec<ChoiceChunk>,
}

#[derive(Debug, Deserialize)]
pub struct ChoiceChunk {
    pub delta: Delta,
}

#[derive(Debug, Deserialize)]
pub struct Delta {
    pub content: Option<String>,
}

#[derive(Clone)]
pub struct LlmService {
    client: Client,
    config: LlmConfig,
    context_extraction_system_prompt: String,
}

impl LlmService {
    pub fn new(config: LlmConfig, context_extraction_system_prompt: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(config.timeout_seconds))
                .build()
                .expect("Failed to create HTTP client"),
            config,
            context_extraction_system_prompt,
        }
    }
    
    /// Generate completion with streaming
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

    /// Generate completion without streaming (wait for full response)
    pub async fn generate_chat(
        &self,
        messages: Vec<ChatMessage>,
    ) -> Result<String, ApiError> {
        debug!("Starting chat generation with {} messages", messages.len());
        
        let request = ChatCompletionRequest {
            messages,
            max_tokens: self.config.max_tokens,
            temperature: 0.7,
            stream: false,
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
        
        #[derive(Deserialize)]
        struct ChatCompletionResponse {
            choices: Vec<Choice>,
        }
        #[derive(Deserialize)]
        struct Choice {
            message: Message,
        }
        #[derive(Deserialize)]
        struct Message {
            content: String,
        }
        
        let chat_response: ChatCompletionResponse = response
            .json()
            .await
            .map_err(|e| ApiError::LlmError(format!("Failed to parse LLM response: {}", e)))?;
            
        chat_response.choices.first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| ApiError::LlmError("No choices returned from LLM".to_string()))
    }
}

// Implement LlmProvider trait
#[async_trait::async_trait]
impl LlmProvider for LlmService {
    async fn generate(&self, messages: &[ChatMessage]) -> Result<String> {
        self.generate_chat(messages.to_vec())
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    async fn generate_stream(&self, messages: &[ChatMessage]) -> Result<Pin<Box<dyn Stream<Item = Result<String, anyhow::Error>> + Send>>> {
        use futures::StreamExt;
        
        let stream = self.chat_stream(messages.to_vec())
            .await
            .map_err(|e| anyhow::anyhow!(e))?;
            
        // Map stream items from ApiError to anyhow::Error
        let mapped_stream = stream.map(|item| {
            item.map_err(|e| anyhow::anyhow!(e))
        });
        
        Ok(Box::pin(mapped_stream))
    }

    async fn summarize_chunks(&self, chunks: &[RetrievalChunk], query: &str) -> Result<String> {
        if chunks.is_empty() {
            return Ok("No relevant documents found.".to_string());
        }

        // Build chunk text
        let chunks_text: String = chunks
            .iter()
            .enumerate()
            .map(|(i, chunk)| {
                format!(
                    "[Chunk {}]\nDocument Title: {}\nContent: {}\n",
                    i + 1,
                    chunk.document_title.as_deref().unwrap_or("Unknown"),
                    chunk.content
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        // Construct System Message with Chunks
        let system_message_content = self.context_extraction_system_prompt.replace("{{CHUNKS}}", &chunks_text);
        
        let messages = vec![
            ChatMessage { role: "system".to_string(), content: system_message_content },
            ChatMessage { role: "user".to_string(), content: query.to_string() },
        ];
        
        self.generate(&messages).await
    }
}

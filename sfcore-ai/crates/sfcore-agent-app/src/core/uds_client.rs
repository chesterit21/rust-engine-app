//! UDS Client for SFCore AI Server
//!
//! Communicates with the inference server via Unix Domain Socket using Tokio.

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

/// Chat message structure (matches server protocol)
#[derive(Debug, Serialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

/// Request to send to server (matches test_server.js format)
#[derive(Debug, Serialize)]
pub struct ChatRequest {
    pub messages: Vec<Message>,
    pub stream: bool,
    pub max_tokens: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
}

/// Response from server
/// Server sends two types:
/// 1. StreamChunk: {"token": "..."}
/// 2. FinalResponse: {"output": "...", "done": true, "metrics": {...}}
#[derive(Debug, Deserialize)]
pub struct ChatResponse {
    // Streaming chunk field
    pub token: Option<String>,
    // Final response fields
    pub output: Option<String>,
    pub done: Option<bool>,
    pub error: Option<String>,
    pub metrics: Option<ResponseMetrics>,
}

#[derive(Debug, Deserialize)]
pub struct ResponseMetrics {
    pub tokens_generated: i32,
    pub speed_tokens_sec: f32,
    pub total_time_ms: u128,
}

/// UDS Client for communicating with server (Async)
pub struct UdsClient {
    socket_path: String,
}

impl UdsClient {
    pub fn new(socket_path: &str) -> Self {
        Self {
            socket_path: socket_path.to_string(),
        }
    }

    /// Send chat request and return a channel for streaming tokens
    pub async fn stream_chat(
        &self,
        prompt: &str,
        max_tokens: i32,
        tx: tokio::sync::mpsc::UnboundedSender<String>,
    ) -> Result<(), String> {
        let mut stream = UnixStream::connect(&self.socket_path)
            .await
            .map_err(|e| format!("Failed to connect to {}: {}", self.socket_path, e))?;

        // Build request with messages array (matching server protocol)
        let request = ChatRequest {
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: "Kamu adalah asisten AI yang membantu.".to_string(),
                },
                Message {
                    role: "user".to_string(),
                    content: prompt.to_string(),
                },
            ],
            stream: true,
            max_tokens,
            temperature: Some(0.7),
        };

        let json = serde_json::to_string(&request)
            .map_err(|e| format!("Failed to serialize: {}", e))?;

        stream
            .write_all(json.as_bytes())
            .await
            .map_err(|e| format!("Failed to write request: {}", e))?;
            
        stream.write_all(b"\n").await.map_err(|e| format!("Failed to write newline: {}", e))?;
        stream.flush().await.map_err(|e| format!("Flush failed: {}", e))?;

        let stream_reader = BufReader::new(stream);
        let mut lines = stream_reader.lines();

        while let Some(line) = lines.next_line().await.map_err(|e| format!("Read failed: {}", e))? {
             let response: ChatResponse = serde_json::from_str(&line)
                .map_err(|e| format!("Parse failed: {} (line: {})", e, line))?;

            if let Some(err) = response.error {
                return Err(err);
            }

            // Handle streaming chunk (token field)
            if let Some(token) = response.token {
                let _ = tx.send(token);
            }
            
            // Handle final response (output field - usually empty in streaming)
            if let Some(output) = response.output {
                if !output.is_empty() {
                    let _ = tx.send(output);
                }
            }

            if response.done.unwrap_or(false) {
                break;
            }
        }

        Ok(())
    }
}

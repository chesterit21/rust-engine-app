use anyhow::{anyhow, Result};
use log::{error, info};
use serde::{Deserialize, Serialize};
use sfcore_ai_engine::{LlamaCppEngine, LlamaCppOptions, ChatMessage};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::sync::mpsc;

/// Request payload from Client
#[derive(Debug, Deserialize)]
pub struct GenerateRequest {
    pub prompt: Option<String>,
    pub messages: Option<Vec<ChatMessage>>,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: i32,
    #[serde(default)]
    pub stream: bool,
    
    // Optional overrides
    pub temperature: Option<f32>,
}

fn default_max_tokens() -> i32 {
    1024
}

/// Streaming Response chunk
#[derive(Debug, Serialize)]
pub struct StreamChunk {
    pub token: String,
}

/// Final Response (Streaming & Non-Streaming)
#[derive(Debug, Serialize)]
pub struct FinalResponse {
    pub output: String, // Full text (empty/partial if streaming? No, full text even in streaming is useful for logs)
    pub done: bool,
    pub metrics: Metrics,
}

#[derive(Debug, Serialize, Clone)]
pub struct Metrics {
    pub tokens_generated: i32,
    pub speed_tokens_sec: f32,
    pub total_time_ms: u128,
}

/// Handle a single connection
pub async fn handle_connection(mut stream: UnixStream, engine: Arc<LlamaCppEngine>) -> Result<()> {
    let (reader, mut writer) = stream.split();
    let mut buf_reader = BufReader::new(reader);
    let mut line = String::new();

    // Read loop for persistent connection
    while buf_reader.read_line(&mut line).await? > 0 {
        let req_str = line.trim();
        if req_str.is_empty() {
            line.clear();
            continue;
        }

        info!("Request: {}...", &req_str.chars().take(50).collect::<String>());

        let req: GenerateRequest = match serde_json::from_str(req_str) {
            Ok(r) => r,
            Err(e) => {
                let err_msg = format!("{{\"error\": \"Invalid JSON: {}\"}}\n", e);
                writer.write_all(err_msg.as_bytes()).await?;
                writer.flush().await?;
                line.clear();
                continue;
            }
        };

        let engine_clone = engine.clone();
        
        // Resolve prompt from 'messages' (template) OR 'prompt' (raw)
        let prompt = if let Some(msgs) = &req.messages {
            match engine.apply_chat_template(msgs) {
                Ok(s) => s,
                Err(e) => {
                    let err_msg = format!("{{\"error\": \"Template error: {}\"}}\n", e);
                    writer.write_all(err_msg.as_bytes()).await?;
                    writer.flush().await?;
                    line.clear();
                    continue;
                }
            }
        } else if let Some(p) = req.prompt {
            p
        } else {
             let err_msg = "{\"error\": \"Missing 'prompt' or 'messages'\"}\n";
             writer.write_all(err_msg.as_bytes()).await?;
             writer.flush().await?;
             line.clear();
             continue;
        };

        let max_tokens = req.max_tokens;
        let is_stream = req.stream;

        // Channel for streaming tokens from Blocking Task -> Async Writer
        let (tx, mut rx) = mpsc::unbounded_channel::<String>();

        // Spawn inference in blocking thread
        let handle = tokio::task::spawn_blocking(move || {
            if is_stream {
                // Streaming Mode: Send tokens to channel
                engine_clone.generate_with_callback(&prompt, max_tokens, |token| {
                    let _ = tx.send(token);
                    true // continue
                })
            } else {
                // Non-Streaming: Just run without callback logic (empty callback)
                engine_clone.generate_with_callback(&prompt, max_tokens, |_| true)
            }
        });

        // Async loop to read stream and write to socket
        // Only run this loop if streaming is requested
        if is_stream {
            while let Some(token) = rx.recv().await {
                let chunk = StreamChunk { token };
                let json = serde_json::to_string(&chunk)?;
                writer.write_all(json.as_bytes()).await?;
                writer.write_all(b"\n").await?;
                writer.flush().await?;
            }
        }

        // Wait for inference to finish and get final result/metrics
        let gen_result = match handle.await {
            Ok(Ok(res)) => res,
            Ok(Err(e)) => {
                let err = format!("{{\"error\": \"Inference failed: {}\"}}\n", e);
                writer.write_all(err.as_bytes()).await?;
                writer.flush().await?;
                line.clear();
                continue;
            }
            Err(e) => {
                let err = format!("{{\"error\": \"Task panicked: {}\"}}\n", e);
                writer.write_all(err.as_bytes()).await?;
                writer.flush().await?;
                line.clear();
                continue;
            }
        };

        // Send Final Response
        // Note: For streaming, this is the "done" message.
        // For non-streaming, this is the ONLY message (containing full output).
        let final_resp = FinalResponse {
            output: if is_stream { String::new() } else { gen_result.output },
            done: true,
            metrics: Metrics {
                tokens_generated: gen_result.tokens_generated,
                speed_tokens_sec: gen_result.tokens_per_sec,
                total_time_ms: gen_result.total_ms,
            },
        };

        let json = serde_json::to_string(&final_resp)?;
        writer.write_all(json.as_bytes()).await?;
        writer.write_all(b"\n").await?;
        writer.flush().await?;

        line.clear();
    }

    Ok(())
}

//! HTTP-SSE Transport Handler

use crate::error::{AppError, ErrorResponse};
use axum::{
    extract::State,
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Response,
    },
    Json,
};
use log::{error, info};
use serde::{Deserialize, Serialize};
use sfcore_ai_engine::{ChatMessage, LlamaCppEngine};
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

/// Request payload (same as UDS handler)
#[derive(Debug, Deserialize)]
pub struct GenerateRequest {
    pub prompt: Option<String>,
    pub messages: Option<Vec<ChatMessage>>,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: i32,
    #[serde(default)]
    pub stream: bool,
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

/// Final Response
#[derive(Debug, Serialize)]
pub struct FinalResponse {
    pub output: String,
    pub done: bool,
    pub metrics: Metrics,
}

#[derive(Debug, Serialize, Clone)]
pub struct Metrics {
    pub tokens_generated: i32,
    pub speed_tokens_sec: f32,
    pub total_time_ms: u128,
}

/// Shared app state
#[derive(Clone)]
pub struct AppState {
    pub engine: Arc<LlamaCppEngine>,
}

/// Main inference handler
pub async fn inference_handler(
    State(state): State<AppState>,
    Json(req): Json<GenerateRequest>,
) -> Result<Response, AppError> {
    info!("Inference request - stream: {}", req.stream);
    
    // Resolve prompt from messages or prompt
    let prompt = if let Some(msgs) = &req.messages {
        state
            .engine
            .apply_chat_template(msgs)
            .map_err(|e| {
                error!("Template error: {}", e);
                AppError::BadRequest(ErrorResponse::invalid_request(
                    format!("Template error: {}", e),
                ))
            })?
    } else if let Some(p) = &req.prompt {
        p.clone()
    } else {
        return Err(AppError::BadRequest(ErrorResponse::invalid_request(
            "Missing 'prompt' or 'messages'",
        )));
    };
    
    let max_tokens = req.max_tokens;
    let is_stream = req.stream;
    
    if is_stream {
        // Return SSE stream
        let stream = create_sse_stream(state.engine.clone(), prompt, max_tokens);
        Ok(Sse::new(stream)
            .keep_alive(KeepAlive::default())
            .into_response())
    } else {
        // Return JSON
        let result = run_blocking_inference(state.engine.clone(), prompt, max_tokens).await?;
        Ok(Json(result).into_response())
    }
}

/// Create SSE stream - FIXED: Return Result<Event, Infallible>
fn create_sse_stream(
    engine: Arc<LlamaCppEngine>,
    prompt: String,
    max_tokens: i32,
) -> impl tokio_stream::Stream<Item = Result<Event, Infallible>> {
    let (tx, rx) = mpsc::unbounded_channel::<Result<Event, Infallible>>();
    
    tokio::task::spawn_blocking(move || {
        // Run inference with callback
        let result = engine.generate_with_callback(&prompt, max_tokens, |token| {
            let chunk = StreamChunk { token };
            if let Ok(json) = serde_json::to_string(&chunk) {
                let event = Event::default().data(json);
                let _ = tx.send(Ok(event)); // Wrap in Ok()
            }
            true // continue
        });
        
        // Send final metrics
        match result {
            Ok(gen_result) => {
                let final_resp = FinalResponse {
                    output: String::new(), // Already streamed
                    done: true,
                    metrics: Metrics {
                        tokens_generated: gen_result.tokens_generated,
                        speed_tokens_sec: gen_result.tokens_per_sec,
                        total_time_ms: gen_result.total_ms,
                    },
                };
                
                if let Ok(json) = serde_json::to_string(&final_resp) {
                    let event = Event::default().data(json);
                    let _ = tx.send(Ok(event)); // Wrap in Ok()
                }
            }
            Err(e) => {
                error!("Inference error: {}", e);
                let err = ErrorResponse::server_error(format!("Inference failed: {}", e));
                if let Ok(json) = serde_json::to_string(&err) {
                    let event = Event::default().data(json);
                    let _ = tx.send(Ok(event)); // Wrap in Ok()
                }
            }
        }
    });
    
    UnboundedReceiverStream::new(rx)
}

/// Run blocking inference (non-streaming)
async fn run_blocking_inference(
    engine: Arc<LlamaCppEngine>,
    prompt: String,
    max_tokens: i32,
) -> Result<FinalResponse, AppError> {
    let handle = tokio::task::spawn_blocking(move || {
        engine.generate_with_callback(&prompt, max_tokens, |_| true)
    });
    
    let gen_result = handle
        .await
        .map_err(|e| {
            error!("Task panicked: {}", e);
            AppError::InternalError(ErrorResponse::server_error("Task panicked"))
        })?
        .map_err(|e| {
            error!("Inference failed: {}", e);
            AppError::InternalError(ErrorResponse::server_error(format!("Inference failed: {}", e)))
        })?;
    
    Ok(FinalResponse {
        output: gen_result.output,
        done: true,
        metrics: Metrics {
            tokens_generated: gen_result.tokens_generated,
            speed_tokens_sec: gen_result.tokens_per_sec,
            total_time_ms: gen_result.total_ms,
        },
    })
}

/// Health check endpoint
pub async fn health_handler() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

use axum::{
    extract::{State, Json},
    response::sse::{Event, KeepAlive, Sse},
};
use tracing::{info, error};
use std::sync::Arc;
use chrono::Utc;
use futures::stream::{self, Stream};
use std::convert::Infallible;

use crate::models::chat::{ChatRequest, ChatResponse, StreamEvent, NewSessionRequest, NewSessionResponse, SourceInfo};
use crate::services::conversation::ConversationManager;
use crate::services::event_bus::{SessionEvent, SystemEvent};
use crate::handlers::search::DocumentInfo;
use crate::utils::error::ApiError;
use crate::state::AppState;
use crate::logging::{ActivityLog, ActivityType, ActivityStatus};
use axum::extract::Query;

/// Handle streaming chat request
/// POST /api/chat/stream
use crate::services::conversation::manager::ChatStreamChunk;

pub async fn chat_stream_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ChatRequest>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, (axum::http::StatusCode, String)> {
    // Log full payload as JSON for debugging
    if let Ok(json_payload) = serde_json::to_string(&req) {
         info!("Incoming chat payload: {}", json_payload);
    } else {
         info!(?req, "Incoming chat request (failed to serialize json)");
    }

    // Validate request
    if req.message.trim().is_empty() {
        return Err((
            axum::http::StatusCode::BAD_REQUEST,
            "Message cannot be empty".to_string(),
        ));
    }

    let conversation_manager = state.conversation_manager.clone();
    let session_id = req.session_id;
    let user_id = req.user_id;
    let message = req.message.clone();
    
    // Backward compatible + NEW multi-doc
    let document_id = req.document_id;
    let document_ids = req.document_ids.clone();
    
    let request_id = format!("{}-{}-{}", session_id, user_id, Utc::now().timestamp_millis());
    
    // Create SSE stream
    let response_stream = async_stream::stream! {
        // Execute manager logic which now returns a Stream
        match conversation_manager
            .handle_message(session_id, user_id, message, document_id, document_ids, request_id.clone())
            .await
        {
            Ok(mut logic_stream) => {
                use futures::StreamExt;
                
                // Forward chunks as they arrive
                while let Some(chunk_res) = logic_stream.next().await {
                    match chunk_res {
                        Ok(chunk) => match chunk {
                            ChatStreamChunk::Stage { request_id, phase, progress, text, detail } => {
                                let data = serde_json::to_string(&serde_json::json!({
                                    "request_id": request_id,
                                    "phase": phase,
                                    "progress": progress,
                                    "text": text,
                                    "detail": detail
                                })).unwrap_or_else(|_| "{}".to_string());

                                yield Ok(Event::default().event("stage").data(data));
                            }
                            ChatStreamChunk::Message { request_id, delta } => {
                                let data = serde_json::to_string(&serde_json::json!({
                                    "request_id": request_id,
                                    "delta": delta
                                })).unwrap_or_else(|_| "{}".to_string());

                                yield Ok(Event::default().event("message").data(data));
                            }
                            ChatStreamChunk::Done { request_id } => {
                                let data = serde_json::to_string(&serde_json::json!({
                                    "request_id": request_id,
                                    "final": true
                                })).unwrap_or_else(|_| "{}".to_string());

                                yield Ok(Event::default().event("done").data(data));
                            }
                            ChatStreamChunk::Error { message } => {
                                let data = serde_json::to_string(&serde_json::json!({
                                    "message": message
                                })).unwrap_or_else(|_| "{}".to_string());
                                yield Ok(Event::default().event("error").data(data));
                            }
                        },
                        Err(e) => {
                            error!("Stream error: {}", e);
                            let data = serde_json::to_string(&serde_json::json!({
                                "request_id": request_id,
                                "message": e.to_string()
                            })).unwrap_or_else(|_| "{}".to_string());

                            yield Ok(Event::default().event("error").data(data));
                        }
                    }
                }
            }
            Err(e) => {
                error!("Error handling message: {}", e);
                let data = serde_json::to_string(&serde_json::json!({
                    "request_id": request_id,
                    "message": e.to_string()
                })).unwrap_or_else(|_| "{}".to_string());

                yield Ok(Event::default().event("error").data(data));
            }
        }
    };

    Ok(Sse::new(response_stream).keep_alive(KeepAlive::default()))
}

pub async fn new_session_handler(
    Json(req): Json<NewSessionRequest>,
) -> Result<Json<NewSessionResponse>, (axum::http::StatusCode, String)> {
    // Generate identifier using i32 casting if needed by manager helper?
    // Manager::generate_session_id signature in my manager.rs was: fn generate_session_id(user_id: i32) -> SessionId
    // So I must cast.
    let session_id = ConversationManager::generate_session_id(req.user_id);
    
    info!("Generated new session ID {} for user {}", session_id, req.user_id);
    
    Ok(Json(NewSessionResponse { session_id }))
}

/// Get conversation cache statistics
/// GET /api/chat/stats
#[derive(serde::Serialize)]
pub struct CacheStatsResponse {
    pub active_sessions: usize,
    pub memory_usage_mb: u64,
    pub memory_total_mb: u64,
    pub memory_usage_percent: f64,
}

pub async fn cache_stats_handler(
    State(state): State<Arc<AppState>>,
) -> Json<CacheStatsResponse> {
    let stats = state.conversation_manager.cache_stats();
    
    Json(CacheStatsResponse {
        active_sessions: stats.active_sessions,
        memory_usage_mb: stats.memory_usage_mb,
        memory_total_mb: stats.memory_total_mb,
        memory_usage_percent: stats.memory_usage_percent,
    })
}

/// Manual cleanup of expired sessions
/// POST /api/chat/cleanup
#[derive(serde::Serialize)]
pub struct CleanupResponse {
    pub sessions_removed: usize,
}

pub async fn cleanup_sessions_handler(
    State(state): State<Arc<AppState>>,
) -> Json<CleanupResponse> {
    let count = state.conversation_manager.cleanup_expired_sessions();
    
    info!("Manual cleanup removed {} expired sessions", count);
    
    Json(CleanupResponse {
        sessions_removed: count,
    })
}

/// Get logging queue statistics
#[derive(serde::Serialize)]
pub struct LoggerStatsResponse {
    pub queue_length: usize,
    pub is_full: bool,
}

pub async fn logger_stats_handler(
    State(state): State<Arc<AppState>>,
) -> Json<LoggerStatsResponse> {
    let logger = &state.conversation_manager.logger();
    
    Json(LoggerStatsResponse {
        queue_length: logger.queue_len(),
        is_full: logger.is_queue_full(),
    })
}

/// Initialize chat session and fetch documents
/// POST /api/chat/init
#[derive(serde::Deserialize)]
pub struct ChatInitRequest {
    pub user_id: i32,
    pub session_id: Option<i64>,
}

#[derive(serde::Serialize)]
pub struct ChatInitResponse {
    pub session_id: i64,
    pub documents: Vec<DocumentInfo>,
    pub processing_docs: Vec<crate::database::DocumentProcessingStatus>,
}

pub async fn init_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ChatInitRequest>,
) -> Result<Json<ChatInitResponse>, ApiError> {
    info!("Chat init request from user {}", req.user_id);

    // 1. Get or Generate Session ID
    let session_id = req.session_id.unwrap_or_else(|| {
        ConversationManager::generate_session_id(req.user_id as i64)
    });

    // 2. Fetch Document List
    let repository = crate::database::Repository::new(state.db_pool.clone());
    let docs = repository
        .get_user_documents(req.user_id)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    let documents: Vec<DocumentInfo> = docs
        .into_iter()
        .map(|doc| DocumentInfo {
            document_id: doc.document_id,
            title: doc.document_title,
            owner_user_id: doc.owner_user_id,
            permission_level: doc.permission_level,
            created_at: doc.created_at.to_rfc3339(),
        })
        .collect();

    // 3. Fetch In-Progress Documents (Phase 2 Resilience)
    let processing_docs = repository
        .get_user_processing_documents(req.user_id)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    info!("Initialized session {} with {} docs and {} in-progress docs for user {}", 
        session_id, documents.len(), processing_docs.len(), req.user_id);

    Ok(Json(ChatInitResponse {
        session_id,
        documents,
        processing_docs,
    }))
}

/// Persistent SSE stream for session events
/// GET /api/chat/events
#[derive(serde::Deserialize)]
pub struct EventsParams {
    pub session_id: i64,
}

pub async fn events_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<EventsParams>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let session_id = params.session_id;
    let rx = state.event_bus.subscribe();

    let sse_stream = stream::unfold(rx, move |mut rx: tokio::sync::broadcast::Receiver<SessionEvent>| async move {
        loop {
            match rx.recv().await {
                Ok(session_event) => {
                    if session_event.session_id == session_id {
                        let data = serde_json::to_string(&session_event.event).unwrap_or_default();
                        let event = Event::default()
                            .event("system_event")
                            .data(data);
                        return Some((Ok(event), rx));
                    }
                    // Continue loop if not our session
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                    // Send error or skip? Let's skip and keep going
                    continue;
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                    return None;
                }
            }
        }
    });

    Sse::new(sse_stream).keep_alive(KeepAlive::default())
}

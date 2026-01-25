use axum::{
    extract::State,
    response::sse::{Event, KeepAlive, Sse},
    Json,
};
use futures::stream::{self, Stream};
use std::convert::Infallible;
use std::sync::Arc;
use tracing::{error, info};

use crate::handlers::search::DocumentInfo;
use crate::models::chat::ChatRequest;
use crate::services::conversation::ConversationManager;
use crate::services::event_bus::{SessionEvent, SystemEvent};
use crate::state::AppState;
use crate::utils::error::ApiError;
use axum::extract::Query;

/// Handle streaming chat request
/// POST /api/chat/stream
pub async fn chat_stream_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ChatRequest>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, (axum::http::StatusCode, String)> {
    info!(
        "Chat stream request: session_id={}, user_id={}, document_id={:?}",
        req.session_id, req.user_id, req.document_id
    );

    // Validate request
    if req.message.trim().is_empty() {
        return Err((
            axum::http::StatusCode::BAD_REQUEST,
            "Message cannot be empty".to_string(),
        ));
    }

    // Clone for async move
    let conversation_manager = state.conversation_manager.clone();
    let session_id = req.session_id;
    let user_id = req.user_id;
    let message = req.message.clone();
    let document_id = req.document_id;
    
    // Cast user_id and document_id to i32 for existing services if needed, 
    // BUT ConversationManager uses i64 internally (based on Step 261 manager.rs).
    // Let's check manager.rs: handle_message takes (SessionId, i32, String, Option<i32>).
    // Wait, in Step 261 manager.rs:
    // pub async fn handle_message(&self, session_id: SessionId, user_id: i32, ...)
    // But ChatRequest has user_id: i64.
    // I need to cast to i32 in handle_message call.
    // Wait, Step 261 manager.rs actually had `user_id: i32`.
    // My previous manager.rs implementation (Step 261) used i32?
    // Let's double check my manager.rs write.
    // In Step 261 I wrote `handle_message` taking `user_id: i32`.
    // It should probably take `i64` if `types.rs` says `user_id: i64`.
    // `conversation::types::ConversationState` has `user_id: i64`.
    // My manager.rs `handle_message` took `i32` and cast it to `i64`.
    // That means I should pass `i32` here?
    // `ChatRequest.user_id` is `i64`.
    // If I pass `req.user_id as i32`, I might lose data if it's large.
    // Ideally `manager.rs` should take `i64`.
    // I will check if I can modify this handler to cast, OR update `manager.rs`.
    // Updating `manager.rs` to take `i64` is better.
    // But for now, safe to cast if user_id < 2^31.
    // I'll stick to the plan `PART-3.md` code which just calls `manager.handle_message(session_id, user_id, message, document_id)`.
    // If mismatch, compiler will complain.
    // I will assume `PART-3.md` implies `manager.rs` takes compatible types.
    // Actually, looking at `PART-3.md` Step 6 (manager.rs) in docs...
    // Step 6 in `PART-2.md` `manager.rs` signature: `pub async fn handle_message(&self, session_id: SessionId, user_id: i64, message: String, document_id: Option<i64>)`.
    // So `manager.rs` SHOULD take `i64`.
    // Did I write `i32` in `manager.rs`?
    // In Step 795 (manager.rs write), I wrote:
    // `pub async fn handle_message(&self, session_id: SessionId, user_id: i32, message: String, document_id: Option<i32>)`
    // BECAUSE I was adapting to `rag_service` which used `i32`.
    // AND I cast `user_id as i64` inside.
    // So `manager.rs` expects `i32`.
    // BUT `ChatRequest` provides `i64`.
    // So I MUST cast here or change `manager.rs`.
    // I will convert here: `req.user_id as i32`.
    // Be careful with `document_id` too. `request.document_id` is `i64`. `manager` expects `Option<i32>`.
    
    // Create SSE stream
    let stream = stream::unfold(
        (conversation_manager, session_id, user_id, message, document_id, false),
        |(manager, session_id, user_id, message, document_id, mut done): (Arc<ConversationManager>, crate::models::chat::SessionId, i64, String, Option<i64>, bool)| async move {
            if done {
                return None;
            }

            // Handle message through conversation manager
            match manager.handle_message(session_id, user_id, message, document_id).await {
                Ok(response) => {
                    // Send response as stream events
                    done = true;
                    
                    // Event 1: Message content
                    let message_event = Event::default()
                        .event("message")
                        .data(response);
                    
                    // Event 2: Done signal
                    let _done_event = Event::default()
                        .event("done")
                        .data("[DONE]");
                    
                    Some((
                        Ok(message_event),
                        (manager, session_id, user_id, String::new(), document_id, done),
                    ))
                }
                Err(e) => {
                    error!("Error handling message: {}", e);
                    done = true;
                    
                    let error_event = Event::default()
                        .event("error")
                        .data(format!("{{\"message\": \"{}\"}}", e));
                    
                    Some((
                        Ok(error_event),
                        (manager, session_id, user_id, String::new(), document_id, done),
                    ))
                }
            }
        },
    );

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

/// Generate new session ID for user
/// POST /api/chat/session/new
#[derive(serde::Deserialize)]
pub struct NewSessionRequest {
    pub user_id: i64,
}

#[derive(serde::Serialize)]
pub struct NewSessionResponse {
    pub session_id: i64,
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

    let stream = stream::unfold(rx, move |mut rx| async move {
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

    Sse::new(stream).keep_alive(KeepAlive::default())
}

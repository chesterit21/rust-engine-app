Implement solusi “selected document(s) harus jadi hard filter retrieval” dengan cara: **tambahkan `document_ids` (multi) di payload**, propagate sampai ConversationManager, lalu di Repository bikin query vector-search yang pakai `WHERE document_id = ANY($3)` + join `vw_user_documents` supaya tetap aman authorization.

Di bawah ini gue kasih **kode lengkap** (file replace / tambah) supaya bisa langsung kamu copy-paste.

## 1) Replace: `src/models/chat.rs`

```rust
use serde::{Deserialize, Serialize};
use unicode_segmentation::UnicodeSegmentation;

/// OpenAI-compatible message format (SHARED across all modules)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,      // "user" | "assistant" | "system"
    pub content: String,
}

impl ChatMessage {
    pub fn user(content: impl Into<String>) -> Self {
        Self { role: "user".to_string(), content: content.into() }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self { role: "assistant".to_string(), content: content.into() }
    }

    pub fn system(content: impl Into<String>) -> Self {
        Self { role: "system".to_string(), content: content.into() }
    }

    /// Estimate token count for this message
    pub fn estimate_tokens(&self) -> usize {
        let role_chars = self.role.graphemes(true).count();
        let content_chars = self.content.graphemes(true).count();
        let total_chars = role_chars + content_chars;
        ((total_chars + 2) / 3).max(1) + 3
    }
}

/// Session ID type
pub type SessionId = i64;

/// Chat request payload
#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub user_id: i64,
    pub session_id: SessionId,
    pub message: String,

    /// Backward-compatible: single selected document
    #[serde(default)]
    pub document_id: Option<i64>,

    /// NEW: multi selected documents (client bisa kirim satu atau lebih)
    /// Note: pakai serde(default) biar request lama tetap valid.
    #[serde(default)]
    pub document_ids: Option<Vec<i64>>,
}

/// Chat response (for non-streaming)
#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub session_id: SessionId,
    pub message: String,
    pub sources: Vec<SourceInfo>,
}

/// Source information for citations
#[derive(Debug, Serialize, Clone)]
pub struct SourceInfo {
    pub document_id: i64,
    pub document_title: String,
    pub chunk_id: i64,
    pub similarity: f32,
}

/// Streaming event types for SSE
#[derive(Debug, Serialize)]
#[serde(tag = "event", content = "data")]
pub enum StreamEvent {
    #[serde(rename = "sources")]
    Sources(Vec<SourceInfo>),

    #[serde(rename = "message")]
    Message(String),

    #[serde(rename = "done")]
    Done,

    #[serde(rename = "error")]
    Error { message: String },
}

/// Generate new session ID for user
#[derive(serde::Deserialize)]
pub struct NewSessionRequest {
    pub user_id: i64,
}

#[derive(serde::Serialize)]
pub struct NewSessionResponse {
    pub session_id: i64,
}

/// Cache statistics response
#[derive(serde::Serialize)]
pub struct CacheStatsResponse {
    pub active_sessions: usize,
    pub memory_usage_mb: u64,
    pub memory_total_mb: u64,
    pub memory_usage_percent: f64,
}

/// Cleanup response
#[derive(serde::Serialize)]
pub struct CleanupResponse {
    pub sessions_removed: usize,
}
```

## 2) Replace: `src/handlers/chat.rs`

```rust
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
    info!(?req, "Incoming chat request");

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

    // Create SSE stream
    let stream = async_stream::stream! {
        match conversation_manager
            .handle_message(session_id, user_id, message, document_id, document_ids)
            .await
        {
            Ok(mut response_stream) => {
                use futures::StreamExt;

                while let Some(chunk_res) = response_stream.next().await {
                    match chunk_res {
                        Ok(chunk) => {
                            yield Ok(Event::default().event("message").data(chunk));
                        }
                        Err(e) => {
                            error!("Stream error: {}", e);
                            yield Ok(Event::default().event("error").data(format!("{{\"message\": \"{}\"}}", e)));
                        }
                    }
                }

                yield Ok(Event::default().event("done").data("[DONE]"));
            }
            Err(e) => {
                error!("Error handling message: {}", e);
                yield Ok(Event::default().event("error").data(format!("{{\"message\": \"{}\"}}", e)));
            }
        }
    };

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

    let session_id = req.session_id.unwrap_or_else(|| {
        ConversationManager::generate_session_id(req.user_id as i64)
    });

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

    let processing_docs = repository
        .get_user_processing_documents(req.user_id)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    info!(
        "Initialized session {} with {} docs and {} in-progress docs for user {}",
        session_id,
        documents.len(),
        processing_docs.len(),
        req.user_id
    );

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
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                Err(tokio::sync::broadcast::error::RecvError::Closed) => return None,
            }
        }
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}
```

## 3) Replace: `src/services/conversation/manager.rs`
>
> File ini sebelumnya cuma terima `document_id: Option<i64>` dan retrieval provider `search(..., document_id)` doang.
> Di versi ini, kita propagate `document_ids` sampai ke retrieval, dan **meta-question + multi-doc** kita balikin klarifikasi biar gak “miss” konteks.

```rust
/// manager.rs
use anyhow::{Context, Result};
use tracing::{debug, error, info, warn};
use crate::models::chat::{ChatMessage, SessionId};
use crate::database::models::{DocumentMetadata, DocumentOverview};
use super::cache::ConversationCache;
use super::context_builder::ContextBuilder;
use super::types::{ConversationState, RetrievalDecision, RetrievalReason};
use super::verification::{LlmVerifier, VerificationResult};
use crate::services::rag_service::ContextMetrics;
use std::collections::HashSet;

// NEW: meta intent analyzer
use crate::services::query_analyzer::QueryAnalyzer;

/// Trait for embedding service (break circular dependency)
#[async_trait::async_trait]
pub trait EmbeddingProvider: Send + Sync {
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;
    async fn embed_weighted(
        &self,
        current_text: &str,
        context_text: &str,
        current_weight: f32,
        history_weight: f32,
    ) -> Result<Vec<f32>>;
}

/// Trait for retrieval service
#[async_trait::async_trait]
pub trait RetrievalProvider: Send + Sync {
    async fn search(
        &self,
        user_id: i64,
        embedding: &[f32],
        document_id: Option<i64>,
        document_ids: Option<Vec<i64>>,
    ) -> Result<Vec<RetrievalChunk>>;

    // ============ NEW METHODS FOR META-QUESTIONS ============
    async fn get_document_metadata(&self, document_id: i32) -> Result<DocumentMetadata>;
    async fn get_document_overview_chunks(&self, document_id: i32, limit: i32) -> Result<Vec<RetrievalChunk>>;
    async fn get_document_overview(&self, document_id: i32, chunk_limit: i32) -> Result<DocumentOverview>;
}

use std::pin::Pin;
use futures::stream::Stream;

/// Trait for LLM service
#[async_trait::async_trait]
pub trait LlmProvider: Send + Sync {
    async fn generate(&self, messages: &[ChatMessage]) -> Result<String>;
    async fn generate_stream(&self, messages: &[ChatMessage]) -> Result<Pin<Box<dyn Stream<Item = Result<String, anyhow::Error>> + Send>>>;
    async fn summarize_chunks(&self, chunks: &[RetrievalChunk], query: &str) -> Result<String>;
}

/// Chunk result from retrieval (define here to avoid circular dep)
#[derive(Debug, Clone)]
pub struct RetrievalChunk {
    pub chunk_id: i64,
    pub document_id: i64,
    pub document_title: Option<String>,
    pub content: String,
    pub similarity: f32,
}

use crate::logging::{ActivityLogger, ActivityLog, ActivityType, ActivityStatus};
use std::time::Instant;

pub struct ConversationManager {
    cache: ConversationCache,
    context_builder: ContextBuilder,
    embedding_provider: Box<dyn EmbeddingProvider>,
    retrieval_provider: Box<dyn RetrievalProvider>,
    llm_provider: Box<dyn LlmProvider>,
    logger: ActivityLogger,
    stream_enabled: bool,
}

impl ConversationManager {
    pub fn new(
        embedding_provider: Box<dyn EmbeddingProvider>,
        retrieval_provider: Box<dyn RetrievalProvider>,
        llm_provider: Box<dyn LlmProvider>,
        logger: ActivityLogger,
        stream_enabled: bool,
        system_prompt: String,
    ) -> Self {
        Self {
            cache: ConversationCache::new(),
            context_builder: ContextBuilder::new(system_prompt),
            embedding_provider,
            retrieval_provider,
            llm_provider,
            logger,
            stream_enabled,
        }
    }

    fn normalize_scope(
        document_id: Option<i64>,
        document_ids: Option<Vec<i64>>,
    ) -> (Option<i64>, Option<Vec<i64>>) {
        if document_id.is_some() {
            return (document_id, None);
        }
        match document_ids {
            None => (None, None),
            Some(ids) => {
                let ids: Vec<i64> = ids.into_iter().filter(|id| *id > 0).collect();
                if ids.is_empty() {
                    return (None, None);
                }
                if ids.len() == 1 {
                    return (Some(ids[0]), None);
                }
                (None, Some(ids))
            }
        }
    }

    pub async fn handle_message(
        self: std::sync::Arc<Self>,
        session_id: SessionId,
        user_id: i64,
        message: String,
        document_id: Option<i64>,
        document_ids: Option<Vec<i64>>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String, anyhow::Error>> + Send>>> {
        let start_time = Instant::now();

        let (effective_doc_id, effective_doc_ids) = Self::normalize_scope(document_id, document_ids);

        self.logger.log(
            ActivityLog::builder(session_id, user_id, ActivityType::ProcessingStage)
                .message("PESAN MASUK")
                .build()
        );

        // Load session state (state masih simpan Option<i64>, jadi untuk multi-doc kita simpan None)
        let mut state = self.get_or_create_session(session_id, user_id, effective_doc_id).await?;

        self.logger.log(
            ActivityLog::builder(session_id, user_id, ActivityType::RequestReceived)
                .message(&message)
                .document_id(effective_doc_id.unwrap_or(0))
                .status(ActivityStatus::Info)
                .build()
        );

        // ===== EARLY EXIT: meta-question + multi-doc -> minta user pilih dokumen =====
        if QueryAnalyzer::is_meta_question(&message) {
            if let Some(ids) = effective_doc_ids.clone() {
                // Ambil judul dokumen untuk bantu user milih (limit biar aman)
                let mut titles: Vec<String> = Vec::new();
                for doc_id in ids.iter().take(8) {
                    if let Ok(meta) = self.retrieval_provider.get_document_metadata(*doc_id as i32).await {
                        titles.push(format!("- {} (id: {})", meta.title, meta.document_id));
                    }
                }

                let mut answer = String::from(
                    "Kamu lagi pilih lebih dari satu dokumen.\n\
                     Pertanyaan seperti \"ini dokumen apa\" butuh target dokumen yang spesifik.\n\
                     Tolong pilih 1 dokumen dulu (kirim `document_id`) atau sebutkan judulnya.\n\n"
                );

                if !titles.is_empty() {
                    answer.push_str("Pilihan dokumen yang kamu select:\n");
                    answer.push_str(&titles.join("\n"));
                }

                let manager = self.clone();
                let mut final_state = state;

                let stream = async_stream::try_stream! {
                    final_state.messages.push(ChatMessage::user(&message));
                    yield answer.clone();
                    final_state.messages.push(ChatMessage::assistant(&answer));
                    final_state.metadata.total_messages += 2;
                    final_state.touch();
                    manager.cache.set(session_id, final_state);
                };

                return Ok(Box::pin(stream));
            }
        }

        // Enforce sliding window
        if state.needs_window_enforcement() {
            self.logger.log(
                ActivityLog::builder(session_id, user_id, ActivityType::SlidingWindowEnforced)
                    .status(ActivityStatus::Warning)
                    .build()
            );
        }
        self.enforce_sliding_window(&mut state)?;

        // Generate embedding
        self.logger.log(
            ActivityLog::builder(session_id, user_id, ActivityType::ProcessingStage)
                .message("KIRIM KE MODEL EMBEDDING")
                .build()
        );

        let query_embedding = self.embedding_provider
            .embed(&message)
            .await
            .context("Failed to generate embedding")?;

        self.logger.log(
            ActivityLog::builder(session_id, user_id, ActivityType::ProcessingStage)
                .message("SELESAI MODEL EMBEDDING")
                .build()
        );

        // ====== ITERATIVE RETRIEVAL LOOP ======
        let verifier = LlmVerifier::new(3);
        let tried_chunk_ids: std::collections::HashSet<i64> = std::collections::HashSet::new();
        let mut iteration = 0;
        const MAX_ITERATIONS: usize = 3;

        let mut context_metrics = ContextMetrics::default();
        let mut retrieval_duration_total = 0i32;

        let manager = self.clone();
        let mut final_state = state;

        let stream = async_stream::try_stream! {
            let mut final_answer = String::new();

            loop {
                iteration += 1;

                if iteration > MAX_ITERATIONS {
                    warn!("Max iterations ({}) reached, returning best effort", MAX_ITERATIONS);
                    final_answer = "Maaf, saya tidak dapat menemukan informasi yang cukup setelah beberapa kali pencarian. Silakan coba pertanyaan yang lebih spesifik atau upload dokumen yang relevan.".to_string();
                    break;
                }

                info!("Retrieval iteration {}/{}", iteration, MAX_ITERATIONS);

                // IMPORTANT: decide_retrieval masih pakai effective_doc_id (single doc) untuk logic yang sudah ada
                let decision = manager.context_builder.decide_retrieval(
                    &final_state,
                    &message,
                    effective_doc_id,
                    Some(&query_embedding),
                )?;

                let retrieval_start = Instant::now();
                let (system_context, metrics) = manager.execute_retrieval_with_metrics(
                    &mut final_state,
                    &decision,
                    &message,
                    effective_doc_id,
                    effective_doc_ids.clone(),
                    &query_embedding,
                    &tried_chunk_ids,
                ).await?;

                let retrieval_duration = retrieval_start.elapsed().as_millis() as i32;
                retrieval_duration_total += retrieval_duration;
                context_metrics = metrics.clone();

                if matches!(decision, RetrievalDecision::Retrieve { .. }) {
                    manager.logger.log(
                        ActivityLog::builder(session_id, user_id, ActivityType::RetrievalExecuted)
                            .retrieval_duration(retrieval_duration)
                            .retrieval_skipped(false)
                            .build()
                    );
                }

                if iteration == 1 {
                    final_state.messages.push(ChatMessage::user(&message));
                }

                let token_count_before = final_state.metadata.total_tokens_last;
                manager.manage_tokens(&mut final_state, &system_context).await?;
                let token_count_after = final_state.metadata.total_tokens_last;

                if token_count_before > 24_000 {
                    manager.logger.log(
                        ActivityLog::builder(session_id, user_id, ActivityType::TokenOverflow)
                            .status(ActivityStatus::Warning)
                            .token_count(token_count_before as i32)
                            .build()
                    );
                }

                let enhanced_system = verifier.build_verification_prompt(
                    &manager.context_builder.base_instruction()
                );

                let mut llm_messages = vec![
                    ChatMessage::system(format!("{}\n\n{}", enhanced_system, system_context))
                ];
                llm_messages.extend(final_state.messages.clone());

                let llm_start = Instant::now();
                manager.logger.log(
                    ActivityLog::builder(session_id, user_id, ActivityType::ProcessingStage)
                        .message(&format!("KIRIM KE MODEL UTAMA (Iteration {})", iteration))
                        .build()
                );

                let llm_response = if manager.stream_enabled {
                    let mut stream = manager.llm_provider.generate_stream(&llm_messages).await?;

                    manager.logger.log(
                        ActivityLog::builder(session_id, user_id, ActivityType::ProcessingStage)
                            .message("LLM SUDAH RESPONSE (streaming)")
                            .build()
                    );

                    use futures::StreamExt;
                    let mut accumulated = String::new();

                    while let Some(chunk_res) = stream.next().await {
                        match chunk_res {
                            Ok(chunk) => {
                                accumulated.push_str(&chunk);
                                if iteration == MAX_ITERATIONS {
                                    yield chunk;
                                }
                            }
                            Err(e) => {
                                error!("LLM stream error: {}", e);
                                Err(e)?;
                            }
                        }
                    }

                    accumulated
                } else {
                    let response = manager.call_llm_with_retry(&llm_messages).await?;

                    manager.logger.log(
                        ActivityLog::builder(session_id, user_id, ActivityType::ProcessingStage)
                            .message("LLM SUDAH RESPONSE")
                            .build()
                    );

                    response
                };

                let llm_duration = llm_start.elapsed().as_millis() as i32;

                match verifier.parse_response(&llm_response) {
                    VerificationResult::Answered(answer) => {
                        info!("LLM successfully answered on iteration {}", iteration);
                        final_answer = answer;

                        let total_duration = start_time.elapsed().as_millis() as i32;
                        manager.logger.log(
                            ActivityLog::builder(session_id, user_id, ActivityType::MessageSent)
                                .message(&message)
                                .response(&final_answer)
                                .token_count(token_count_after as i32)
                                .processing_time(total_duration)
                                .llm_duration(llm_duration)
                                .retrieval_duration(retrieval_duration_total)
                                .document_id(effective_doc_id.unwrap_or(0))
                                .custom("retrieval_iterations", iteration as i64)
                                .custom("context_truncated", if context_metrics.truncated { 1i64 } else { 0i64 })
                                .custom("documents_retrieved", context_metrics.documents_included as i64)
                                .custom("chunks_used", context_metrics.chunks_included as i64)
                                .custom("verification_result", "answered")
                                .build()
                        );

                        break;
                    }

                    VerificationResult::NeedMoreContext { doc_ids: _doc_ids, reason } => {
                        warn!("Iteration {}: LLM needs more context. Reason: {}", iteration, reason);

                        if iteration >= MAX_ITERATIONS {
                            final_answer = format!(
                                "Maaf, informasi dalam dokumen tidak cukup lengkap untuk menjawab pertanyaan Anda. {}",
                                if !reason.is_empty() { format!("Alasan: {}", reason) } else { String::new() }
                            );

                            manager.logger.log(
                                ActivityLog::builder(session_id, user_id, ActivityType::MessageSent)
                                    .message(&message)
                                    .response(&final_answer)
                                    .status(ActivityStatus::Warning)
                                    .custom("verification_result", "need_more_context")
                                    .custom("retrieval_iterations", iteration as i64)
                                    .build()
                            );

                            break;
                        }

                        continue;
                    }

                    VerificationResult::NotRelevant { reason } => {
                        warn!("Iteration {}: LLM says context not relevant. Reason: {}", iteration, reason);

                        if iteration >= MAX_ITERATIONS {
                            final_answer = format!(
                                "Maaf, dokumen yang tersedia tidak relevan dengan pertanyaan Anda. {}",
                                if !reason.is_empty() { format!("Detail: {}", reason) } else { String::new() }
                            );

                            manager.logger.log(
                                ActivityLog::builder(session_id, user_id, ActivityType::MessageSent)
                                    .message(&message)
                                    .response(&final_answer)
                                    .status(ActivityStatus::Warning)
                                    .custom("verification_result", "not_relevant")
                                    .custom("retrieval_iterations", iteration as i64)
                                    .build()
                            );

                            break;
                        }

                        continue;
                    }
                }
            }

            if !manager.stream_enabled || iteration < MAX_ITERATIONS {
                yield final_answer.clone();
            }

            final_state.messages.push(ChatMessage::assistant(&final_answer));
            final_state.last_query_embedding = Some(query_embedding);
            final_state.metadata.total_messages += 2;
            final_state.touch();

            manager.cache.set(session_id, final_state);
        };

        Ok(Box::pin(stream))
    }

    async fn execute_retrieval_with_metrics(
        &self,
        state: &mut ConversationState,
        decision: &RetrievalDecision,
        current_message: &str,
        document_id: Option<i64>,
        document_ids: Option<Vec<i64>>,
        current_embedding: &[f32],
        tried_chunk_ids: &HashSet<i64>,
    ) -> Result<(String, ContextMetrics)> {
        match decision {
            RetrievalDecision::Skip { reason } => {
                debug!("Skipping retrieval: {:?}", reason);
                state.metadata.retrieval_skipped_count += 1;
                Ok((state.system_context.clone(), ContextMetrics::default()))
            }

            RetrievalDecision::Retrieve { reason, context_aware } => {
                if matches!(reason, RetrievalReason::DocumentMetadataQuery) {
                    let context = self.execute_metadata_query(state, document_id).await?;
                    return Ok((context, ContextMetrics::default()));
                }

                info!("Performing retrieval: {:?}", reason);
                state.metadata.total_retrievals += 1;

                let query_embedding = if *context_aware {
                    let context_text = self.context_builder
                        .prepare_context_aware_text(current_message, &state.messages);

                    let config = self.context_builder.weighted_config();
                    self.embedding_provider
                        .embed_weighted(
                            current_message,
                            &context_text,
                            config.current_weight,
                            config.history_weight,
                        )
                        .await?
                } else {
                    current_embedding.to_vec()
                };

                let mut chunks = self.retrieval_provider
                    .search(state.user_id, &query_embedding, document_id, document_ids)
                    .await
                    .context("Retrieval failed")?;

                chunks.retain(|c| !tried_chunk_ids.contains(&c.chunk_id));

                if chunks.is_empty() {
                    warn!("No new chunks found after filtering tried chunks");
                    return Ok((
                        "Tidak ada informasi tambahan yang ditemukan.".to_string(),
                        ContextMetrics::default()
                    ));
                }

                let doc_chunks: Vec<crate::database::DocumentChunk> = chunks.iter().map(|c| {
                    crate::database::DocumentChunk {
                        chunk_id: c.chunk_id,
                        document_id: c.document_id as i32,
                        document_title: c.document_title.clone().unwrap_or_default(),
                        content: c.content.clone(),
                        similarity: c.similarity,
                        chunk_index: 0,
                        page_number: None,
                    }
                }).collect();

                let (context, metrics) = self.build_structured_rag_context(doc_chunks)?;

                self.logger.log(
                    ActivityLog::builder(state.session_id, state.user_id, ActivityType::ProcessingStage)
                        .message(&format!(
                            "CONTEXT GENERATED: {} tokens, {} docs, {} chunks",
                            metrics.total_tokens,
                            metrics.documents_included,
                            metrics.chunks_included
                        ))
                        .build()
                );

                state.system_context = context.clone();
                state.last_retrieval_summary = context.clone();
                state.document_id = document_id;

                Ok((context, metrics))
            }
        }
    }

    fn build_structured_rag_context(
        &self,
        chunks: Vec<crate::database::DocumentChunk>,
    ) -> Result<(String, ContextMetrics)> {
        use crate::utils::token_estimator;
        use std::collections::HashMap;

        if chunks.is_empty() {
            return Ok((
                "Tidak ada konteks yang relevan ditemukan.".to_string(),
                ContextMetrics::default(),
            ));
        }

        let mut grouped: HashMap<i32, Vec<crate::database::DocumentChunk>> = HashMap::new();
        for chunk in chunks {
            grouped.entry(chunk.document_id).or_default().push(chunk);
        }

        let max_tokens = 20_000;
        let mut context = String::from("DOKUMEN YANG TERSEDIA:\n\n");
        let mut metrics = ContextMetrics::default();
        let mut current_tokens = token_estimator::estimate_tokens(&context);

        for (doc_id, chunks) in grouped {
            let doc_title = chunks.first().map(|c| c.document_title.as_str()).unwrap_or("Unknown");
            let avg_sim: f32 = chunks.iter().map(|c| c.similarity).sum::<f32>() / chunks.len() as f32;

            let doc_header = format!(
                "<document id=\"doc_{}\" title=\"{}\" relevance=\"{:.3}\">\n",
                doc_id, doc_title, avg_sim
            );

            if current_tokens + token_estimator::estimate_tokens(&doc_header) > max_tokens {
                metrics.truncated = true;
                break;
            }

            context.push_str(&doc_header);
            current_tokens += token_estimator::estimate_tokens(&doc_header);
            metrics.documents_included += 1;

            for chunk in chunks {
                let chunk_text = format!(
                    "<chunk id=\"chunk_{}\" similarity=\"{:.3}\">\n{}\n</chunk>\n\n",
                    chunk.chunk_id, chunk.similarity, chunk.content.trim()
                );

                if current_tokens + token_estimator::estimate_tokens(&chunk_text) > max_tokens {
                    metrics.truncated = true;
                    break;
                }

                context.push_str(&chunk_text);
                current_tokens += token_estimator::estimate_tokens(&chunk_text);
                metrics.chunks_included += 1;
            }

            context.push_str("</document>\n\n");
        }

        metrics.total_tokens = current_tokens;
        Ok((context, metrics))
    }

    async fn execute_metadata_query(
        &self,
        state: &mut ConversationState,
        document_id: Option<i64>,
    ) -> Result<String> {
        info!("Processing document metadata query");

        if let Some(doc_id) = document_id {
            self.logger.log(
                ActivityLog::builder(state.session_id, state.user_id, ActivityType::ProcessingStage)
                    .message("FETCH DOCUMENT METADATA")
                    .build()
            );

            let overview = self.retrieval_provider
                .get_document_overview(doc_id as i32, 5)
                .await
                .context("Failed to fetch document overview")?;

            let context_text = self.build_metadata_context(&overview);

            let system_context = self.context_builder.build_system_context(
                &context_text,
                Some(&format!("Document: {}", overview.metadata.title)),
            );

            state.system_context = system_context.clone();
            state.last_retrieval_summary = context_text;
            state.document_id = document_id;
            state.metadata.total_retrievals += 1;

            self.logger.log(
                ActivityLog::builder(state.session_id, state.user_id, ActivityType::ProcessingStage)
                    .message("METADATA FETCH COMPLETED")
                    .build()
            );

            Ok(system_context)
        } else {
            let error_msg = "Untuk menjawab pertanyaan tentang dokumen, silakan upload atau pilih dokumen terlebih dahulu.";
            state.system_context = error_msg.to_string();
            Ok(error_msg.to_string())
        }
    }

    pub async fn get_or_create_session(
        &self,
        session_id: SessionId,
        user_id: i64,
        document_id: Option<i64>,
    ) -> Result<ConversationState> {
        if let Some(state) = self.cache.get(session_id) {
            debug!("Found existing session {}", session_id);
            return Ok(state);
        }

        if !self.cache.can_create_new_session() {
            anyhow::bail!("Memory limit reached (90%), cannot create new session");
        }

        info!("Creating new session {} for user {}", session_id, user_id);

        self.logger.log(
            ActivityLog::builder(session_id, user_id, ActivityType::SessionCreated)
                .status(ActivityStatus::Info)
                .build()
        );

        let state = ConversationState::new(session_id, user_id, document_id);
        self.cache.set(session_id, state.clone());

        Ok(state)
    }

    pub fn generate_session_id(user_id: i64) -> SessionId {
        let now = chrono::Utc::now();
        let timestamp = now.format("%Y%m%d%H%M%S").to_string();
        format!("{}{}", timestamp, user_id)
            .parse()
            .expect("Failed to parse session_id")
    }

    fn enforce_sliding_window(&self, state: &mut ConversationState) -> Result<()> {
        if !state.needs_window_enforcement() {
            return Ok(());
        }

        info!(
            "Enforcing sliding window for session {} (current pairs: {})",
            state.session_id,
            state.message_pair_count()
        );

        if state.messages.len() >= 2 {
            state.messages.drain(0..2);
            debug!("Removed oldest message pair (Q1, A1)");
        }

        Ok(())
    }

    async fn manage_tokens(
        &self,
        state: &mut ConversationState,
        system_context: &str,
    ) -> Result<()> {
        use super::token_counter::TokenCounter;

        let token_count = TokenCounter::count_payload(system_context, &state.messages, "");
        debug!(
            "Token count: {} (system: {}, history: {})",
            token_count.total, token_count.system_tokens, token_count.history_tokens
        );

        state.metadata.total_tokens_last = token_count.total;

        if !token_count.is_over_soft_limit() {
            return Ok(());
        }

        warn!("Token count {} exceeds 20K, performing cascade deletion", token_count.total);

        self.logger.log(
            ActivityLog::builder(state.session_id, state.user_id, ActivityType::CascadeDeletion)
                .status(ActivityStatus::Warning)
                .token_count(token_count.total as i32)
                .build()
        );

        let mut current_count = token_count.total;
        let mut deletion_round = 1;

        while current_count > 20_000 && state.messages.len() >= 2 {
            info!("Deletion round {}: removing oldest pair", deletion_round);

            state.messages.drain(0..2);

            let new_count = TokenCounter::count_payload(system_context, &state.messages, "");
            current_count = new_count.total;

            debug!("After deletion round {}: {} tokens", deletion_round, current_count);
            deletion_round += 1;

            if state.messages.is_empty() {
                warn!("All history deleted, only current message remains");
                break;
            }
        }

        if current_count > 23_000 {
            warn!(
                "Token count {} still over 23K after deletion, truncating retrieval",
                current_count
            );

            let truncated_summary = state.last_retrieval_summary
                .chars()
                .take(500)
                .collect::<String>() + "... (truncated)";

            let new_system = self.context_builder.build_system_context(
                &truncated_summary,
                state.document_id.map(|id| format!("Document ID: {}", id)).as_deref(),
            );

            state.system_context = new_system;
            info!("Retrieval context truncated");
        }

        Ok(())
    }

    async fn call_llm_with_retry(&self, messages: &[ChatMessage]) -> Result<String> {
        const MAX_RETRIES: u32 = 3;

        for attempt in 1..=MAX_RETRIES {
            match self.llm_provider.generate(messages).await {
                Ok(response) => {
                    debug!("LLM call succeeded on attempt {}", attempt);
                    return Ok(response);
                }
                Err(e) => {
                    if attempt < MAX_RETRIES {
                        warn!("LLM call failed (attempt {}): {}. Retrying...", attempt, e);
                        tokio::time::sleep(tokio::time::Duration::from_secs(attempt as u64)).await;
                    } else {
                        error!("LLM call failed after {} attempts: {}", MAX_RETRIES, e);
                        anyhow::bail!("Server ada gangguan, silakan coba lagi nanti");
                    }
                }
            }
        }

        unreachable!()
    }

    pub fn cache_stats(&self) -> super::cache::CacheStats {
        self.cache.stats()
    }

    pub fn cleanup_expired_sessions(&self) -> usize {
        self.cache.cleanup_expired()
    }

    pub fn logger(&self) -> &ActivityLogger {
        &self.logger
    }

    fn build_metadata_context(&self, overview: &DocumentOverview) -> String {
        let metadata = &overview.metadata;

        let mut context = String::new();

        context.push_str("=== INFORMASI DOKUMEN ===\n");
        context.push_str(&format!("Judul: {}\n", metadata.title));

        if let Some(desc) = &metadata.description {
            context.push_str(&format!("Deskripsi: {}\n", desc));
        }

        if let Some(summary) = &metadata.auto_summary {
            context.push_str(&format!("\nRingkasan:\n{}\n", summary));
        }

        context.push_str(&format!("\nTotal bagian: {}\n", metadata.total_chunks));

        if let Some(size) = metadata.file_size {
            let size_kb = size as f64 / 1024.0;
            context.push_str(&format!("Ukuran file: {:.1} KB\n", size_kb));
        }

        if !overview.first_chunks.is_empty() {
            context.push_str("\n=== CUPLIKAN AWAL DOKUMEN ===\n\n");

            for (i, chunk) in overview.first_chunks.iter().enumerate() {
                let preview = chunk.content.chars().take(300).collect::<String>();
                let ellipsis = if chunk.content.len() > 300 { "..." } else { "" };
                context.push_str(&format!("[Bagian {}]\n{}{}\n\n", i + 1, preview, ellipsis));
            }
        }

        context
    }
}
```

## 4) Replace: `src/services/rag_service.rs`
>
> Existing `retrieve_with_embedding` cuma bisa `document_id: Option<i32>`.
> Di versi ini, kita tambahin multi-doc path yang manggil repository method baru.

```rust
use crate::config::RagConfig;
use crate::database::{DocumentChunk, Repository};
use crate::services::{EmbeddingService, LlmService};
use crate::utils::error::ApiError;
use anyhow::Result;
use pgvector::Vector;
use std::sync::Arc;
use tracing::{debug, info};
use crate::database::models::{DocumentMetadata, DocumentOverview};
use crate::services::conversation::manager::{RetrievalProvider, RetrievalChunk};
use std::collections::HashMap;
use crate::utils::token_estimator;

#[derive(Debug, Clone)]
pub struct GroupedDocument {
    pub doc_id: i32,
    pub doc_title: String,
    pub chunks: Vec<DocumentChunk>,
    pub avg_similarity: f32,
    pub total_tokens: usize,
}

#[derive(Debug, Default, Clone)]
pub struct ContextMetrics {
    pub total_tokens: usize,
    pub documents_included: usize,
    pub chunks_included: usize,
    pub truncated: bool,
}

#[derive(Clone)]
pub struct RagService {
    pub repository: Arc<Repository>,
    pub embedding_service: Arc<EmbeddingService>,
    pub llm_service: Arc<LlmService>,
    pub config: RagConfig,
}

impl RagService {
    pub fn new(
        repository: Arc<Repository>,
        embedding_service: Arc<EmbeddingService>,
        llm_service: Arc<LlmService>,
        config: RagConfig,
    ) -> Self {
        Self { repository, embedding_service, llm_service, config }
    }

    pub async fn retrieve(
        &self,
        user_id: i32,
        query: &str,
        document_id: Option<i32>,
    ) -> Result<Vec<DocumentChunk>, ApiError> {
        info!("Retrieving context for user {} query: {}", user_id, query);
        let query_embedding = self.embedding_service.embed(query).await?;
        self.retrieve_with_embedding(user_id, query, query_embedding, document_id, None).await
    }

    pub async fn retrieve_with_embedding(
        &self,
        user_id: i32,
        query_text: &str,
        query_embedding: Vec<f32>,
        document_id: Option<i32>,
        document_ids: Option<Vec<i32>>,
    ) -> Result<Vec<DocumentChunk>, ApiError> {
        info!("Retrieving context with embedding for user {}", user_id);

        let vector = Vector::from(query_embedding);

        // RULE:
        // - kalau multi-doc: pakai vector search scoped ke ANY(document_ids)
        // - kalau query_text kosong: jangan hybrid (biar gak fulltext mismatch), pakai vector
        let use_hybrid = self.config.rerank_enabled && !query_text.trim().is_empty() && document_ids.is_none();

        let mut chunks = if let Some(ids) = document_ids.clone() {
            self.repository
                .search_user_documents_multi(
                    user_id,
                    vector,
                    ids,
                    self.config.retrieval_top_k as i32,
                )
                .await
                .map_err(|e| ApiError::DatabaseError(e.to_string()))?
        } else if use_hybrid {
            self.repository
                .hybrid_search_user_documents(
                    user_id,
                    vector,
                    query_text.to_string(),
                    self.config.retrieval_top_k as i32,
                    document_id,
                )
                .await
                .map_err(|e| ApiError::DatabaseError(e.to_string()))?
        } else {
            self.repository
                .search_user_documents(
                    user_id,
                    vector,
                    self.config.retrieval_top_k as i32,
                    document_id,
                )
                .await
                .map_err(|e| ApiError::DatabaseError(e.to_string()))?
        };

        // Intro injection tetap untuk single-doc
        if let Some(doc_id) = document_id {
            let has_intro = chunks.iter().any(|c| c.chunk_index == 0);
            if !has_intro {
                match self.repository.get_first_chunk(doc_id).await {
                    Ok(Some(intro_chunk)) => {
                        debug!("Injecting intro chunk (index 0) for context robustness");
                        chunks.insert(0, intro_chunk);
                    }
                    Ok(None) => debug!("No intro chunk found for doc {}", doc_id),
                    Err(e) => tracing::warn!("Failed to fetch intro chunk: {}", e),
                }
            }
        }

        debug!("Retrieved {} chunks", chunks.len());
        Ok(chunks)
    }

    pub fn build_context(&self, chunks: Vec<DocumentChunk>) -> String {
        if chunks.is_empty() {
            return String::from("Tidak ada konteks yang relevan ditemukan.");
        }

        let mut context = String::from("Konteks yang relevan:\n\n");

        for (i, chunk) in chunks.iter().enumerate() {
            context.push_str(&format!(
                "[Dokumen: {} | Halaman: {}]\n{}\n\n",
                chunk.document_title,
                chunk.page_number.unwrap_or(0),
                chunk.content
            ));

            if context.len() > self.config.max_context_length {
                debug!(
                    "Context truncated at {} chunks (max length: {})",
                    i + 1,
                    self.config.max_context_length
                );
                break;
            }
        }

        context
    }

    pub fn build_prompt(&self, user_query: &str, context: &str) -> Vec<crate::models::chat::ChatMessage> {
        let system_message = crate::models::chat::ChatMessage {
            role: "system".to_string(),
            content: format!(
                "Anda adalah asisten AI yang membantu menjawab pertanyaan berdasarkan dokumen yang diberikan. \
                 Jawab pertanyaan dengan akurat berdasarkan konteks yang tersedia. \
                 Jika informasi tidak ada dalam konteks, katakan dengan jelas.\n\n{}",
                context
            ),
        };

        let user_message = crate::models::chat::ChatMessage {
            role: "user".to_string(),
            content: user_query.to_string(),
        };

        vec![system_message, user_message]
    }

    pub fn build_structured_context(&self, chunks: Vec<DocumentChunk>) -> (String, ContextMetrics) {
        if chunks.is_empty() {
            return (
                "Tidak ada konteks yang relevan ditemukan.".to_string(),
                ContextMetrics::default(),
            );
        }

        let grouped = self.group_chunks_by_document(chunks);
        let mut sorted_docs: Vec<GroupedDocument> = grouped.into_values().collect();
        sorted_docs.sort_by(|a, b| b.avg_similarity.partial_cmp(&a.avg_similarity).unwrap());

        self.format_grouped_context(sorted_docs)
    }

    fn group_chunks_by_document(&self, chunks: Vec<DocumentChunk>) -> HashMap<i32, GroupedDocument> {
        let mut grouped: HashMap<i32, GroupedDocument> = HashMap::new();

        for chunk in chunks {
            let entry = grouped.entry(chunk.document_id).or_insert_with(|| GroupedDocument {
                doc_id: chunk.document_id,
                doc_title: chunk.document_title.clone(),
                chunks: Vec::new(),
                avg_similarity: 0.0,
                total_tokens: 0,
            });

            entry.total_tokens += token_estimator::estimate_tokens(&chunk.content);
            entry.chunks.push(chunk);
        }

        for doc in grouped.values_mut() {
            let sum: f32 = doc.chunks.iter().map(|c| c.similarity).sum();
            doc.avg_similarity = if doc.chunks.is_empty() { 0.0 } else { sum / doc.chunks.len() as f32 };
        }

        grouped
    }

    fn format_grouped_context(&self, sorted_docs: Vec<GroupedDocument>) -> (String, ContextMetrics) {
        let max_tokens = self.config.max_context_tokens;

        let mut context = String::from("DOKUMEN YANG TERSEDIA:\n\n");
        let mut metrics = ContextMetrics::default();
        let mut current_tokens = token_estimator::estimate_tokens(&context);

        for doc in sorted_docs {
            let doc_header = format!(
                "<document id=\"doc_{}\" title=\"{}\" relevance=\"{:.3}\">\n",
                doc.doc_id,
                doc.doc_title,
                doc.avg_similarity
            );

            let header_tokens = token_estimator::estimate_tokens(&doc_header);

            if current_tokens + header_tokens > max_tokens {
                metrics.truncated = true;
                break;
            }

            context.push_str(&doc_header);
            current_tokens += header_tokens;
            metrics.documents_included += 1;

            for chunk in &doc.chunks {
                let chunk_text = format!(
                    "<chunk id=\"chunk_{}\" page=\"{}\" similarity=\"{:.3}\">\n{}\n</chunk>\n\n",
                    chunk.chunk_id,
                    chunk.page_number.unwrap_or(0),
                    chunk.similarity,
                    chunk.content.trim()
                );

                let chunk_tokens = token_estimator::estimate_tokens(&chunk_text);

                if current_tokens + chunk_tokens > max_tokens {
                    metrics.truncated = true;
                    break;
                }

                context.push_str(&chunk_text);
                current_tokens += chunk_tokens;
                metrics.chunks_included += 1;
            }

            context.push_str("</document>\n\n");
            current_tokens += 2;

            if metrics.truncated {
                break;
            }
        }

        metrics.total_tokens = current_tokens;

        (context, metrics)
    }
}

#[async_trait::async_trait]
impl RetrievalProvider for RagService {
    async fn search(
        &self,
        user_id: i64,
        embedding: &[f32],
        document_id: Option<i64>,
        document_ids: Option<Vec<i64>>,
    ) -> Result<Vec<RetrievalChunk>> {
        let doc_ids_i32 = document_ids.map(|v| v.into_iter().map(|x| x as i32).collect::<Vec<_>>());

        let chunks = self.retrieve_with_embedding(
            user_id as i32,
            "", // no text in trait
            embedding.to_vec(),
            document_id.map(|id| id as i32),
            doc_ids_i32,
        ).await;

        match chunks {
            Ok(docs) => Ok(docs.into_iter().map(|d| RetrievalChunk {
                chunk_id: d.chunk_id,
                document_id: d.document_id as i64,
                document_title: Some(d.document_title),
                content: d.content,
                similarity: d.similarity,
            }).collect()),
            Err(e) => anyhow::bail!("Retrieval failed: {}", e),
        }
    }

    async fn get_document_metadata(&self, document_id: i32) -> Result<DocumentMetadata> {
        self.repository.get_document_metadata(document_id).await
    }

    async fn get_document_overview_chunks(&self, document_id: i32, limit: i32) -> Result<Vec<RetrievalChunk>> {
        let chunks = self.repository.get_document_overview_chunks(document_id, limit).await?;
        Ok(chunks.into_iter().map(|c| RetrievalChunk {
            chunk_id: c.chunk_id,
            document_id: c.document_id as i64,
            content: c.content,
            document_title: Some(c.document_title),
            similarity: c.similarity,
        }).collect())
    }

    async fn get_document_overview(&self, document_id: i32, chunk_limit: i32) -> Result<DocumentOverview> {
        self.repository.get_document_overview(document_id, chunk_limit).await
    }
}
```

## 5) Replace: `src/database/repository.rs` (tambah method multi-doc)
>
> Existing `search_user_documents(...)` cuma support `document_id: Option<i32>` lewat stored function `search_user_documents($1,$2,$3,$4)`.
> Ini kita tambah method baru `search_user_documents_multi(...)` yang query langsung (tanpa ubah DB function), jadi aman buat production.

Tambahkan method ini **di dalam `impl Repository`** (biarkan method lain tetap sama):

```rust
    /// Vector search scoped ke banyak dokumen (document_ids) dengan authorization via vw_user_documents
    pub async fn search_user_documents_multi(
        &self,
        user_id: i32,
        query_embedding: Vector,
        document_ids: Vec<i32>,
        limit: i32,
    ) -> Result<Vec<DocumentChunk>> {
        if document_ids.is_empty() {
            return Ok(vec![]);
        }

        // NOTE:
        // - Join vw_user_documents untuk enforce akses user (pattern sama dengan get_user_documents).
        // - similarity pakai (1 - cosine_distance). pgvector cosine distance operator: <=>.
        let chunks = sqlx::query_as::<_, DocumentChunk>(
            r#"
            SELECT
                c.id as chunk_id,
                c.document_id,
                d."DocumentTitle" as document_title,
                c.content,
                (1.0 - (c.embedding <=> $2))::float4 as similarity,
                c.chunk_index,
                NULL::int as page_number
            FROM rag_document_chunks c
            JOIN "TblDocuments" d ON d."Id" = c.document_id
            JOIN vw_user_documents ud
              ON ud.document_id = c.document_id
             AND ud.user_id = $1
            WHERE c.document_id = ANY($3)
            ORDER BY (c.embedding <=> $2) ASC
            LIMIT $4
            "#
        )
        .bind(user_id)
        .bind(query_embedding)
        .bind(&document_ids)
        .bind(limit)
        .persistent(false)
        .fetch_all(self.pool.get_pool())
        .await?;

        debug!(
            "Found {} relevant chunks for user {} (scoped to {} docs)",
            chunks.len(),
            user_id,
            document_ids.len()
        );

        Ok(chunks)
    }
```

***

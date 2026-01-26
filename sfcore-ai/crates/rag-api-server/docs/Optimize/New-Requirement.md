### HISTORY

Bisa, dan itu justru pola yang “bener” untuk RAG production: 1 request user → BE menjalankan beberapa tahap (plan → retrieval → answer), lalu BE mengirim **event progres** + token jawaban via SSE sampai ditutup dengan event `done`.

## Kontrak SSE yang jelas

Sekarang model event kamu baru ada `sources | message | done | error` (di `StreamEvent`).
Supaya client & BE sama-sama paham “ini belum selesai”, tambahin event tipe progres/stage (mis: `stage`) dan sertakan `request_id` + `phase` + `is_final`.

Contoh payload SSE (data JSON) yang stabil:

- `event: stage` data: `{"request_id":"...","phase":"plan","status":"start"}`
- `event: stage` data: `{"request_id":"...","phase":"retrieve","status":"done","chunks":12}`
- `event: message` data: `{"request_id":"...","delta":"..."}`
- `event: done` data: `{"request_id":"...","final":true}`

Ini lebih aman daripada “ngirim text biasa”, karena UI bisa render progress bar dan tahu kapan selesai.

## Orkestrasi 2–3x panggil LLM

Dengan service kamu sekarang, kamu sudah punya dua mode: non-stream (`generate_chat`) dan stream (`chat_stream`/`generate_stream`).
Pattern yang gue rekomendasikan:

1) **Plan call** (non-stream, cepat): minta LLM output JSON (intent, keywords, perlu doc atau tidak, perlu summary atau Q&A). Pakai `generate_chat(...)`.
2) **Retrieval** (DB): pakai hasil plan untuk nentuin doc scope + query vector/hybrid.  
3) **Answer call** (stream): kirim final prompt (system role “RAG assistant”) + context, lalu stream token ke UI pakai `generate_stream(...)`.

Kalau dokumen kepanjangan, kamu bisa tambah step “map-reduce summary”: panggil `summarize_chunks(...)` per batch chunk, lalu final synthesis (jadi total 3+ call).

## Real-case 1 & 2 (yang kamu tulis)

1) User “hi bro” tanpa dokumen: plan call harus bisa memutuskan “ini smalltalk/general”, jadi BE **skip retrieval** dan langsung answer stream dengan base instruction (general assistant). (Saat ini `ContextBuilder` baru deteksi meta-question dokumen & clarification; smalltalk belum ada, jadi perlu rule/intent baru atau LLM plan).
2) User kirim doc_id lalu minta summary: plan call output `intent=summary`, `doc_scope=doc_id`, plus “bagian mana”/keywords kalau perlu; BE fetch overview chunks/metadata (kamu sudah punya jalur metadata query via `DocumentMetadataQuery` di builder), lalu answer stream.

## Catatan UX yang penting

Jangan tampilkan “planning” mentah ke user (raw keyword list/struktur internal) karena itu bikin UX aneh; yang dikirim ke UI cukup “Sedang memahami pertanyaan…” / “Sedang mengambil konteks…” (stage events), lalu baru stream jawaban final.
Kalau kamu tetap mau menampilkan “intermediate response”, jadikan itu **status message** (event `stage`) bukan event `message` agar UI tidak menganggap itu jawaban final.

Bisa, dan cara paling “natural” tanpa bikin user merasa pola itu adalah: **pisahin stream jadi 2 channel event**: (1) status/phase text yang berubah-ubah, (2) token jawaban final. Di code kamu sekarang semua chunk selalu dikirim sebagai `event("message")`, jadi UI gak punya cara bedain “ini status” vs “ini jawaban”.

## Prinsip desain (biar natural + gak repetitif)

1) **Status text harus variatif**: bukan 1 kalimat template yang sama tiap request, tapi kumpulan phrase + disisipi konteks (judul dokumen, tipe task, jumlah halaman/chunk) + dipilih berdasarkan seed (request_id) supaya gak “itu-itu lagi”.  
2) **Status text jangan bocorin chain-of-thought**: cukup “Sedang memahami permintaan…”, “Sedang membaca dokumen…”, “Sedang merangkum…”, bukan “Saya akan melakukan langkah 1/2/3…”.  
3) Jawaban final tetap **stream token** dari LLM, supaya UX kerasa hidup.

## Workflow yang pas buat skenario kamu

Satu request user, BE boleh panggil LLM 2–3x, tapi streamnya begini:

1) SSE langsung mulai → kirim `stage: "Oke, gue cek dulu ya…"` (langsung, tanpa nunggu LLM).  
2) Call-1 (planner / intent / keyword extractor) **boleh non-stream** biar cepat dan deterministik, tapi sambil itu BE tetap kirim stage update “Nangkep dulu maksud pertanyaannya…”.  
3) Retrieval DB → kirim stage “Lagi ambil konteks dari dokumen X…”.  
4) Call-2 (final answer) **stream** token jawaban sebagai `event: message` (ini yang tampil sebagai jawaban utama).  
5) Selesai → `event: done`.

Kalau butuh map-reduce summary (dokumen > context limit), kamu ulang: retrieval batch → stage “Lanjut bagian berikutnya…” → stream final synthesis. (Yang penting UI tahu ini masih request yang sama: pakai `request_id`.)  

## Perubahan minimal di backend (konsepnya)

Karena handler kamu sekarang hardcode `event("message")` untuk semua output, kamu perlu ubah `ConversationManager::handle_message` supaya return stream bertipe **enum event** (bukan `String`), misalnya:

- `ChatEvent::Stage { text, phase, request_id }`
- `ChatEvent::Delta { text, request_id }`
- `ChatEvent::Done { request_id }`

Lalu di `chat_stream_handler`, baru map:

- `Stage` → `Event::default().event("stage").data(json)`
- `Delta` → `Event::default().event("message").data(json)`
- `Done` → `event("done")`

Ini nyambung langsung dengan struktur handler SSE kamu sekarang yang memang sudah dukung multiple event type (`message/error/done`).

## Biar “kayak Gemini” (status terus berubah)

Trik yang aman tanpa “template ketahuan”:

- Simpan 20–40 kandidat status phrase di server, pilih berdasarkan hash(query + session_id + counter), lalu inject detail kecil: “dokumen: {title}”, “bagian: {n}”, “progress: {x}%”.
- Tambahkan “micro-variation”: sinonim + panjang kalimat beda, dan jangan kirim status tiap 200ms; kirimnya event-driven (mulai plan, selesai plan, mulai retrieval, mulai summarization, dst).

Kita implement “Gemini-like changing status” dengan **SSE event `stage` terpisah dari `message`**, jadi UI bisa render teks status yang berubah-ubah sambil tetap stream token jawaban final.
Ini patch production-grade karena client bisa bedain “status proses” vs “jawaban”, bukan sekadar nerima string mentah setiap kali.

## 1) Replace `src/handlers/chat.rs`

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
use crate::state::AppState;
use crate::utils::error::ApiError;
use axum::extract::Query;

// NEW
use chrono::Utc;
use crate::services::conversation::manager::ChatStreamChunk;

/// Handle streaming chat request
/// POST /api/chat/stream
pub async fn chat_stream_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ChatRequest>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, (axum::http::StatusCode, String)> {
    info!(?req, "Incoming chat request");

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
    let document_id = req.document_id;

    // NEW: request_id untuk tracking 1 user request (client bisa pakai ini buat UI state)
    let request_id = format!("{}-{}-{}", session_id, user_id, Utc::now().timestamp_millis());

    let stream = async_stream::stream! {
        match conversation_manager
            .handle_message(session_id, user_id, message, document_id, request_id.clone())
            .await
        {
            Ok(mut response_stream) => {
                use futures::StreamExt;

                while let Some(chunk_res) = response_stream.next().await {
                    match chunk_res {
                        Ok(chunk) => {
                            match chunk {
                                ChatStreamChunk::Stage { request_id, phase, text } => {
                                    let data = serde_json::to_string(&serde_json::json!({
                                        "request_id": request_id,
                                        "phase": phase,
                                        "text": text
                                    })).unwrap_or_else(|_| "{}".to_string());

                                    yield Ok(Event::default().event("stage").data(data));
                                }
                                ChatStreamChunk::Delta { request_id, delta } => {
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
                            }
                        }
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

    Json(CleanupResponse { sessions_removed: count })
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
                        let event = Event::default().event("system_event").data(data);
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

## 2) Replace `src/services/conversation/manager.rs` (bagian yang relevan untuk stream stage)

Ini versi lengkap `ChatStreamChunk` + perubahan signature `handle_message` + emitter stage + mapping stream LLM jadi `Delta`. (Bagian lain di file kamu biarkan sama, tapi **replace** sesuai ini supaya compile.)

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
use std::time::Instant;

use std::pin::Pin;
use futures::stream::Stream;

use crate::logging::{ActivityLogger, ActivityLog, ActivityType, ActivityStatus};

// NEW
use serde::Serialize;

// ========= NEW SSE CHUNK TYPES =========
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ChatStreamChunk {
    Stage {
        request_id: String,
        phase: String,
        text: String,
    },
    Delta {
        request_id: String,
        delta: String,
    },
    Done {
        request_id: String,
    },
}

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
    ) -> Result<Vec<RetrievalChunk>>;

    async fn get_document_metadata(&self, document_id: i32) -> Result<DocumentMetadata>;
    async fn get_document_overview_chunks(&self, document_id: i32, limit: i32) -> Result<Vec<RetrievalChunk>>;
    async fn get_document_overview(&self, document_id: i32, chunk_limit: i32) -> Result<DocumentOverview>;
}

/// Trait for LLM service
#[async_trait::async_trait]
pub trait LlmProvider: Send + Sync {
    async fn generate(&self, messages: &[ChatMessage]) -> Result<String>;
    async fn generate_stream(&self, messages: &[ChatMessage]) -> Result<Pin<Box<dyn Stream<Item = Result<String, anyhow::Error>> + Send>>>;
    async fn summarize_chunks(&self, chunks: &[RetrievalChunk], query: &str) -> Result<String>;
}

/// Chunk result from retrieval
#[derive(Debug, Clone)]
pub struct RetrievalChunk {
    pub chunk_id: i64,
    pub document_id: i64,
    pub document_title: Option<String>,
    pub content: String,
    pub similarity: f32,
}

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

    fn pick_stage_text(phase: &str, seed: &str) -> String {
        use std::hash::{Hash, Hasher};

        fn hash_u64(s: &str) -> u64 {
            let mut h = std::collections::hash_map::DefaultHasher::new();
            s.hash(&mut h);
            h.finish()
        }

        let k = hash_u64(&format!("{}|{}", phase, seed));
        let idx = (k % 6) as usize;

        // Variasi kalimat, biar gak kerasa template.
        let text = match phase {
            "understand" => [
                "Baik, saya pahami dahulu maksud pertanyaan Anda ya…",
                "Sip, saya pahami dahulu konteks pertanyaannya…",
                "Bentar, saya cerna dahulu permintaannya…",
                "Oke, saya cek dahulu kebutuhan jawaban yang Anda mau…",
                "Sip, saya pastikan dahulu arah jawabannya…",
                "Oke, saya rangkum dahulu intent pertanyaannya…",
            ][idx],
            "embed" => [
                "Sedang menyiapkan representasi pertanyaannya…",
                "Oke, saya proses pertanyaannya sebentar…",
                "Sedang memproses query Anda…",
                "Sip, saya hitung relevansi awalnya…",
                "Sedang menyusun pemahaman semantik…",
                "Sedang menyiapkan pencarian konteks…",
            ][idx],
            "retrieve" => [
                "Sedang mengambil konteks dari dokumen yang Anda pilih…",
                "Oke, saya cari bagian paling relevan di dokumen…",
                "Sedang membaca bagian-bagian penting dokumen…",
                "Sip, saya mengumpulkan kutipan yang relevan dahulu…",
                "Oke, saya memfilter konteks yang sesuai…",
                "Sedang menarik konteks terbaik dari dokumen…",
            ][idx],
            "answer" => [
                "Oke, saya susun jawabannya…",
                "Sip, saya mulai menjawabnya…",
                "Oke, saya merangkai jawaban dari konteks yang ada…",
                "Sip, saya menulis jawaban yang paling pas…",
                "Oke, saya menjawab dengan detail yang secukupnya…",
                "Sip, saya menyelesaikan jawabannya…",
            ][idx],
            _ => "Baik, saya proses dahulu ya…",
        };

        text.to_string()
    }

    fn stage(request_id: &str, phase: &str) -> ChatStreamChunk {
        ChatStreamChunk::Stage {
            request_id: request_id.to_string(),
            phase: phase.to_string(),
            text: Self::pick_stage_text(phase, request_id),
        }
    }

    pub async fn handle_message(
        self: std::sync::Arc<Self>,
        session_id: SessionId,
        user_id: i64,
        message: String,
        document_id: Option<i64>,
        request_id: String,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatStreamChunk, anyhow::Error>> + Send>>> {
        let start_time = Instant::now();

        self.logger.log(
            ActivityLog::builder(session_id, user_id, ActivityType::ProcessingStage)
                .message("PESAN MASUK")
                .build()
        );

        // Load session state
        let mut state = self.get_or_create_session(session_id, user_id, document_id).await?;

        self.logger.log(
            ActivityLog::builder(session_id, user_id, ActivityType::RequestReceived)
                .message(&message)
                .document_id(document_id.unwrap_or(0))
                .status(ActivityStatus::Info)
                .build()
        );

        // Sliding window
        if state.needs_window_enforcement() {
            self.logger.log(
                ActivityLog::builder(session_id, user_id, ActivityType::SlidingWindowEnforced)
                    .status(ActivityStatus::Warning)
                    .build()
            );
        }
        self.enforce_sliding_window(&mut state)?;

        let manager = self.clone();
        let mut final_state = state;

        let stream = async_stream::try_stream! {
            // === Stage: understand ===
            yield ConversationManager::stage(&request_id, "understand");

            // Embedding
            yield ConversationManager::stage(&request_id, "embed");

            let query_embedding = manager.embedding_provider
                .embed(&message)
                .await
                .context("Failed to generate embedding")?;

            // Iterative loop (existing behavior)
            let verifier = LlmVerifier::new(3);
            let tried_chunk_ids: HashSet<i64> = HashSet::new();
            let mut iteration = 0usize;
            const MAX_ITERATIONS: usize = 3;

            let mut context_metrics = ContextMetrics::default();
            let mut retrieval_duration_total = 0i32;

            let mut final_answer = String::new();

            loop {
                iteration += 1;

                if iteration > MAX_ITERATIONS {
                    warn!("Max iterations reached, returning best effort");
                    final_answer = "Maaf, saya tidak dapat menemukan informasi yang cukup setelah beberapa kali pencarian. Silakan coba pertanyaan yang lebih spesifik atau upload dokumen yang relevan.".to_string();
                    break;
                }

                // Decide retrieval
                let decision = manager.context_builder.decide_retrieval(
                    &final_state,
                    &message,
                    document_id,
                    Some(&query_embedding),
                )?;

                // === Stage: retrieve (kalau memang retrieve) ===
                if matches!(decision, RetrievalDecision::Retrieve { .. }) {
                    yield ConversationManager::stage(&request_id, "retrieve");
                }

                let retrieval_start = Instant::now();
                let (system_context, metrics) = manager.execute_retrieval_with_metrics(
                    &mut final_state,
                    &decision,
                    &message,
                    document_id,
                    &query_embedding,
                    &tried_chunk_ids,
                ).await?;

                let retrieval_duration = retrieval_start.elapsed().as_millis() as i32;
                retrieval_duration_total += retrieval_duration;
                context_metrics = metrics.clone();

                // Add user message (only once)
                if iteration == 1 {
                    final_state.messages.push(ChatMessage::user(&message));
                }

                // Token management
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

                // === Stage: answer ===
                yield ConversationManager::stage(&request_id, "answer");

                // Build messages
                let enhanced_system = verifier.build_verification_prompt(
                    manager.context_builder.base_instruction()
                );

                let mut llm_messages = vec![
                    ChatMessage::system(format!("{}\n\n{}", enhanced_system, system_context))
                ];
                llm_messages.extend(final_state.messages.clone());

                // Call LLM (stream)
                let llm_start = Instant::now();

                let llm_response = if manager.stream_enabled {
                    let mut s = manager.llm_provider.generate_stream(&llm_messages).await?;

                    use futures::StreamExt;
                    let mut accumulated = String::new();

                    // Stream token langsung (natural) sebagai Delta
                    while let Some(chunk_res) = s.next().await {
                        match chunk_res {
                            Ok(delta) => {
                                if !delta.is_empty() {
                                    accumulated.push_str(&delta);
                                    yield ChatStreamChunk::Delta {
                                        request_id: request_id.clone(),
                                        delta,
                                    };
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
                    manager.llm_provider.generate(&llm_messages).await?
                };

                let llm_duration = llm_start.elapsed().as_millis() as i32;

                // Verify (existing logic)
                match verifier.parse_response(&llm_response) {
                    VerificationResult::Answered(answer) => {
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
                                .document_id(document_id.unwrap_or(0))
                                .custom("retrieval_iterations", iteration as i64)
                                .custom("context_truncated", if context_metrics.truncated { 1i64 } else { 0i64 })
                                .custom("documents_retrieved", context_metrics.documents_included as i64)
                                .custom("chunks_used", context_metrics.chunks_included as i64)
                                .custom("verification_result", "answered")
                                .build()
                        );

                        break;
                    }
                    VerificationResult::NeedMoreContext { .. } => {
                        if iteration >= MAX_ITERATIONS {
                            final_answer = "Maaf, informasi dalam dokumen tidak cukup lengkap untuk menjawab pertanyaan Anda.".to_string();
                            break;
                        }
                        continue;
                    }
                    VerificationResult::NotRelevant { .. } => {
                        if iteration >= MAX_ITERATIONS {
                            final_answer = "Maaf, dokumen yang tersedia tidak relevan dengan pertanyaan Anda.".to_string();
                            break;
                        }
                        continue;
                    }
                }
            }

            // Kalau streaming on, kita sudah kirim delta. Tapi final_answer bisa beda dari streamed text (karena verifier).
            // Untuk konsistensi UX: kita tidak kirim ulang full final_answer sebagai delta.
            // Kita hanya update state + Done.
            final_state.messages.push(ChatMessage::assistant(&final_answer));
            final_state.last_query_embedding = Some(query_embedding);
            final_state.metadata.total_messages += 2;
            final_state.touch();

            manager.cache.set(session_id, final_state);

            yield ChatStreamChunk::Done { request_id: request_id.clone() };
        };

        Ok(Box::pin(stream))
    }

    // ================== BELOW: KEEP YOUR EXISTING IMPLEMENTATIONS ==================
    // execute_retrieval_with_metrics(...)
    // build_structured_rag_context(...)
    // execute_metadata_query(...)
    // get_or_create_session(...)
    // generate_session_id(...)
    // enforce_sliding_window(...)
    // manage_tokens(...)
    // call_llm_with_retry(...)
    // cache_stats/cleanup/logger/build_metadata_context(...)
    //
    // NOTE: method signatures yang dipanggil di atas harus tetap ada seperti versi kamu sekarang.
    // ================== END ==================
}
```

## 3) Client contract (wajib biar mulus)

Di client React, treat event:

- `stage`: update “status line” (replace text, bukan append).
- `message`: append ke bubble jawaban.
- `done`: stop spinner & mark completed.

Gaskeun—kita ubah SSE jadi **2 jalur event**: `stage` (status natural yang berubah-ubah) + `message` (delta jawaban), lalu `done` di akhir, supaya UI/UX jelas mana proses mana jawaban.

## Patch 1 — Replace `src/services/conversation/manager.rs`

Ini versi **full file** (replace total) yang tetap mempertahankan verifier loop kamu, tapi streaming ke client dilakukan lewat SSE event typed (status + delta), dan final answer di-stream **dari server** supaya konsisten dengan hasil verifikasi.

```rust
/// manager.rs
use anyhow::{Context, Result};
use futures::stream::Stream;
use serde::Serialize;
use std::collections::{HashSet, hash_map::DefaultHasher};
use std::hash::{Hash, Hasher};
use std::pin::Pin;
use std::time::Instant;
use tracing::{debug, error, info, warn};

use crate::database::models::{DocumentMetadata, DocumentOverview};
use crate::logging::{ActivityLog, ActivityLogger, ActivityStatus, ActivityType};
use crate::models::chat::{ChatMessage, SessionId};
use crate::services::rag_service::ContextMetrics;

use super::cache::ConversationCache;
use super::context_builder::ContextBuilder;
use super::token_counter::TokenCounter;
use super::types::{ConversationState, RetrievalDecision, RetrievalReason};
use super::verification::{LlmVerifier, VerificationResult};

/// ===== NEW: stream chunk types (typed SSE payload) =====
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ChatStreamChunk {
    Stage {
        request_id: String,
        phase: String,
        text: String,
    },
    Message {
        request_id: String,
        delta: String,
    },
    Done {
        request_id: String,
    },
}

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
    ) -> Result<Vec<RetrievalChunk>>;

    // Meta-question helpers
    async fn get_document_metadata(&self, document_id: i32) -> Result<DocumentMetadata>;
    async fn get_document_overview_chunks(
        &self,
        document_id: i32,
        limit: i32,
    ) -> Result<Vec<RetrievalChunk>>;
    async fn get_document_overview(&self, document_id: i32, chunk_limit: i32)
        -> Result<DocumentOverview>;
}

/// Trait for LLM service
#[async_trait::async_trait]
pub trait LlmProvider: Send + Sync {
    async fn generate(&self, messages: &[ChatMessage]) -> Result<String>;
    async fn generate_stream(
        &self,
        messages: &[ChatMessage],
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String, anyhow::Error>> + Send>>>;
    async fn summarize_chunks(&self, chunks: &[RetrievalChunk], query: &str) -> Result<String>;
}

/// Chunk result from retrieval
#[derive(Debug, Clone)]
pub struct RetrievalChunk {
    pub chunk_id: i64,
    pub document_id: i64,
    pub document_title: Option<String>,
    pub content: String,
    pub similarity: f32,
}

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

    fn stable_pick(seed: &str, phase: &str, n: usize) -> usize {
        let mut h = DefaultHasher::new();
        seed.hash(&mut h);
        phase.hash(&mut h);
        (h.finish() as usize) % n
    }

    fn stage_text(request_id: &str, phase: &str) -> String {
        // Variasi biar gak ketahuan template dan tetap natural.
        let options: &[&str] = match phase {
            "understand" => &[
                "Baik, Saya tangkap dulu maksud pertanyaan Anda ya…",
                "Sip, Saya pahamin dulu konteks pertanyaannya…",
                "Bentar ya, Saya cerna dulu permintaannya…",
                "Oke, Saya cek dulu kebutuhan jawabannya…",
                "Sip, Saya pastiin dulu arah jawabannya…",
                "Oke, Saya rangkum intent pertanyaannya…",
            ],
            "embed" => &[
                "Lagi proses pertanyaannya sebentar…",
                "Sip, Saya siapin pencarian konteksnya…",
                "Sebentar, Saya hitung relevansi semantiknya…",
                "Oke, Saya normalize query-nya dulu…",
                "Sip, Saya siapin vektor pencariannya…",
                "Oke, Saya susun pemahaman semantiknya…",
            ],
            "retrieve" => &[
                "Saya lagi ambil konteks dari dokumen yang Anda pilih…",
                "Oke, Saya cari bagian paling relevan dari dokumen…",
                "Sedang baca cuplikan penting dokumen…",
                "Sip, Saya kumpulin bagian yang relevan…",
                "Oke, Saya rapihin konteks biar pas…",
                "Sedang narik konteks terbaik dari dokumen…",
            ],
            "answer" => &[
                "Oke, Saya susun jawabannya…",
                "Sip, Saya mulai jawab ya…",
                "Oke, Saya rangkai jawaban dari konteks yang ada…",
                "Sip, Saya bikin jawabannya singkat tapi jelas…",
                "Oke, Saya finalisasi jawabannya…",
                "Sip, Saya tulis jawaban yang paling pas…",
            ],
            "finalize" => &[
                "Oke, Saya rapihin hasilnya…",
                "Sip, Saya finalize biar enak dibaca…",
                "Oke, sebentar ya, Saya beresin jawabannya…",
                "Sip, Saya kunci jawabannya…",
                "Oke, Saya cek sekali lagi…",
                "Sip, almost done…",
            ],
            _ => &["Oke, Saya proses dulu ya…"],
        };

        let idx = Self::stable_pick(request_id, phase, options.len());
        options[idx].to_string()
    }

    fn stage_event(request_id: &str, phase: &str) -> ChatStreamChunk {
        ChatStreamChunk::Stage {
            request_id: request_id.to_string(),
            phase: phase.to_string(),
            text: Self::stage_text(request_id, phase),
        }
    }

    fn stream_text_as_deltas(
        request_id: &str,
        text: &str,
        max_chars_per_delta: usize,
    ) -> Vec<ChatStreamChunk> {
        if text.is_empty() {
            return vec![];
        }

        let mut out = Vec::new();
        let mut buf = String::new();
        let mut count = 0usize;

        for ch in text.chars() {
            buf.push(ch);
            count += 1;

            if count >= max_chars_per_delta {
                out.push(ChatStreamChunk::Message {
                    request_id: request_id.to_string(),
                    delta: buf.clone(),
                });
                buf.clear();
                count = 0;
            }
        }

        if !buf.is_empty() {
            out.push(ChatStreamChunk::Message {
                request_id: request_id.to_string(),
                delta: buf,
            });
        }

        out
    }

    pub async fn handle_message(
        self: std::sync::Arc<Self>,
        session_id: SessionId,
        user_id: i64,
        message: String,
        document_id: Option<i64>,
        request_id: String,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatStreamChunk, anyhow::Error>> + Send>>> {
        let start_time = Instant::now();

        self.logger.log(
            ActivityLog::builder(session_id, user_id, ActivityType::ProcessingStage)
                .message("PESAN MASUK")
                .build(),
        );

        let mut state = self.get_or_create_session(session_id, user_id, document_id).await?;

        self.logger.log(
            ActivityLog::builder(session_id, user_id, ActivityType::RequestReceived)
                .message(&message)
                .document_id(document_id.unwrap_or(0))
                .status(ActivityStatus::Info)
                .build(),
        );

        if state.needs_window_enforcement() {
            self.logger.log(
                ActivityLog::builder(session_id, user_id, ActivityType::SlidingWindowEnforced)
                    .status(ActivityStatus::Warning)
                    .build(),
            );
        }
        self.enforce_sliding_window(&mut state)?;

        let manager = self.clone();
        let mut final_state = state;

        let stream = async_stream::try_stream! {
            // Stage awal: natural (langsung muncul di UI)
            yield ConversationManager::stage_event(&request_id, "understand");

            // Embedding
            yield ConversationManager::stage_event(&request_id, "embed");

            let query_embedding = manager.embedding_provider
                .embed(&message)
                .await
                .context("Failed to generate embedding")?;

            // Iterative retrieval loop (tetap pakai verifier kamu)
            let verifier = LlmVerifier::new(3);
            let mut tried_chunk_ids: HashSet<i64> = HashSet::new();
            let mut iteration = 0usize;
            const MAX_ITERATIONS: usize = 3;

            let mut context_metrics = ContextMetrics::default();
            let mut retrieval_duration_total = 0i32;

            let mut final_answer = String::new();

            loop {
                iteration += 1;

                if iteration > MAX_ITERATIONS {
                    warn!("Max iterations ({}) reached, returning best effort", MAX_ITERATIONS);
                    final_answer = "Maaf, saya tidak dapat menemukan informasi yang cukup setelah beberapa kali pencarian. Silakan coba pertanyaan yang lebih spesifik atau upload dokumen yang relevan.".to_string();
                    break;
                }

                info!("Retrieval iteration {}/{}", iteration, MAX_ITERATIONS);

                let decision = manager.context_builder.decide_retrieval(
                    &final_state,
                    &message,
                    document_id,
                    Some(&query_embedding),
                )?;

                if let RetrievalDecision::Skip { reason } = &decision {
                    if let super::types::SkipReason::SameDocumentAndHighSimilarity(sim) = reason {
                        manager.logger.log(
                            ActivityLog::builder(session_id, user_id, ActivityType::RetrievalSkipped)
                                .similarity(*sim)
                                .retrieval_skipped(true)
                                .build(),
                        );
                    }
                }

                if matches!(decision, RetrievalDecision::Retrieve { .. }) {
                    yield ConversationManager::stage_event(&request_id, "retrieve");
                }

                let retrieval_start = Instant::now();
                let (system_context, metrics) = manager.execute_retrieval_with_metrics(
                    &mut final_state,
                    &decision,
                    &message,
                    document_id,
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
                            .build(),
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
                            .build(),
                    );
                }

                // Stage menjelang jawab
                yield ConversationManager::stage_event(&request_id, "answer");

                let enhanced_system = verifier.build_verification_prompt(
                    manager.context_builder.base_instruction()
                );

                let mut llm_messages = vec![
                    ChatMessage::system(format!("{}\n\n{}", enhanced_system, system_context))
                ];
                llm_messages.extend(final_state.messages.clone());

                // NOTE:
                // Kita pakai non-stream call untuk internal loop,
                // agar output yang di-stream ke client konsisten dengan hasil verifikasi.
                let llm_start = Instant::now();
                manager.logger.log(
                    ActivityLog::builder(session_id, user_id, ActivityType::ProcessingStage)
                        .message(&format!("KIRIM KE MODEL UTAMA (Iteration {})", iteration))
                        .build(),
                );

                let llm_response = manager.call_llm_with_retry(&llm_messages).await?;

                manager.logger.log(
                    ActivityLog::builder(session_id, user_id, ActivityType::ProcessingStage)
                        .message("LLM SUDAH RESPONSE")
                        .build(),
                );

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
                                .document_id(document_id.unwrap_or(0))
                                .custom("retrieval_iterations", iteration as i64)
                                .custom("context_truncated", if context_metrics.truncated { 1i64 } else { 0i64 })
                                .custom("documents_retrieved", context_metrics.documents_included as i64)
                                .custom("chunks_used", context_metrics.chunks_included as i64)
                                .custom("verification_result", "answered")
                                .build(),
                        );

                        break;
                    }

                    VerificationResult::NeedMoreContext { doc_ids, reason } => {
                        warn!(
                            "Iteration {}: LLM needs more context from docs {:?}. Reason: {}",
                            iteration, doc_ids, reason
                        );

                        if iteration >= MAX_ITERATIONS {
                            final_answer = format!(
                                "Maaf, informasi dalam dokumen tidak cukup lengkap untuk menjawab pertanyaan Anda. {}",
                                if !reason.is_empty() { format!("Alasan: {}", reason) } else { String::new() }
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
                            break;
                        }

                        continue;
                    }
                }
            }

            yield ConversationManager::stage_event(&request_id, "finalize");

            // Stream final answer ke client (delta)
            if manager.stream_enabled {
                for ev in ConversationManager::stream_text_as_deltas(&request_id, &final_answer, 48) {
                    yield ev;
                }
            } else {
                yield ChatStreamChunk::Message {
                    request_id: request_id.clone(),
                    delta: final_answer.clone(),
                };
            }

            // Update state
            final_state.messages.push(ChatMessage::assistant(&final_answer));
            final_state.last_query_embedding = Some(query_embedding);
            final_state.metadata.total_messages += 2;
            final_state.touch();
            manager.cache.set(session_id, final_state);

            // Done
            yield ChatStreamChunk::Done { request_id: request_id.clone() };
        };

        Ok(Box::pin(stream))
    }

    async fn execute_retrieval_with_metrics(
        &self,
        state: &mut ConversationState,
        decision: &RetrievalDecision,
        current_message: &str,
        document_id: Option<i64>,
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

                    let cfg = self.context_builder.weighted_config();
                    self.embedding_provider
                        .embed_weighted(
                            current_message,
                            &context_text,
                            cfg.current_weight,
                            cfg.history_weight,
                        )
                        .await?
                } else {
                    current_embedding.to_vec()
                };

                let mut chunks = self.retrieval_provider
                    .search(state.user_id, &query_embedding, document_id)
                    .await
                    .context("Retrieval failed")?;

                chunks.retain(|c| !tried_chunk_ids.contains(&c.chunk_id));

                if chunks.is_empty() {
                    warn!("No new chunks found after filtering tried chunks");
                    return Ok((
                        "Tidak ada informasi tambahan yang ditemukan.".to_string(),
                        ContextMetrics::default(),
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
                            metrics.total_tokens, metrics.documents_included, metrics.chunks_included
                        ))
                        .build(),
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
                    .build(),
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
                    .build(),
            );

            Ok(system_context)
        } else {
            let msg = "Untuk menjawab pertanyaan tentang dokumen, silakan upload atau pilih dokumen terlebih dahulu.";
            state.system_context = msg.to_string();
            Ok(msg.to_string())
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
                .build(),
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

    async fn manage_tokens(&self, state: &mut ConversationState, system_context: &str) -> Result<()> {
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
                .build(),
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
                        warn!(
                            "LLM call failed (attempt {}): {}. Retrying...",
                            attempt, e
                        );
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

## Patch 2 — Replace `src/handlers/chat.rs`

Ini ganti handler SSE kamu yang sebelumnya cuma `event("message")` dan manual `done`.

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
use crate::state::AppState;
use crate::utils::error::ApiError;
use axum::extract::Query;

// NEW
use chrono::Utc;
use crate::services::conversation::manager::ChatStreamChunk;

/// POST /api/chat/stream
pub async fn chat_stream_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ChatRequest>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, (axum::http::StatusCode, String)> {
    info!(?req, "Incoming chat request");

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
    let document_id = req.document_id;

    let request_id = format!("{}-{}-{}", session_id, user_id, Utc::now().timestamp_millis());

    let stream = async_stream::stream! {
        match conversation_manager
            .handle_message(session_id, user_id, message, document_id, request_id.clone())
            .await
        {
            Ok(mut response_stream) => {
                use futures::StreamExt;

                while let Some(chunk_res) = response_stream.next().await {
                    match chunk_res {
                        Ok(chunk) => {
                            match chunk {
                                ChatStreamChunk::Stage { request_id, phase, text } => {
                                    let data = serde_json::to_string(&serde_json::json!({
                                        "request_id": request_id,
                                        "phase": phase,
                                        "text": text
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
                            }
                        }
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

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

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

    Json(CleanupResponse { sessions_removed: count })
}

/// GET /api/chat/logger/stats
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

/// GET /api/chat/events (existing)
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

## Kontrak UI (biar langsung kepakai)

- Event `stage`: update 1 baris status (replace text).
- Event `message`: append `delta` ke bubble jawaban.
- Event `done`: stop loading untuk `request_id` itu.

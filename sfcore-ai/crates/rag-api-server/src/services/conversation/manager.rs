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

/// ===== V2: stream chunk types =====
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ChatStreamChunk {
    Stage {
        request_id: String,
        phase: String,
        progress: u8,          // 0..=100
        text: String,          // status text (natural)
        detail: Option<String> // info kecil untuk variasi (mis. judul doc, iterasi, dll)
    },
    Message {
        request_id: String,
        delta: String,
    },
    Done {
        request_id: String,
    },
}

/// ===== Planner Types =====
#[derive(Debug, serde::Deserialize)]
struct PlannerOut {
    intent: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PlannerIntent {
    Metadata,
    Vector,
    Clarify,
}

impl PlannerIntent {
    fn from_str(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "metadata" => Self::Metadata,
            "clarify" => Self::Clarify,
            _ => Self::Vector,
        }
    }
}

/// Extract first JSON object substring from a possibly noisy LLM output.
/// Handles nested braces and braces inside JSON strings (with escapes).
fn extract_first_json_object(s: &str) -> Option<&str> {
    let mut start: Option<usize> = None;
    let mut depth: i32 = 0;

    let mut in_string = false;
    let mut escaped = false;

    for (i, ch) in s.char_indices() {
        if start.is_none() {
            if ch == '{' {
                start = Some(i);
                depth = 1;
                in_string = false;
                escaped = false;
            }
            continue;
        }

        // We are inside an object candidate
        if in_string {
            if escaped {
                escaped = false;
                continue;
            }
            match ch {
                '\\' => escaped = true,
                '"' => in_string = false,
                _ => {}
            }
            continue;
        }

        match ch {
            '"' => in_string = true,
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    let st = start?;
                    return Some(&s[st..=i]); // inclusive end
                }
            }
            _ => {}
        }
    }

    None
}

/// Trait for embedding service
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
        query_text: &str,
        document_id: Option<i64>,
        document_ids: Option<Vec<i64>>,
    ) -> Result<Vec<RetrievalChunk>>;

    async fn get_document_metadata(&self, document_id: i32) -> Result<DocumentMetadata>;
    async fn get_document_overview_chunks(&self, document_id: i32, limit: i32) -> Result<Vec<RetrievalChunk>>;
    async fn get_document_overview(&self, document_id: i32, chunk_limit: i32) -> Result<DocumentOverview>;
}

/// Trait for LLM service
#[async_trait::async_trait]
pub trait LlmProvider: Send + Sync {
    async fn generate(&self, messages: &[ChatMessage]) -> Result<String>;
    
    // NEW: planner-friendly generation
    async fn generate_with(
        &self,
        messages: &[ChatMessage],
        max_tokens: usize,
        temperature: f32,
    ) -> Result<String>;

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

    fn stage_text(request_id: &str, phase: &str, progress: u8, detail: Option<&str>) -> String {
        let options: &[&str] = match phase {
            "understand" => &[
                "Oke, gue tangkep dulu maksud pertanyaanmu ya…",
                "Sip, gue pahamin dulu konteks pertanyaannya…",
                "Bentar ya, gue cerna dulu permintaannya…",
                "Oke, gue cek dulu arah jawabannya…",
                "Sip, gue pastiin dulu kebutuhan jawabannya…",
                "Oke, gue interpret dulu intent pertanyaannya…",
            ],
            "plan" => &[
                "Oke, gue rencanain langkah pencariannya…",
                "Sip, gue tenuin strategi jawabnya…",
                "Bentar, gue cek perlu cari di dokumen atau metadata…",
                "Oke, gue filter dulu tipe informasinya…",
            ],
            "embed" => &[
                "Lagi proses pertanyaannya sebentar…",
                "Sip, gue siapin pencarian konteksnya…",
                "Bentar, gue hitung relevansi semantiknya…",
                "Oke, gue normalize query-nya dulu…",
                "Sip, gue siapin vektor pencariannya…",
                "Oke, gue susun pemahaman semantiknya…",
            ],
            "retrieve" => &[
                "Gue lagi ambil konteks dari dokumen yang kamu pilih…",
                "Oke, gue cari bagian paling relevan dari dokumen…",
                "Sedang baca cuplikan penting dokumen…",
                "Sip, gue kumpulin bagian yang nyambung…",
                "Oke, gue rapihin konteks biar pas…",
                "Sedang narik konteks terbaik dari dokumen…",
            ],
            "compose" => &[
                "Oke, gue susun jawabannya…",
                "Sip, gue mulai jawab ya…",
                "Oke, gue rangkai jawaban dari konteks yang ada…",
                "Sip, gue bikin jawabannya ringkas tapi jelas…",
                "Oke, gue finalisasi alur jawabannya…",
                "Sip, gue tulis jawaban yang paling pas…",
            ],
            "finalize" => &[
                "Oke, gue rapihin hasilnya…",
                "Sip, gue finalize biar enak dibaca…",
                "Oke, bentar ya, gue beresin jawabannya…",
                "Sip, gue kunci jawabannya…",
                "Oke, gue cek sekali lagi…",
                "Sip, almost done…",
            ],
            _ => &["Oke, gue proses dulu ya…"],
        };

        let idx = Self::stable_pick(request_id, phase, options.len());
        let base = options[idx];

        let mut suffix = String::new();
        if progress >= 60 && progress < 85 {
            suffix.push_str(" (sebentar lagi kelar)");
        }
        if let Some(d) = detail {
            suffix.push_str(&format!(" — {}", d));
        }

        format!("{}{}", base, suffix)
    }

    fn stage_event(request_id: &str, phase: &str, progress: u8, detail: Option<String>) -> ChatStreamChunk {
        ChatStreamChunk::Stage {
            request_id: request_id.to_string(),
            phase: phase.to_string(),
            progress,
            text: Self::stage_text(request_id, phase, progress, detail.as_deref()),
            detail,
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
        document_ids: Option<Vec<i64>>,
        request_id: String,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatStreamChunk, anyhow::Error>> + Send>>> {
        let start_time = Instant::now();

        // Merge legacy document_id into document_ids
        let mut final_doc_ids = document_ids.unwrap_or_default();
        if let Some(id) = document_id {
            if !final_doc_ids.contains(&id) {
                final_doc_ids.push(id);
            }
        }
        
        // Normalize: Sort and Dedup to ensure consistent state comparison
        final_doc_ids.sort_unstable();
        final_doc_ids.dedup();

        let effective_doc_ids = if final_doc_ids.is_empty() { None } else { Some(final_doc_ids) };

        self.logger.log(
            ActivityLog::builder(session_id, user_id, ActivityType::ProcessingStage)
                .message("PESAN MASUK")
                .build(),
        );

        let mut state = self.get_or_create_session(session_id, user_id, effective_doc_ids.clone()).await?;

        self.logger.log(
            ActivityLog::builder(session_id, user_id, ActivityType::RequestReceived)
                .message(&message)
                .document_id(
                    effective_doc_ids.as_ref()
                        .and_then(|ids| ids.first().copied())
                        .unwrap_or(0)
                )
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
            yield ConversationManager::stage_event(&request_id, "understand", 5, None);

            // === PLANNER CALL ===
            yield ConversationManager::stage_event(&request_id, "plan", 10, None);

            let planner_messages = vec![
                ChatMessage::system(
                    "You are a planning module for a RAG system.\n\
                    Return ONLY valid JSON exactly like: {\"intent\":\"metadata\"} or {\"intent\":\"vector\"} or {\"intent\":\"clarify\"}.\n\
                    No markdown. No extra keys."
                        .to_string(),
                ),
                ChatMessage::user(format!(
                    "message: {}\ndocument_ids: {:?}",
                    message, effective_doc_ids
                )),
            ];

            let planner_raw = manager.llm_provider
                .generate_with(&planner_messages, 160, 0.0)
                .await
                .unwrap_or_else(|_| "{\"intent\":\"vector\"}".to_string());

            let planner_json = extract_first_json_object(&planner_raw).unwrap_or(planner_raw.as_str());

            let planner_out = serde_json::from_str::<PlannerOut>(planner_json)
                .unwrap_or(PlannerOut { intent: "vector".to_string() });

            let planner_intent = PlannerIntent::from_str(&planner_out.intent);
            
            manager.logger.log(
                 ActivityLog::builder(session_id, user_id, ActivityType::ProcessingStage)
                    .message(&format!("PLANNER INTENT: {:?}", planner_intent))
                    .build(),
            );

            // === FAIL FAST CHECK: Require Document Selection ===
            if effective_doc_ids.is_none() && (planner_intent == PlannerIntent::Metadata || planner_intent == PlannerIntent::Vector) {
                 warn!("No documents selected for Document Query");
                 yield ConversationManager::stage_event(&request_id, "finalize", 100, Some("Dokumen belum dipilih.".to_string()));
                 yield ChatStreamChunk::Message { 
                     request_id: request_id.clone(), 
                     delta: "Silakan pilih atau upload dokumen terlebih dahulu agar saya dapat menjawab pertanyaan Anda berdasarkan konteks dokumen.".to_string() 
                 };
                 yield ChatStreamChunk::Done { request_id: request_id.clone() };
                 return;
            }

            // === EMBEDDING (Only if not metadata) ===
            let mut query_embedding: Option<Vec<f32>> = None;

            if planner_intent != PlannerIntent::Metadata {
                yield ConversationManager::stage_event(&request_id, "embed", 15, None);
                
                query_embedding = Some(
                    manager.embedding_provider
                        .embed(&message)
                        .await
                        .context("Failed to generate embedding")?
                );
                
                yield ConversationManager::stage_event(&request_id, "embed", 25, Some("Siap cari konteks…".to_string()));
            } else {
                 yield ConversationManager::stage_event(&request_id, "embed", 20, Some("Skip embedding (metadata only)".to_string()));
            }

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

                // === RETRIEVAL DECISION (Planner Aware) ===
                let decision = match planner_intent {
                    PlannerIntent::Metadata => RetrievalDecision::Retrieve {
                        reason: RetrievalReason::DocumentMetadataQuery,
                        context_aware: false,
                    },
                    PlannerIntent::Clarify => RetrievalDecision::Retrieve {
                        reason: RetrievalReason::ClarificationWithContext,
                        context_aware: true,
                    },
                    PlannerIntent::Vector => manager.context_builder.decide_retrieval(
                        &final_state,
                        &message,
                        effective_doc_ids.clone(),
                        query_embedding.as_ref(),
                    )?,
                };
                
                // Override for subsequent iterations to use standar vector search
                // (Only trust planner heavily on first iteration)
                let decision = if iteration > 1 && planner_intent == PlannerIntent::Metadata {
                     manager.context_builder.decide_retrieval(
                        &final_state,
                        &message,
                        effective_doc_ids.clone(),
                        query_embedding.as_ref(),
                    )?
                } else {
                    decision
                };

                if matches!(decision, RetrievalDecision::Retrieve { .. }) {
                    let d = effective_doc_ids.as_ref().map(|ids| format!("docs: {}", ids.len()));
                    yield ConversationManager::stage_event(&request_id, "retrieve", 35, d);
                }

                let retrieval_start = Instant::now();
                
                let emb_slice: &[f32] = query_embedding.as_deref().unwrap_or(&[]);
                
                let (system_context, metrics) = manager.execute_retrieval_with_metrics(
                    &mut final_state,
                    &decision,
                    &message,
                    effective_doc_ids.clone(),
                    emb_slice,
                    &mut tried_chunk_ids,
                ).await?;

                let retrieval_duration = retrieval_start.elapsed().as_millis() as i32;
                retrieval_duration_total += retrieval_duration;
                context_metrics = metrics.clone();

                // milestone stage: retrieval done
                yield ConversationManager::stage_event(
                    &request_id,
                    "retrieve",
                    60,
                    Some(format!("konteks: {} doc, {} chunk", metrics.documents_included, metrics.chunks_included))
                );

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

                yield ConversationManager::stage_event(&request_id, "compose", 75, None);

                let enhanced_system = verifier.build_verification_prompt(
                    manager.context_builder.base_instruction()
                );

                let mut llm_messages = vec![
                    ChatMessage::system(format!("{}\n\n{}", enhanced_system, system_context))
                ];
                llm_messages.extend(final_state.messages.clone());

                let llm_start = Instant::now();
                manager.logger.log(
                    ActivityLog::builder(session_id, user_id, ActivityType::ProcessingStage)
                        .message(&format!("KIRIM KE MODEL UTAMA (Iteration {})", iteration))
                        .build(),
                );

                // Internal: non-stream biar hasil final konsisten (verifier bisa revise)
                let llm_response = manager.call_llm_with_retry(&llm_messages).await?;

                let llm_duration = llm_start.elapsed().as_millis() as i32;

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
                                .document_id(
                                    effective_doc_ids.as_ref()
                                        .and_then(|ids| ids.first().copied())
                                        .unwrap_or(0)
                                )
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
                        warn!("NeedMoreContext: {:?} {}", doc_ids, reason);

                        yield ConversationManager::stage_event(
                            &request_id,
                            "retrieve",
                            45, // turun sedikit, kasih kesan "ambil lagi"
                            Some("Butuh konteks tambahan…".to_string())
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
                        warn!("NotRelevant: {}", reason);

                        yield ConversationManager::stage_event(
                            &request_id,
                            "retrieve",
                            45,
                            Some("Coba cari konteks lain yang lebih nyambung…".to_string())
                        );

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

            yield ConversationManager::stage_event(&request_id, "finalize", 90, None);

            // Stream final answer (server-side streaming)
            let deltas = ConversationManager::stream_text_as_deltas(&request_id, &final_answer, 48);
            let total = deltas.len().max(1);

            for (i, ev) in deltas.into_iter().enumerate() {
                // progress 92..99 selama mengetik
                let p = 92 + (((i + 1) * 7) / total).min(7) as u8;
                if i == 0 {
                    yield ConversationManager::stage_event(&request_id, "finalize", p, Some("Ngetik jawabannya…".to_string()));
                }
                yield ev;
            }

            // Update state
            final_state.messages.push(ChatMessage::assistant(&final_answer));
            // Only update last_query_embedding if we actually generated one
            if let Some(emb) = query_embedding {
                final_state.last_query_embedding = Some(emb);
            }
            final_state.metadata.total_messages += 2;
            final_state.touch();
            manager.cache.set(session_id, final_state);

            yield ConversationManager::stage_event(&request_id, "finalize", 100, Some("Selesai.".to_string()));
            yield ChatStreamChunk::Done { request_id: request_id.clone() };
        };

        Ok(Box::pin(stream))
    }

    async fn execute_retrieval_with_metrics(
        &self,
        state: &mut ConversationState,
        decision: &RetrievalDecision,
        current_message: &str,
        document_ids: Option<Vec<i64>>,
        current_embedding: &[f32],
        tried_chunk_ids: &mut HashSet<i64>,
    ) -> Result<(String, ContextMetrics)> {
        match decision {
            RetrievalDecision::Skip { reason } => {
                debug!("Skipping retrieval: {:?}", reason);
                state.metadata.retrieval_skipped_count += 1;
                Ok((state.system_context.clone(), ContextMetrics::default()))
            }

            RetrievalDecision::Retrieve { reason, context_aware } => {
                if matches!(reason, RetrievalReason::DocumentMetadataQuery) {
                    let context = self.execute_metadata_query(state, document_ids).await?;
                    return Ok((context, ContextMetrics::default()));
                }

                state.metadata.total_retrievals += 1;

                let query_embedding = if *context_aware {
                    let context_text = self.context_builder
                        .prepare_context_aware_text(current_message, &state.messages);

                    let cfg = self.context_builder.weighted_config();
                    self.embedding_provider
                        .embed_weighted(current_message, &context_text, cfg.current_weight, cfg.history_weight)
                        .await?
                } else {
                    current_embedding.to_vec()
                };

                let mut chunks = self.retrieval_provider
                    .search(
                        state.user_id, 
                        &query_embedding, 
                        current_message, 
                        None, 
                        document_ids.clone()
                    )
                    .await
                    .context("Retrieval failed")?;

                chunks.retain(|c| !tried_chunk_ids.contains(&c.chunk_id));

                if chunks.is_empty() {
                    return Ok(("Tidak ada informasi tambahan yang ditemukan.".to_string(), ContextMetrics::default()));
                }

                let doc_chunks: Vec<crate::database::DocumentChunk> = chunks.iter().map(|c| {
                    tried_chunk_ids.insert(c.chunk_id); // Record usage
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
                state.document_ids = document_ids;

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
            return Ok(("Tidak ada konteks yang relevan ditemukan.".to_string(), ContextMetrics::default()));
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
                    "<chunk id=\"chunk_{}\" page=\"{}\" similarity=\"{:.3}\">\n{}\n</chunk>\n\n",
                    chunk.chunk_id, chunk.page_number.unwrap_or(0), chunk.similarity, chunk.content.trim()
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
        document_ids: Option<Vec<i64>>,
    ) -> Result<String> {
        // Only fetch metadata if there is at least one document
        if let Some(ids) = &document_ids {
             if let Some(&first_doc_id) = ids.first() {
                let overview = self.retrieval_provider
                    .get_document_overview(first_doc_id as i32, 5)
                    .await
                    .context("Failed to fetch document overview")?;

                let context_text = self.build_metadata_context(&overview);
                let system_context = self.context_builder.build_system_context(
                    &context_text,
                    Some(&format!("Document: {}", overview.metadata.title)),
                );

                state.system_context = system_context.clone();
                state.last_retrieval_summary = context_text;
                state.document_ids = document_ids.clone();
                state.metadata.total_retrievals += 1;

                return Ok(system_context);
            }
        }
        
        let msg = "Untuk menjawab pertanyaan tentang dokumen, silakan upload atau pilih dokumen terlebih dahulu.";
        state.system_context = msg.to_string();
        Ok(msg.to_string())
    }

    pub async fn get_or_create_session(
        &self,
        session_id: SessionId,
        user_id: i64,
        document_ids: Option<Vec<i64>>,
    ) -> Result<ConversationState> {
        if let Some(mut state) = self.cache.get(session_id) {
            // Update document scope if changed
            if state.document_ids != document_ids {
                 state.document_ids = document_ids;
                 // Persist the change to cache!
                 self.cache.set(session_id, state.clone());
            }
            return Ok(state);
        }

        if !self.cache.can_create_new_session() {
            anyhow::bail!("Memory limit reached (90%), cannot create new session");
        }

        self.logger.log(
            ActivityLog::builder(session_id, user_id, ActivityType::SessionCreated)
                .status(ActivityStatus::Info)
                .build(),
        );

        let state = ConversationState::new(session_id, user_id, document_ids);
        self.cache.set(session_id, state.clone());
        Ok(state)
    }

    pub fn generate_session_id(user_id: i64) -> SessionId {
        let now = chrono::Utc::now();
        let timestamp = now.format("%Y%m%d%H%M%S").to_string();
        format!("{}{}", timestamp, user_id).parse().expect("Failed to parse session_id")
    }

    fn enforce_sliding_window(&self, state: &mut ConversationState) -> Result<()> {
        if !state.needs_window_enforcement() {
            return Ok(());
        }

        if state.messages.len() >= 2 {
            state.messages.drain(0..2);
        }

        Ok(())
    }

    async fn manage_tokens(&self, state: &mut ConversationState, system_context: &str) -> Result<()> {
        let token_count = TokenCounter::count_payload(system_context, &state.messages, "");
        state.metadata.total_tokens_last = token_count.total;

        if !token_count.is_over_soft_limit() {
            return Ok(());
        }

        while state.messages.len() >= 2 && TokenCounter::count_payload(system_context, &state.messages, "").total > 20_000 {
            state.messages.drain(0..2);
        }

        Ok(())
    }

    async fn call_llm_with_retry(&self, messages: &[ChatMessage]) -> Result<String> {
        const MAX_RETRIES: u32 = 3;

        for attempt in 1..=MAX_RETRIES {
            match self.llm_provider.generate(messages).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    if attempt < MAX_RETRIES {
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
        context
    }
}
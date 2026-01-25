/// manager.rs
use anyhow::{Context, Result};
use tracing::{debug, error, info, warn};
use crate::models::chat::{ChatMessage, SessionId};
use crate::database::models::{DocumentMetadata, DocumentOverview};
use super::cache::ConversationCache;
use super::context_builder::ContextBuilder;
use super::token_counter::TokenCounter;
use super::types::{ConversationState, RetrievalDecision, RetrievalReason};

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

    // ============ NEW METHODS FOR META-QUESTIONS ============
    
    /// Get document metadata (for overview questions)
    async fn get_document_metadata(&self, document_id: i32) -> Result<DocumentMetadata>;
    
    /// Get first N chunks of document (for overview generation)
    async fn get_document_overview_chunks(&self, document_id: i32, limit: i32) -> Result<Vec<RetrievalChunk>>;
    
    /// Get complete document overview (metadata + first chunks)
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

    pub fn generate_session_id(user_id: i64) -> SessionId {
        let now = chrono::Utc::now();
        let timestamp = now.format("%Y%m%d%H%M%S").to_string();
        format!("{}{}", timestamp, user_id)
            .parse()
            .expect("Failed to parse session_id")
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
        
        // Log session creation
        self.logger.log(
            ActivityLog::builder(session_id, user_id, ActivityType::SessionCreated)
                .status(ActivityStatus::Info)
                .build()
        );
        
        let state = ConversationState::new(session_id, user_id, document_id);
        self.cache.set(session_id, state.clone());

        Ok(state)
    }

    pub async fn handle_message(
        self: std::sync::Arc<Self>,
        session_id: SessionId,
        user_id: i64,
        message: String,
        document_id: Option<i64>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String, anyhow::Error>> + Send>>> { // 1. Log Entry (Persistent)
        let start_time = Instant::now();
        
        self.logger.log(
            ActivityLog::builder(session_id, user_id, ActivityType::ProcessingStage)
                .message("PESAN MASUK")
                .build()
        );

        // 2. Load History
        let mut state = self.get_or_create_session(session_id, user_id, document_id).await?;
        self.logger.log(
            ActivityLog::builder(session_id, user_id, ActivityType::RequestReceived)
                .message(&message)
                .document_id(document_id.unwrap_or(0))
                .status(ActivityStatus::Info)
                .build()
        );

        // 2. Sliding Window Log
        if state.needs_window_enforcement() {
            self.logger.log(
                ActivityLog::builder(session_id, user_id, ActivityType::SlidingWindowEnforced)
                    .status(ActivityStatus::Warning)
                    .build()
            );
        }

        self.enforce_sliding_window(&mut state)?;

        // 3. Generate Embedding
        self.logger.log(
            ActivityLog::builder(session_id, user_id, ActivityType::ProcessingStage)
                .message("KIRIM KE MODEL EMBEDDING")
                .build()
        );

        let current_embedding = self.embedding_provider
            .embed(&message)
            .await
            .context("Failed to generate embedding")?;

        self.logger.log(
            ActivityLog::builder(session_id, user_id, ActivityType::ProcessingStage)
                .message("SELESAI MODEL EMBEDDING")
                .build()
        );

        let decision = self.context_builder.decide_retrieval(
            &state,
            &message,
            document_id,
            Some(&current_embedding),
        )?;

        // 3. Log Retrieval Decision (Skip)
        if let RetrievalDecision::Skip { reason } = &decision {
             if let super::types::SkipReason::SameDocumentAndHighSimilarity(sim) = reason {
                self.logger.log(
                    ActivityLog::builder(session_id, user_id, ActivityType::RetrievalSkipped)
                        .similarity(*sim)
                        .retrieval_skipped(true)
                        .build()
                );
            }
        }

        let retrieval_start = Instant::now();
        let system_context = self.execute_retrieval_decision(
            &mut state,
            &decision,
            &message,
            document_id,
            &current_embedding,
        ).await?;
        let retrieval_duration = retrieval_start.elapsed().as_millis() as i32;

        // 4. Log Retrieval Executed
        if matches!(decision, RetrievalDecision::Retrieve { .. }) {
            self.logger.log(
                ActivityLog::builder(session_id, user_id, ActivityType::RetrievalExecuted)
                    .retrieval_duration(retrieval_duration)
                    .retrieval_skipped(false)
                    .build()
            );
        }

        state.messages.push(ChatMessage::user(&message));

        // 5. Token Management Log
        let token_count_before = state.metadata.total_tokens_last;
        self.manage_tokens(&mut state, &system_context).await?;
        let token_count_after = state.metadata.total_tokens_last;

        if token_count_before > 24_000 {
            self.logger.log(
                ActivityLog::builder(session_id, user_id, ActivityType::TokenOverflow)
                    .status(ActivityStatus::Warning)
                    .token_count(token_count_before as i32)
                    .build()
            );
        }

        let llm_messages = self.prepare_llm_payload(&state, &system_context);
        let llm_start = Instant::now();
        
        // Clone things needed for the async stream block
        let manager = self.clone();
        // State is moved into the block

        let stream = async_stream::try_stream! {
            let mut full_response = String::new();

            if manager.stream_enabled {
                manager.logger.log(
                    ActivityLog::builder(session_id, user_id, ActivityType::ProcessingStage)
                        .message("KIRIM KE MODEL UTAMA")
                        .build()
                );
                
                let mut stream = manager.llm_provider.generate_stream(&llm_messages).await?;
                
                manager.logger.log(
                    ActivityLog::builder(session_id, user_id, ActivityType::ProcessingStage)
                        .message("LLM SUDAH RESPONSE")
                        .build()
                );
                
                use futures::StreamExt;
                while let Some(chunk_res) = stream.next().await {
                    match chunk_res {
                        Ok(chunk) => {
                            full_response.push_str(&chunk);
                            yield chunk;
                        }
                        Err(e) => {
                            // Convert error to anyhow::Error and yield (implicitly by ? in try_stream)
                            Err(e)?;
                        }
                    }
                }
            } else {
                manager.logger.log(
                    ActivityLog::builder(session_id, user_id, ActivityType::ProcessingStage)
                        .message("KIRIM KE MODEL UTAMA")
                        .build()
                );
                
                let response = manager.call_llm_with_retry(&llm_messages).await?;
                
                manager.logger.log(
                    ActivityLog::builder(session_id, user_id, ActivityType::ProcessingStage)
                        .message("LLM SUDAH RESPONSE")
                        .build()
                );
                
                full_response = response.clone();
                yield response;
            }
            
            info!("DONE-MEMBERIKAN-JAWABAN - LLM response completed");

            // Post-processing (Save History & Log)
            let llm_duration = llm_start.elapsed().as_millis() as i32;
            let total_duration = start_time.elapsed().as_millis() as i32;

            // Update state
            let mut final_state = state;
            final_state.messages.push(ChatMessage::assistant(&full_response));
            final_state.last_query_embedding = Some(current_embedding);
            final_state.metadata.total_messages += 2;
            final_state.touch();

            manager.cache.set(session_id, final_state);

            // 7. Log Final Completion (MessageSent)
            manager.logger.log(
                ActivityLog::builder(session_id, user_id, ActivityType::MessageSent)
                    .message(&message)
                    .response(&full_response)
                    .token_count(token_count_after as i32)
                    .processing_time(total_duration)
                    .llm_duration(llm_duration)
                    .retrieval_duration(retrieval_duration)
                    .document_id(document_id.unwrap_or(0))
                    .build()
            );
        };

        Ok(Box::pin(stream))
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

    async fn execute_retrieval_decision(
        &self,
        state: &mut ConversationState,
        decision: &RetrievalDecision,
        current_message: &str,
        document_id: Option<i64>,
        current_embedding: &[f32],
    ) -> Result<String> {
        match decision {
            RetrievalDecision::Skip { reason } => {
                debug!("Skipping retrieval: {:?}", reason);
                state.metadata.retrieval_skipped_count += 1;
                Ok(state.system_context.clone())
            }
            RetrievalDecision::Retrieve { reason, context_aware } => {
                match reason {
                    // ============ NEW: Handle DocumentMetadataQuery ============
                    RetrievalReason::DocumentMetadataQuery => {
                        info!("Processing document metadata query (overview/summary question)");
                        
                        if let Some(doc_id) = document_id {
                            // Log metadata retrieval start
                            self.logger.log(
                                ActivityLog::builder(state.session_id, state.user_id, ActivityType::ProcessingStage)
                                    .message("FETCH DOCUMENT METADATA")
                                    .build()
                            );
                            
                            // Get document overview (metadata + first 5 chunks)
                            let overview = self.retrieval_provider
                                .get_document_overview(doc_id as i32, 5)
                                .await
                                .context("Failed to fetch document overview")?;
                            
                            // Build context from metadata + first chunks
                            let context_text = self.build_metadata_context(&overview);
                            
                            // Build system context (no need for LLM summarization here)
                            let system_context = self.context_builder.build_system_context(
                                &context_text,
                                Some(&format!("Document: {}", overview.metadata.title)),
                            );
                            
                            state.system_context = system_context.clone();
                            state.last_retrieval_summary = context_text;
                            state.document_id = document_id;
                            state.metadata.total_retrievals += 1;
                            
                            // Log completion
                            self.logger.log(
                                ActivityLog::builder(state.session_id, state.user_id, ActivityType::ProcessingStage)
                                    .message("METADATA FETCH COMPLETED")
                                    .build()
                            );
                            
                            return Ok(system_context);
                        } else {
                            // No document_id provided
                            let error_msg = "Untuk menjawab pertanyaan tentang dokumen, \
                                            silakan upload atau pilih dokumen terlebih dahulu.";
                            
                            state.system_context = error_msg.to_string();
                            return Ok(error_msg.to_string());
                        }
                    }
                    // ============ END NEW ============

                    _ => {
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

                // Catch retrieval errors
                let chunks = match self.retrieval_provider
                    .search(state.user_id, &query_embedding, document_id)
                    .await {
                        Ok(res) => res,
                        Err(e) => {
                            self.logger.log(
                                ActivityLog::builder(state.session_id, state.user_id, ActivityType::RetrievalError)
                                    .status(ActivityStatus::Error)
                                    .error(e.to_string(), "RetrievalProviderError")
                                    .build()
                            );
                            error!("Retrieval provider failed: {:?}", e);
                            return Err(e).context("Retrieval failed");
                        }
                    };

                let summary = self.llm_provider
                    .summarize_chunks(&chunks, current_message)
                    .await
                    .context("Failed to summarize chunks")?;

                let system_context = self.context_builder.build_system_context(
                    &summary,
                    document_id.map(|id| format!("Document ID: {}", id)).as_deref(),
                );
                
                // Log Prompt Generation
                self.logger.log(
                    ActivityLog::builder(state.session_id, state.user_id, ActivityType::ProcessingStage)
                        .message("GENERATE MESSAGE PROMPT + CHUNK")
                        .build()
                );

                state.system_context = system_context.clone();
                state.last_retrieval_summary = summary;
                state.document_id = document_id;

                Ok(system_context)
                    }
                }
            }
        }
    }

    async fn manage_tokens(
        &self,
        state: &mut ConversationState,
        system_context: &str,
    ) -> Result<()> {
        let token_count = TokenCounter::count_payload(
            system_context,
            &state.messages,
            "",
        );

        debug!("Token count: {} (system: {}, history: {})", 
            token_count.total, token_count.system_tokens, token_count.history_tokens);

        state.metadata.total_tokens_last = token_count.total;

        if !token_count.is_over_soft_limit() {
            return Ok(());
        }

        warn!("Token count {} exceeds 20K, performing cascade deletion", token_count.total);
        
        // Log cascade deletion start
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
            
            let new_count = TokenCounter::count_payload(
                system_context,
                &state.messages,
                "",
            );
            current_count = new_count.total;
            
            debug!("After deletion round {}: {} tokens", deletion_round, current_count);
            deletion_round += 1;

            if state.messages.is_empty() {
                warn!("All history deleted, only current message remains");
                break;
            }
        }

        if current_count > 23_000 {
            warn!("Token count {} still over 23K after deletion, truncating retrieval", current_count);
            
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

    fn prepare_llm_payload(
        &self,
        state: &ConversationState,
        system_context: &str,
    ) -> Vec<ChatMessage> {
        let mut messages = vec![ChatMessage::system(system_context)];
        messages.extend(state.messages.clone());
        messages
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

    /// Build context from document metadata and first chunks
    /// Used for meta-questions like "what is this document about?"
    fn build_metadata_context(&self, overview: &DocumentOverview) -> String {
        let metadata = &overview.metadata;
        
        let mut context = String::new();
        
        // Add document metadata
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
        
        // Add first chunks as preview
        if !overview.first_chunks.is_empty() {
            context.push_str("\n=== CUPLIKAN AWAL DOKUMEN ===\n\n");
            
            for (i, chunk) in overview.first_chunks.iter().enumerate() {
                // Limit preview to first 300 characters per chunk
                let preview = chunk.content
                    .chars()
                    .take(300)
                    .collect::<String>();
                
                let ellipsis = if chunk.content.len() > 300 { "..." } else { "" };
                
                context.push_str(&format!("[Bagian {}]\n{}{}\n\n", i + 1, preview, ellipsis));
            }
        }
        
        context
    }
}

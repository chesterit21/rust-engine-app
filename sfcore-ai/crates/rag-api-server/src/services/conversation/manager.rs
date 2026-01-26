/// manager.rs
use anyhow::{Context, Result};
use tracing::{debug, error, info, warn};
use crate::models::chat::{ChatMessage, SessionId};
use crate::database::models::{DocumentMetadata, DocumentOverview};
use super::cache::ConversationCache;
use super::context_builder::ContextBuilder;
use super::token_counter::TokenCounter;
use super::types::{ConversationState, RetrievalDecision, RetrievalReason};
use super::verification::{LlmVerifier, VerificationResult};
use crate::services::rag_service::ContextMetrics;
use std::collections::HashSet;


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
    
    pub async fn handle_message(
        self: std::sync::Arc<Self>,
        session_id: SessionId,
        user_id: i64,
        message: String,
        document_id: Option<i64>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String, anyhow::Error>> + Send>>> {
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

        // ====== NEW: ITERATIVE RETRIEVAL LOOP ======
        let verifier = LlmVerifier::new(3); // Max 3 iterations
        let mut tried_chunk_ids: HashSet<i64> = HashSet::new();
        let mut iteration = 0;
        const MAX_ITERATIONS: usize = 3;
        
        let mut context_metrics = ContextMetrics::default();
        let mut retrieval_duration_total = 0i32;
        
        // Clone things for the async stream
        let manager = self.clone();
        let mut final_state = state;
        
        let stream = async_stream::try_stream! {
            let mut full_response = String::new();
            let mut final_answer = String::new();
            
            // ITERATIVE LOOP
            loop {
                iteration += 1;
                
                if iteration > MAX_ITERATIONS {
                    warn!("Max iterations ({}) reached, returning best effort", MAX_ITERATIONS);
                    final_answer = "Maaf, saya tidak dapat menemukan informasi yang cukup setelah beberapa kali pencarian. Silakan coba pertanyaan yang lebih spesifik atau upload dokumen yang relevan.".to_string();
                    break;
                }
                
                info!("Retrieval iteration {}/{}", iteration, MAX_ITERATIONS);
                
                // Decide retrieval strategy
                let decision = manager.context_builder.decide_retrieval(
                    &final_state,
                    &message,
                    document_id,
                    Some(&query_embedding),
                )?;
                
                // Log skip decision
                if let RetrievalDecision::Skip { reason } = &decision {
                    if let super::types::SkipReason::SameDocumentAndHighSimilarity(sim) = reason {
                        manager.logger.log(
                            ActivityLog::builder(session_id, user_id, ActivityType::RetrievalSkipped)
                                .similarity(*sim)
                                .retrieval_skipped(true)
                                .build()
                        );
                    }
                }
                
                // Execute retrieval
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
                
                // Log retrieval executed
                if matches!(decision, RetrievalDecision::Retrieve { .. }) {
                    manager.logger.log(
                        ActivityLog::builder(session_id, user_id, ActivityType::RetrievalExecuted)
                            .retrieval_duration(retrieval_duration)
                            .retrieval_skipped(false)
                            .build()
                    );
                }
                
                // Add user message (only on first iteration)
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
                
                // Build LLM messages with enhanced prompt
                let enhanced_system = verifier.build_verification_prompt(
                    &manager.context_builder.base_instruction()
                );
                
                let mut llm_messages = vec![
                    ChatMessage::system(format!("{}\n\n{}", enhanced_system, system_context))
                ];
                llm_messages.extend(final_state.messages.clone());
                
                // Call LLM
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
                    
                    // Collect streaming response
                    use futures::StreamExt;
                    let mut accumulated = String::new();
                    
                    while let Some(chunk_res) = stream.next().await {
                        match chunk_res {
                            Ok(chunk) => {
                                accumulated.push_str(&chunk);
                                // Stream to client ONLY on final iteration (avoid confusion)
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
                
                // VERIFY LLM RESPONSE
                match verifier.parse_response(&llm_response) {
                    VerificationResult::Answered(answer) => {
                        info!("LLM successfully answered on iteration {}", iteration);
                        final_answer = answer;
                        
                        // Log success
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
                        
                        break; // Exit loop
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
                        
                        // Mark chunks as tried (to avoid re-fetching same chunks)
                        // Note: We don't have direct access to chunk IDs from context here
                        // This is handled in execute_retrieval_with_metrics
                        
                        // Continue loop for next iteration
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
                        
                        // Try expanding search (implementation depends on your needs)
                        // For now, continue with same strategy
                        continue;
                    }
                }
            }
            
            // Stream final answer (if not streamed already)
            if !manager.stream_enabled || iteration < MAX_ITERATIONS {
                yield final_answer.clone();
            }
            
            // Update state with assistant message
            final_state.messages.push(ChatMessage::assistant(&final_answer));
            final_state.last_query_embedding = Some(query_embedding);
            final_state.metadata.total_messages += 2;
            final_state.touch();
            
            // Save state
            manager.cache.set(session_id, final_state);
        };

        Ok(Box::pin(stream))
    }
    
    // ADD NEW METHOD for retrieval with metrics
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
                // Handle special cases (metadata queries, etc)
                if matches!(reason, RetrievalReason::DocumentMetadataQuery) {
                    // Use existing metadata query logic (already in your code)
                    let context = self.execute_metadata_query(state, document_id).await?;
                    return Ok((context, ContextMetrics::default()));
                }
                
                info!("Performing retrieval: {:?}", reason);
                state.metadata.total_retrievals += 1;
                
                // Generate query embedding (context-aware if needed)
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
                
                // Retrieve chunks
                let mut chunks = self.retrieval_provider
                    .search(state.user_id, &query_embedding, document_id)
                    .await
                    .context("Retrieval failed")?;
                
                // Filter out already-tried chunks
                chunks.retain(|c| !tried_chunk_ids.contains(&c.chunk_id));
                
                if chunks.is_empty() {
                    warn!("No new chunks found after filtering tried chunks");
                    return Ok((
                        "Tidak ada informasi tambahan yang ditemukan.".to_string(),
                        ContextMetrics::default()
                    ));
                }
                
                // Convert to DocumentChunk for structured context building
                let doc_chunks: Vec<crate::database::DocumentChunk> = chunks.iter().map(|c| {
                    crate::database::DocumentChunk {
                        chunk_id: c.chunk_id,
                        document_id: c.document_id as i32,
                        document_title: c.document_title.clone().unwrap_or_default(),
                        content: c.content.clone(),
                        similarity: c.similarity,
                        chunk_index: 0, // Not used in context building
                        page_number: None,
                    }
                }).collect();
                
                // Build structured context using RagService
                let (context, metrics) = self.build_structured_rag_context(doc_chunks)?;
                
                // Log generation
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
                
                // Update state
                state.system_context = context.clone();
                state.last_retrieval_summary = context.clone();
                state.document_id = document_id;
                
                Ok((context, metrics))
            }
        }
    }
    
    // ADD helper method to use RagService's structured context
    fn build_structured_rag_context(
        &self,
        chunks: Vec<crate::database::DocumentChunk>,
    ) -> Result<(String, ContextMetrics)> {
        // We need access to RagService here
        // Since RagService is behind Box<dyn RetrievalProvider>, we can't call it directly
        // Solution: Add build_structured_context to config or duplicate logic
        
        // For now, duplicate the logic (or refactor RagService as concrete type)
        use crate::utils::token_estimator;
        use std::collections::HashMap;
        
        if chunks.is_empty() {
            return Ok((
                "Tidak ada konteks yang relevan ditemukan.".to_string(),
                ContextMetrics::default(),
            ));
        }
        
        // Group by document
        let mut grouped: HashMap<i32, Vec<crate::database::DocumentChunk>> = HashMap::new();
        for chunk in chunks {
            grouped.entry(chunk.document_id).or_default().push(chunk);
        }
        
        // Build context
        let max_tokens = 20_000; // From config
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
    
    // Extract metadata query logic
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



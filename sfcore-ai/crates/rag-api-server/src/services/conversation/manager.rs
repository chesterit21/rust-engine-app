use anyhow::{Context, Result};
use tracing::{debug, error, info, warn};
use crate::models::chat::ChatMessage;
use super::cache::ConversationCache;
use super::context_builder::ContextBuilder;
use super::token_counter::TokenCounter;
use super::types::{ConversationState, RetrievalDecision};
use crate::models::chat::SessionId;

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
}

/// Trait for LLM service
#[async_trait::async_trait]
pub trait LlmProvider: Send + Sync {
    async fn generate(&self, messages: &[ChatMessage]) -> Result<String>;
    async fn summarize_chunks(&self, chunks: &[RetrievalChunk]) -> Result<String>;
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
}

impl ConversationManager {
    pub fn new(
        embedding_provider: Box<dyn EmbeddingProvider>,
        retrieval_provider: Box<dyn RetrievalProvider>,
        llm_provider: Box<dyn LlmProvider>,
        logger: ActivityLogger,
    ) -> Self {
        Self {
            cache: ConversationCache::new(),
            context_builder: ContextBuilder::default(),
            embedding_provider,
            retrieval_provider,
            llm_provider,
            logger,
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
        &self,
        session_id: SessionId,
        user_id: i64,
        message: String,
        document_id: Option<i64>,
    ) -> Result<String> {
        let start_time = Instant::now();
        info!("Handling message for session {}, user {}", session_id, user_id);

        // 1. Log Initial Payload (RequestReceived)
        self.logger.log(
            ActivityLog::builder(session_id, user_id, ActivityType::RequestReceived)
                .message(&message)
                .document_id(document_id.unwrap_or(0))
                .status(ActivityStatus::Info)
                .build()
        );

        let mut state = self.get_or_create_session(session_id, user_id, document_id).await?;

        // 2. Sliding Window Log
        if state.needs_window_enforcement() {
            self.logger.log(
                ActivityLog::builder(session_id, user_id, ActivityType::SlidingWindowEnforced)
                    .status(ActivityStatus::Warning)
                    .build()
            );
        }

        self.enforce_sliding_window(&mut state)?;

        let current_embedding = self.embedding_provider
            .embed(&message)
            .await
            .map_err(|e| {
                error!("Embedding failed: {:?}", e); // Log full debug info including context
                e
            })
            .context("Failed to embed current message")?;

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

        if token_count_before > 20_000 {
            self.logger.log(
                ActivityLog::builder(session_id, user_id, ActivityType::TokenOverflow)
                    .status(ActivityStatus::Warning)
                    .token_count(token_count_before as i32)
                    .build()
            );
        }

        let llm_messages = self.prepare_llm_payload(&state, &system_context);

        let llm_start = Instant::now();
        let assistant_response = match self.call_llm_with_retry(&llm_messages).await {
            Ok(response) => response,
            Err(e) => {
                // 6. Log LLM Error
                self.logger.log(
                    ActivityLog::builder(session_id, user_id, ActivityType::LlmError)
                        .status(ActivityStatus::Error)
                        .error(e.to_string(), "LlmCallFailed")
                        .build()
                );
                return Err(e);
            }
        };
        let llm_duration = llm_start.elapsed().as_millis() as i32;

        state.messages.push(ChatMessage::assistant(&assistant_response));
        state.last_query_embedding = Some(current_embedding);
        state.metadata.total_messages += 2;
        state.touch();

        self.cache.set(session_id, state);

        let total_duration = start_time.elapsed().as_millis() as i32;

        // 7. Log Final Completion (MessageSent)
        self.logger.log(
            ActivityLog::builder(session_id, user_id, ActivityType::MessageSent)
                .message(&message)
                .response(&assistant_response)
                .token_count(token_count_after as i32)
                .processing_time(total_duration)
                .llm_duration(llm_duration)
                .retrieval_duration(retrieval_duration)
                .document_id(document_id.unwrap_or(0))
                .build()
        );

        Ok(assistant_response)
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
                            return Err(e).context("Retrieval failed");
                        }
                    };

                let summary = self.llm_provider
                    .summarize_chunks(&chunks)
                    .await
                    .context("Failed to summarize chunks")?;

                let system_context = self.context_builder.build_system_context(
                    &summary,
                    document_id.map(|id| format!("Document ID: {}", id)).as_deref(),
                );

                state.system_context = system_context.clone();
                state.last_retrieval_summary = summary;
                state.document_id = document_id;

                Ok(system_context)
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
}

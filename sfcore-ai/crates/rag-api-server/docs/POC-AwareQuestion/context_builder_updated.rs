use anyhow::{Context, Result};
use tracing::{debug, info};
use crate::models::chat::ChatMessage;
use crate::utils::similarity::cosine_similarity;
use super::types::{
    ConversationState, RetrievalDecision, RetrievalReason, 
    SkipReason, SystemContextComponents, WeightedEmbeddingConfig
};

// Import query analyzer
use crate::services::query_analyzer::{QueryAnalyzer, QueryIntent};

pub struct ContextBuilder {
    base_instruction: String,
    similarity_threshold: f32,
    weighted_config: WeightedEmbeddingConfig,
}

impl ContextBuilder {
    pub fn new(base_instruction: String) -> Self {
        Self {
            base_instruction,
            similarity_threshold: 0.75,
            weighted_config: WeightedEmbeddingConfig::default(),
        }
    }

    pub fn default_base_instruction() -> String {
        r#"You are an intelligent AI assistant for a Document Management System.

Your role is to help users understand and work with their documents by:
- Answering questions based on the provided document context
- Providing accurate information from the retrieved documents
- Being concise and helpful in your responses
- Admitting when information is not available in the context

Guidelines:
- Always base your answers on the provided document context
- If the context doesn't contain relevant information, say so clearly
- Cite document sources when possible
- Be conversational but professional
- Keep responses focused and relevant to the user's question"#.to_string()
    }

    pub fn decide_retrieval(
        &self,
        state: &ConversationState,
        current_message: &str,
        current_document_id: Option<i64>,
        current_embedding: Option<&Vec<f32>>,
    ) -> Result<RetrievalDecision> {
        // STEP 1: Analyze query intent FIRST
        let intent = QueryAnalyzer::analyze_intent(current_message);
        
        // STEP 2: Handle meta-questions (overview/summary)
        match intent {
            QueryIntent::DocumentOverview | QueryIntent::DocumentSummary => {
                info!(
                    "Meta-question detected ({:?}), will fetch document metadata instead of vector search",
                    intent
                );
                return Ok(RetrievalDecision::Retrieve {
                    reason: RetrievalReason::DocumentMetadataQuery,
                    context_aware: false, // No need for weighted embedding
                });
            }
            QueryIntent::Clarification => {
                // Use conversation history for context
                if state.messages.len() >= 2 {
                    info!("Clarification question detected, using conversation context");
                    return Ok(RetrievalDecision::Retrieve {
                        reason: RetrievalReason::ClarificationWithContext,
                        context_aware: true, // Use weighted embedding with history
                    });
                }
                // If no history, treat as first message
            }
            QueryIntent::SpecificContent => {
                // Continue with normal vector search logic
                debug!("Specific content question, proceeding with vector search");
            }
        }
        
        // STEP 3: Normal retrieval logic for specific content questions
        if state.messages.is_empty() {
            debug!("First message in session, need retrieval");
            return Ok(RetrievalDecision::Retrieve {
                reason: RetrievalReason::FirstMessage,
                context_aware: false,
            });
        }

        if state.document_id != current_document_id {
            info!(
                "Document ID changed from {:?} to {:?}, need new retrieval",
                state.document_id, current_document_id
            );
            return Ok(RetrievalDecision::Retrieve {
                reason: RetrievalReason::DocumentIdChanged,
                context_aware: true,
            });
        }

        if let (Some(current_emb), Some(last_emb)) = 
            (current_embedding, &state.last_query_embedding) 
        {
            let similarity = cosine_similarity(current_emb, last_emb)
                .context("Failed to calculate similarity")?;

            debug!("Similarity with last query: {:.4}", similarity);

            if similarity > self.similarity_threshold {
                info!(
                    "High similarity ({:.4} > {}), skipping retrieval",
                    similarity, self.similarity_threshold
                );
                return Ok(RetrievalDecision::Skip {
                    reason: SkipReason::SameDocumentAndHighSimilarity(similarity),
                });
            } else {
                info!(
                    "Low similarity ({:.4} <= {}), need new retrieval",
                    similarity, self.similarity_threshold
                );
                return Ok(RetrievalDecision::Retrieve {
                    reason: RetrievalReason::LowSimilarity(similarity),
                    context_aware: true,
                });
            }
        }

        debug!("No previous embedding found, performing retrieval");
        Ok(RetrievalDecision::Retrieve {
            reason: RetrievalReason::FirstMessage,
            context_aware: false,
        })
    }

    pub fn build_system_context(
        &self,
        retrieval_summary: &str,
        document_metadata: Option<&str>,
    ) -> String {
        let components = SystemContextComponents {
            base_instruction: self.base_instruction.clone(),
            retrieval_context: retrieval_summary.to_string(),
            metadata_section: document_metadata.map(|s| s.to_string()),
        };

        components.build()
    }

    pub fn prepare_context_aware_text(
        &self,
        current_message: &str,
        history: &[ChatMessage],
    ) -> String {
        if history.is_empty() {
            return current_message.to_string();
        }

        let last_user_messages: Vec<String> = history
            .iter()
            .filter(|msg| msg.role == "user")
            .rev()
            .take(self.weighted_config.max_history_messages)
            .map(|msg| msg.content.clone())
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();

        if last_user_messages.is_empty() {
            return current_message.to_string();
        }

        let history_text = last_user_messages.join(" ");
        format!("{} {}", history_text, current_message)
    }

    pub fn weighted_config(&self) -> &WeightedEmbeddingConfig {
        &self.weighted_config
    }
}

impl Default for ContextBuilder {
    fn default() -> Self {
        Self::new(Self::default_base_instruction())
    }
}

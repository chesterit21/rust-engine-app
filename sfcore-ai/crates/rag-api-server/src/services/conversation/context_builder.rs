use anyhow::{Context, Result};
use tracing::{debug, info};
use crate::models::chat::ChatMessage;
use crate::utils::similarity::cosine_similarity;
use crate::services::query_analyzer::{QueryAnalyzer, QueryIntent};
use super::types::{
    ConversationState, RetrievalDecision, RetrievalReason, 
    SkipReason, SystemContextComponents, WeightedEmbeddingConfig
};

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
    pub fn base_instruction(&self) -> &str {
        &self.base_instruction
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
        current_document_ids: Option<Vec<i64>>,
        current_embedding: Option<&Vec<f32>>,
    ) -> Result<RetrievalDecision> {
        // 1. Analyze Intent (POC Meta-Question Enhancement)
        let intent = QueryAnalyzer::analyze_intent(current_message);
        
        match intent {
            QueryIntent::DocumentOverview | QueryIntent::DocumentSummary => {
                debug!("Meta-question detected, triggering MetadataQuery retrieval");
                return Ok(RetrievalDecision::Retrieve {
                    reason: RetrievalReason::DocumentMetadataQuery,
                    context_aware: false,
                });
            }
            QueryIntent::Clarification => {
                debug!("Clarification intent detected, triggering ContextAware retrieval");
                return Ok(RetrievalDecision::Retrieve {
                    reason: RetrievalReason::ClarificationWithContext,
                    context_aware: true,
                });
            }
            _ => {} // Continue to normal logic for SpecificContent
        }

        if state.messages.is_empty() {
            debug!("First message in session, need retrieval");
            return Ok(RetrievalDecision::Retrieve {
                reason: RetrievalReason::FirstMessage,
                context_aware: false,
            });
        }

        if state.document_ids != current_document_ids {
            info!(
                "Document Context changed from {:?} to {:?}, need new retrieval",
                state.document_ids, current_document_ids
            );
            return Ok(RetrievalDecision::Retrieve {
                reason: RetrievalReason::DocumentContextChanged,
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

use crate::config::RagConfig;
use crate::database::{DocumentChunk, Repository};
use crate::services::{EmbeddingService, LlmService};
use crate::utils::error::ApiError;
use anyhow::Result;
use pgvector::Vector;
use std::sync::Arc;
use tracing::{debug, info};

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
        Self {
            repository,
            embedding_service,
            llm_service,
            config,
        }
    }
    
    /// Retrieve relevant chunks untuk user query
    pub async fn retrieve(
        &self,
        user_id: i32,
        query: &str,
        document_id: Option<i32>,
    ) -> Result<Vec<DocumentChunk>, ApiError> {
        info!("Retrieving context for user {} query: {}", user_id, query);
        
        // Generate query embedding
        let query_embedding = self.embedding_service.embed(query).await?;
        let vector = Vector::from(query_embedding);
        
        // Search dengan authorization
        let chunks = if self.config.rerank_enabled {
            // Hybrid search (vector + full-text)
            self.repository
                .hybrid_search_user_documents(
                    user_id,
                    vector,
                    query.to_string(),
                    self.config.retrieval_top_k as i32,
                    document_id,
                )
                .await
                .map_err(|e| ApiError::DatabaseError(e.to_string()))?
        } else {
            // Pure vector search
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
        
        debug!("Retrieved {} chunks", chunks.len());
        
        Ok(chunks)
    }
    
    /// Build context dari chunks
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
            
            // Limit total context length
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
    
    /// Build prompt dengan RAG context
    pub fn build_prompt(&self, user_query: &str, context: &str) -> Vec<super::llm_service::ChatMessage> {
        let system_message = super::llm_service::ChatMessage {
            role: "system".to_string(),
            content: format!(
                "Anda adalah asisten AI yang membantu menjawab pertanyaan berdasarkan dokumen yang diberikan. \
                 Jawab pertanyaan dengan akurat berdasarkan konteks yang tersedia. \
                 Jika informasi tidak ada dalam konteks, katakan dengan jelas.\n\n{}",
                context
            ),
        };
        
        let user_message = super::llm_service::ChatMessage {
            role: "user".to_string(),
            content: user_query.to_string(),
        };
        
        vec![system_message, user_message]
    }
}

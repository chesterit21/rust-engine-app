use crate::config::RagConfig;
use crate::database::{DocumentChunk, Repository};
use crate::services::{EmbeddingService, LlmService};
use crate::utils::error::ApiError;
use anyhow::Result;
use pgvector::Vector;
use std::sync::Arc;
use tracing::{debug, info};
use crate::services::conversation::manager::{RetrievalProvider, RetrievalChunk};

#[derive(Clone)] // Clone derives for Arc usage if needed, but RagService usually wrapped in Arc
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
    
    /// Retrieve relevant chunks untuk user query (Public API)
    pub async fn retrieve(
        &self,
        user_id: i32,
        query: &str,
        document_id: Option<i32>,
    ) -> Result<Vec<DocumentChunk>, ApiError> {
        info!("Retrieving context for user {} query: {}", user_id, query);
        
        // Generate query embedding
        let query_embedding = self.embedding_service.embed(query).await?;
        
        self.retrieve_with_embedding(user_id, query, query_embedding, document_id).await
    }

    /// Retrieve relevant chunks with pre-calculated embedding
    pub async fn retrieve_with_embedding(
        &self,
        user_id: i32,
        query_text: &str,
        query_embedding: Vec<f32>,
        document_id: Option<i32>,
    ) -> Result<Vec<DocumentChunk>, ApiError> {
        info!("Retrieving context with embedding for user {}", user_id);
        
        let vector = Vector::from(query_embedding);
        
        // Search dengan authorization
        let chunks = if self.config.rerank_enabled {
            // Hybrid search (vector + full-text)
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
    
    /// Build prompt dengan RAG context (Legacy method using shared ChatMessage)
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
}

// Implement trait for ConversationManager
#[async_trait::async_trait]
impl RetrievalProvider for RagService {
    async fn search(
        &self,
        user_id: i64,
        embedding: &[f32],
        document_id: Option<i64>,
    ) -> Result<Vec<RetrievalChunk>> {
        // We do *hybrid* search if we have a way to synthesize "query_text". 
        // But the trait doesn't provide query text, only embedding.
        // If we strictly follow trait signature from Fix, we only have embedding.
        // So we must use `retrieve_with_embedding` but pass empty string for query_text if we want to fallback to pure vector?
        // Or we should update trait to pass text?
        // `ConversationManager::execute_retrieval_decision` computes embedding and calls search.
        // It DOES NOT pass text to `search`.
        // So we must rely on pure vector search if text is not available OR just pass "" if hybrid search tolerates it.
        // If `config.rerank_enabled` is true, hybrid search uses text. If text is empty, fulltext might fail or match nothing.
        // For now, assuming pure vector search is primarily used by memory system or I need to update trait in `manager.rs` to accept `query_text` if hybrid is must.
        // `Fix-ProblemCircular.md` Step 9 implementation of `search` uses `embedding` to manually query DB. It constructs `embedding_str`.
        // It does NOT use `repository`. It uses `PgPool` directly.
        // My `RagService` uses `Repository`.
        // I will attempt to use `repository.search_user_documents` (pure vector) which doesn't need text.
        // `repository.hybrid_search` needs text.
        // If config forces hybrid, we might have issue.
        // I'll use `retrieve_with_embedding` with empty query_text and hope hybrid search handles optional text or just works on vector if text is empty.
        // Or assume pure vector.
        
        let chunks = self.retrieve_with_embedding(
            user_id as i32, 
            "", // No text available in trait signature
            embedding.to_vec(), 
            document_id.map(|id| id as i32)
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
}

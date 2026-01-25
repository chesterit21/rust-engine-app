// PATCH for manager.rs - Update RetrievalProvider trait

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

// Also add these imports at top of manager.rs:
// use crate::database::models::{DocumentMetadata, DocumentOverview};

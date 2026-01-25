// PATCH for rag_service.rs - Add trait implementations

// Add this import at top:
// use crate::database::models::{DocumentMetadata, DocumentOverview};

// Implement trait for ConversationManager
#[async_trait::async_trait]
impl RetrievalProvider for RagService {
    async fn search(
        &self,
        user_id: i64,
        embedding: &[f32],
        document_id: Option<i64>,
    ) -> Result<Vec<RetrievalChunk>> {
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
    
    // ============ NEW TRAIT METHOD IMPLEMENTATIONS ============
    
    async fn get_document_metadata(&self, document_id: i32) -> Result<DocumentMetadata> {
        self.repository
            .get_document_metadata(document_id)
            .await
            .context("Failed to fetch document metadata")
    }
    
    async fn get_document_overview_chunks(&self, document_id: i32, limit: i32) -> Result<Vec<RetrievalChunk>> {
        let chunks = self.repository
            .get_document_overview_chunks(document_id, limit)
            .await
            .context("Failed to fetch overview chunks")?;
        
        // Convert to RetrievalChunk format
        Ok(chunks.into_iter().map(|d| RetrievalChunk {
            chunk_id: d.chunk_id,
            document_id: d.document_id as i64,
            document_title: Some(d.document_title),
            content: d.content,
            similarity: d.similarity,
        }).collect())
    }
    
    async fn get_document_overview(&self, document_id: i32, chunk_limit: i32) -> Result<DocumentOverview> {
        self.repository
            .get_document_overview(document_id, chunk_limit)
            .await
            .context("Failed to fetch document overview")
    }
}

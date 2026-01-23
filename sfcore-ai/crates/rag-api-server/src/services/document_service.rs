use crate::database::Repository;
use crate::document::{chunker::TextChunker, parser::DocumentParser};
use crate::services::EmbeddingService;
use crate::utils::error::ApiError;
use anyhow::Result;
use pgvector::Vector;
use std::sync::Arc;
use tracing::{debug, info, warn};

pub struct DocumentService {
    repository: Arc<Repository>,
    embedding_service: Arc<EmbeddingService>,
    chunk_size: usize,
    chunk_overlap: usize,
}

impl DocumentService {
    pub fn new(repository: Arc<Repository>, embedding_service: Arc<EmbeddingService>) -> Self {
        Self {
            repository,
            embedding_service,
            chunk_size: 512,
            chunk_overlap: 50,
        }
    }
    
    /// Process uploaded file: decode -> parse -> chunk -> embed -> save
    pub async fn process_upload(
        &self,
        user_id: i32,
        filename: String,
        file_data: Vec<u8>,
    ) -> Result<(i32, usize), ApiError> {
        info!("Processing upload: {} ({} bytes)", filename, file_data.len());
        
        // 1. Detect file type
        let file_type = self.detect_file_type(&filename)?;
        debug!("Detected file type: {}", file_type);
        
        // 2. Parse document
        let content = self.parse_document(&file_data, &file_type).await?;
        debug!("Extracted {} characters", content.len());
        
        if content.trim().is_empty() {
            return Err(ApiError::BadRequest(
                "No text content found in document".to_string(),
            ));
        }
        
        // 3. Chunk text
        let chunks = self.chunk_text(&content)?;
        info!("Created {} chunks", chunks.len());
        
        if chunks.is_empty() {
            return Err(ApiError::BadRequest("Failed to create chunks".to_string()));
        }
        
        // 4. Generate embeddings (batch)
        let texts: Vec<String> = chunks.clone();
        let embeddings = self.embedding_service.embed_batch(texts).await?;
        debug!("Generated {} embeddings", embeddings.len());
        
        // 5. Create document record in TblDocuments
        let document_id = self.create_document_record(user_id, &filename).await?;
        info!("Created document record: id={}", document_id);
        
        // 6. Save chunks to rag_document_chunks
        let chunks_len = chunks.len();
        let chunk_data: Vec<(String, Vector)> = chunks
            .into_iter()
            .zip(embeddings.into_iter())
            .map(|(content, embedding)| (content, Vector::from(embedding)))
            .collect();
        
        self.repository
            .insert_document_chunks(document_id, chunk_data)
            .await
            .map_err(|e| ApiError::DatabaseError(e.to_string()))?;
        
        info!("Document {} processed successfully with {} chunks", document_id, chunks_len);
        
        Ok((document_id, chunks_len))
    }
    
    fn detect_file_type(&self, filename: &str) -> Result<String, ApiError> {
        let extension = std::path::Path::new(filename)
            .extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| ApiError::BadRequest("No file extension found".to_string()))?
            .to_lowercase();
        
        match extension.as_str() {
            "pdf" => Ok("pdf".to_string()),
            "docx" | "doc" => Ok("docx".to_string()),
            "txt" => Ok("text".to_string()),
            "md" => Ok("markdown".to_string()),
            "html" | "htm" => Ok("html".to_string()),
            _ => Err(ApiError::BadRequest(format!(
                "Unsupported file type: {}",
                extension
            ))),
        }
    }
    
    async fn parse_document(&self, data: &[u8], file_type: &str) -> Result<String, ApiError> {
        // Save to temp file
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join(format!("upload_{}.{}", uuid::Uuid::new_v4(), file_type));
        
        tokio::fs::write(&temp_file, data)
            .await
            .map_err(|e| ApiError::InternalError(format!("Failed to write temp file: {}", e)))?;
        
        // Parse
        let parsed = DocumentParser::parse(&temp_file)
            .map_err(|e| ApiError::InternalError(format!("Failed to parse document: {}", e)))?;
        
        // Cleanup
        let _ = tokio::fs::remove_file(&temp_file).await;
        
        Ok(parsed.content)
    }
    
    fn chunk_text(&self, text: &str) -> Result<Vec<String>, ApiError> {
        let chunker = TextChunker::new(self.chunk_size, self.chunk_overlap);
        
        let chunks = chunker
            .chunk(text)
            .map_err(|e| ApiError::InternalError(format!("Failed to chunk text: {}", e)))?;
        
        Ok(chunks.into_iter().map(|c| c.content).collect())
    }
    
    async fn create_document_record(&self, user_id: i32, filename: &str) -> Result<i32, ApiError> {
        // TODO: Real implementation - insert ke TblDocuments
        // Untuk sekarang, generate temporary ID
        let document_id = (chrono::Utc::now().timestamp() % 100000) as i32;
        
        info!(
            "Document record created: id={}, user={}, filename={}",
            document_id, user_id, filename
        );
        
        Ok(document_id)
    }
}

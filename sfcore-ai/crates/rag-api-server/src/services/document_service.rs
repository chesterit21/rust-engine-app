use crate::database::Repository;
use crate::document::{chunker::TextChunker, parser::DocumentParser};
use crate::services::EmbeddingService;
use crate::utils::error::ApiError;
use anyhow::Result;
use pgvector::Vector;
use std::sync::Arc;
use tracing::{debug, info};

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
    pub async fn process_upload<F>(
        &self,
        user_id: i32,
        filename: String,
        file_data: Vec<u8>,
        on_progress: F,
    ) -> Result<(i32, usize), ApiError>
    where
        F: Fn(f32, String, String) + Send + Sync,
    {
        info!("Processing upload: {} ({} bytes)", filename, file_data.len());
        on_progress(0.1, "Detecting file type...".to_string(), "detecting".to_string());
        
        // 1. Detect file type
        let file_type = self.detect_file_type(&filename)?;
        debug!("Detected file type: {}", file_type);
        
        // 1.5 Create document record EARLY so we can track progress persistently
        let document_id = self.create_document_record(user_id, &filename, file_data.len() as i32, &file_type).await?;
        info!("Created document record: id={}", document_id);

        let update_status = |repo: Arc<Repository>, doc_id: i32, progress: f32, msg: String, flag: String| {
            let repo = repo.clone();
            let msg_clone = msg.clone();
            let flag_clone = flag.clone();
            tokio::spawn(async move {
                let _ = repo.upsert_document_processing_status(doc_id, &flag_clone, progress, Some(msg_clone)).await;
            });
        };

        let report_progress = |progress: f32, message: String, status_flag: String| {
            on_progress(progress, message.clone(), status_flag.clone());
            update_status(self.repository.clone(), document_id, progress, message, status_flag);
        };

        report_progress(0.1, "Detecting file type...".to_string(), "detecting".to_string());
        
        // 2. Parse document
        report_progress(0.2, "Parsing document content...".to_string(), "parsing".to_string());
        let content = self.parse_document(&file_data, &file_type).await?;
        debug!("Extracted {} characters", content.len());
        
        if content.trim().is_empty() {
            let _ = self.repository.upsert_document_processing_status(document_id, "failed", 0.0, Some("Empty document".to_string())).await;
            return Err(ApiError::BadRequest(
                "No text content found in document".to_string(),
            ));
        }
        
        // 3. Chunk text
        report_progress(0.4, "Chunking text...".to_string(), "chunking".to_string());
        let chunks = self.chunk_text(&content)?;
        info!("Created {} chunks", chunks.len());
        
        if chunks.is_empty() {
             let _ = self.repository.upsert_document_processing_status(document_id, "failed", 0.0, Some("No chunks created".to_string())).await;
            return Err(ApiError::BadRequest("Failed to create chunks".to_string()));
        }
        
        // 4. Generate embeddings (batch)
        report_progress(0.6, "Generating embeddings (this might take a while)...".to_string(), "embedding-inprogress".to_string());
        let texts: Vec<String> = chunks.clone();
        let embeddings = self.embedding_service.embed_batch(texts).await?;
        debug!("Generated {} embeddings", embeddings.len());
        
        // 5. Build chunk data
        report_progress(0.8, "Preparing chunks for database...".to_string(), "saving".to_string());
        let chunks_len = chunks.len();
        let chunk_data: Vec<(String, Vector)> = chunks
            .into_iter()
            .zip(embeddings.into_iter())
            .map(|(content, embedding)| (content, Vector::from(embedding)))
            .collect();
        
        // 6. Save chunks to rag_document_chunks
        report_progress(0.9, "Indexing chunks...".to_string(), "indexing".to_string());
        self.repository
            .insert_document_chunks(document_id, chunk_data)
            .await
            .map_err(|e| ApiError::DatabaseError(e.to_string()))?;
        
        report_progress(1.0, "Processing completed".to_string(), "completed".to_string());
        
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
            "pptx" | "ppt" => Ok("pptx".to_string()),
            "xlsx" | "xls" => Ok("xlsx".to_string()),
            "rtf" => Ok("rtf".to_string()),
            "txt" => Ok("txt".to_string()),
            "md" | "markdown" => Ok("md".to_string()),
            "html" | "htm" => Ok("html".to_string()),
            "png" | "jpg" | "jpeg" | "tiff" | "bmp" => Ok(extension),
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
    
    async fn create_document_record(
        &self, 
        user_id: i32, 
        filename: &str,
        file_size: i32,
        file_type: &str,
    ) -> Result<i32, ApiError> {
        self.repository
            .create_document(user_id, filename, file_size, file_type)
            .await
            .map_err(|e| ApiError::DatabaseError(e.to_string()))
    }
}

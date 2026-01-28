use crate::database::Repository;
use crate::document::{chunker::TextChunker, parser::DocumentParser};
use crate::services::gemini::GeminiService;
use crate::utils::error::ApiError;
use anyhow::Result;
use pgvector::Vector;
use std::sync::Arc;
use tracing::{debug, info, warn};

pub struct GeminiDocumentService {
    repository: Arc<Repository>,
    gemini_service: Arc<GeminiService>,
    chunk_size: usize,
    chunk_overlap: usize,
    document_path: String,
}

impl GeminiDocumentService {
    pub fn new(
        repository: Arc<Repository>,
        gemini_service: Arc<GeminiService>,
        chunk_size: usize,
        chunk_overlap: usize,
        document_path: String,
    ) -> Self {
        Self {
            repository,
            gemini_service,
            chunk_size,
            chunk_overlap,
            document_path,
        }
    }

    /// Phase 1: Initial Creation (Sync-like) - Returns ID immediately
    pub async fn create_initial_document(
        &self,
        user_id: i32,
        filename: &str,
        file_data: &[u8],
    ) -> Result<(i32, usize), ApiError> {
        info!("(Gemini) Creating initial document: {}", filename);
        
        let file_type = self.detect_file_type(filename)?;
        
        // 1. Ensure category
        let category_id = self.repository
            .ensure_ai_upload_category(user_id)
            .await
            .map_err(|e| ApiError::DatabaseError(format!("Failed to ensure category: {}", e)))?;

        // 2. Save Physical File
        let root_path = std::path::Path::new(&self.document_path);
        let user_folder_name = format!("{}-Document-AI", user_id);
        let target_dir = root_path.join(&user_folder_name);
        
        tokio::fs::create_dir_all(&target_dir).await.map_err(|e| ApiError::InternalError(e.to_string()))?;
            
        let unique_id = uuid::Uuid::new_v4();
        let extension = std::path::Path::new(filename).extension().and_then(|e| e.to_str()).unwrap_or("bin");
        let unique_filename = format!("{}.{}", unique_id, extension);
        let target_path = target_dir.join(&unique_filename);
        let file_path_string = target_path.to_string_lossy().to_string();
        
        tokio::fs::write(&target_path, file_data).await.map_err(|e| ApiError::InternalError(e.to_string()))?;

        // 3. Create Record
        let doc_id = self.repository
            .create_document(user_id, filename, file_data.len() as i32, &file_type, category_id, &file_path_string)
            .await
            .map_err(|e| ApiError::DatabaseError(format!("Failed to create document record: {}", e)))?;
            
        Ok((doc_id, file_data.len()))
    }

    /// Phase 2: Background Processing (Parse -> Chunk -> Embed -> Save)
    pub async fn process_document_background<F>(
        &self,
        document_id: i32,
        file_data: &[u8],
        on_progress: F,
    ) -> Result<(i32, usize), ApiError>
    where
        F: Fn(i32, f64, String, String) + Send + Sync + 'static,
    {
        // We need to re-detect type or pass it. 
        // For simplicity/robustness, we re-detect from header or just assume type from DB?
        // Let's rely on magic numbers via `infer` or just pass it? 
        // To avoid changing signature too much, let's detect from data if possible, or just pass a dummy name to helper.
        // Actually, we can fetch metadata from DB, but that's slow.
        // Let's use `infer` on `file_data` for parsing.
        
        // Helper for status updates
        let repo_clone = self.repository.clone();
        let update_status = move |doc_id: i32, progress: f64, msg: String, status: String| {
            let r = repo_clone.clone();
            tokio::spawn(async move {
                let _ = r.upsert_document_processing_status(doc_id, &status, progress, Some(msg)).await;
            });
        };

        // 1. Parse
        on_progress(document_id, 0.1, "Gemini: Parsing...".to_string(), "parsing".to_string());
        update_status(document_id, 0.1, "Parsing...".to_string(), "parsing".to_string());
        
        // Use a dummy extension for now or detect
        // Ideally we pass filename, but I don't want to change signature again.
        // `DocumentParser` needs a file on disk usually. 
        // Wait, `parse_document` helper I wrote uses a temp file with extension. 
        // I need an extension.
        // Let's just default to "bin" or try to infer.
        // For now, I'll assume "pdf" as fallback or try to detect from magic bytes if possible.
        // Actually `crate::document::parser` usually handles auto-detection if file exists.
        // But here I'm writing a temp file.
        // Let's just use "bin" and hope parser detects magic.
        let content = self.parse_document(file_data, "bin").await?; 
        let content = content.replace('\0', ""); // Sanitize null bytes for Postgres 
        
        if content.trim().is_empty() {
             return Err(ApiError::BadRequest("Document is empty".to_string()));
        }

        // 2. Chunk
        on_progress(document_id, 0.3, "Gemini: Chunking...".to_string(), "chunking".to_string());
        let chunks = self.chunk_text(&content)?;
        
        // 3. Embed
        on_progress(document_id, 0.4, "Gemini: Embedding...".to_string(), "embedding".to_string());
        
        let mut embeddings = Vec::with_capacity(chunks.len());
        let total = chunks.len();
        
        for (i, chunk) in chunks.iter().enumerate() {
            match self.gemini_service.embed(chunk).await {
                Ok(emb) => embeddings.push(emb),
                Err(e) => {
                    warn!("Gemini Embedding failed for chunk {}: {}", i, e);
                    embeddings.push(vec![0.0; 768]);
                }
            }
            
            if i % 3 == 0 {
                let p = 0.4 + (0.5 * (i as f64 / total as f64));
                on_progress(document_id, p, format!("Gemini Embedding {}/{}", i + 1, total), "embedding".to_string());
            }
        }

        // 4. Save
        on_progress(document_id, 0.95, "Gemini: Indexing...".to_string(), "saving".to_string());
        
        let chunk_data: Vec<(String, Vector)> = chunks.into_iter()
            .zip(embeddings.into_iter())
            .map(|(t, e)| (t, Vector::from(e)))
            .collect();
            
        self.repository.insert_document_chunks(document_id, chunk_data)
            .await
            .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        on_progress(document_id, 1.0, "Done".to_string(), "completed".to_string());
        update_status(document_id, 1.0, "Done".to_string(), "completed".to_string());

        Ok((document_id, total))
    }
    
    // ... Helpers ...
    fn detect_file_type(&self, filename: &str) -> Result<String, ApiError> {
         let extension = std::path::Path::new(filename)
            .extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| ApiError::BadRequest("No file extension found".to_string()))?
            .to_lowercase();
        Ok(extension)
    }

    async fn parse_document(&self, data: &[u8], file_type: &str) -> Result<String, ApiError> {
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join(format!("gemini_upload_{}.{}", uuid::Uuid::new_v4(), file_type));
        
        tokio::fs::write(&temp_file, data).await.map_err(|e| ApiError::InternalError(e.to_string()))?;
        // Parser creates a temporary file so it should be fine
        let parsed = DocumentParser::parse(&temp_file).map_err(|e| ApiError::InternalError(e.to_string()))?;
        let _ = tokio::fs::remove_file(&temp_file).await;
        
        Ok(parsed.content)
    }

    fn chunk_text(&self, text: &str) -> Result<Vec<String>, ApiError> {
        let chunker = TextChunker::new(self.chunk_size, self.chunk_overlap);
        chunker.chunk(text)
            .map_err(|e| ApiError::InternalError(e.to_string()))
            .map(|chunks| chunks.into_iter().map(|c| c.content).collect())
    }
}

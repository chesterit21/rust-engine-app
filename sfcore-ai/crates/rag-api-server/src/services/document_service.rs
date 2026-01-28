use crate::database::Repository;
use crate::document::{chunker::TextChunker, parser::DocumentParser};
use crate::services::EmbeddingService;
use crate::services::LlmService;
use crate::models::chat::ChatMessage;
use crate::utils::error::ApiError;
use anyhow::Result;
use pgvector::Vector;
use std::sync::Arc;
use infer;
use tracing::{debug, info, warn};

pub struct DocumentService {
    repository: Arc<Repository>,
    embedding_service: Arc<EmbeddingService>,
    llm_service: Arc<LlmService>,
    chunk_size: usize,
    chunk_overlap: usize,
    document_path: String,
    embedding_batch_size: usize, // NEW
}

impl DocumentService {
    pub fn new(
        repository: Arc<Repository>,
        embedding_service: Arc<EmbeddingService>,
        llm_service: Arc<LlmService>,
        config: &crate::config::RagConfig,
        limits: &crate::config::LimitsConfig, // NEW ARG
    ) -> Self {
        Self {
            repository,
            embedding_service,
            llm_service,
            chunk_size: config.chunk_size,
            chunk_overlap: (config.chunk_size as f32 * config.chunk_overlap_percentage) as usize,
            document_path: config.document_path.clone(),
            embedding_batch_size: limits.embedding_batch_size, // STORE
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
        F: Fn(i32, f64, String, String) + Send + Sync,
    {
        info!("Processing upload: {} ({} bytes)", filename, file_data.len());
        // Phase 1: Pre-detection (ID unknown, use 0)
        on_progress(0, 0.1, "Detecting file type...".to_string(), "detecting".to_string());
        
        // 1. Detect file type
        let file_type = self.detect_file_type(&filename)?;
        debug!("Detected file type: {}", file_type);
        
        // 1.5 Create document record EARLY so we can track progress persistently
        // Phase 5: Pass file_data for saving
        let document_id = self.create_document_record(user_id, &filename, file_data.len() as i32, &file_type, &file_data).await?;
        info!("Created document record: id={}", document_id);

        let update_status = |repo: Arc<Repository>, doc_id: i32, progress: f64, msg: String, flag: String| {
            let repo = repo.clone();
            let msg_clone = msg.clone();
            let flag_clone = flag.clone();
            tokio::spawn(async move {
                let _ = repo.upsert_document_processing_status(doc_id, &flag_clone, progress, Some(msg_clone)).await;
            });
        };

        let report_progress = |progress: f64, message: String, status_flag: String| {
            on_progress(document_id, progress, message.clone(), status_flag.clone());
            update_status(self.repository.clone(), document_id, progress, message, status_flag);
        };

        report_progress(0.1, "Detecting file type...".to_string(), "detecting".to_string());
        
        // 2. Parse document
        report_progress(0.2, "Parsing document content...".to_string(), "parsing".to_string());
        let content = self.parse_document(&file_data, &file_type).await?;
        debug!("Extracted {} characters", content.len());
        
        if content.trim().is_empty() {
            report_progress(0.0, "Empty document content".to_string(), "failed".to_string());
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
             report_progress(0.0, "Document parsing failed".to_string(), "failed".to_string());
             let _ = self.repository.upsert_document_processing_status(document_id, "failed", 0.0, Some("No chunks created".to_string())).await;
            return Err(ApiError::BadRequest("Failed to create chunks".to_string()));
        }
        
        // 4. Generate embeddings (batch)
        report_progress(0.6, "Generating embeddings (this might take a while)...".to_string(), "embedding-inprogress".to_string());
        let texts: Vec<String> = chunks.clone();
        
        // Previous join_all(futures) spawned ALL requests at once, flooding the semaphore queue.
        // If queue time > 15s (acquire_timeout), tasks fail/stagnate.
        // We use configured batch size or default to 5 if not provided (safety)
        let batch_size = self.embedding_batch_size.max(1);
        let mut embeddings = Vec::with_capacity(texts.len());
        
        for (i, batch_texts) in texts.chunks(batch_size).enumerate() {
             let total_batches = (texts.len() + batch_size - 1) / batch_size;
             report_progress(
                 0.6 + (0.2 * (i as f64 / total_batches as f64)), 
                 format!("Embedding batch {}/{}...", i + 1, total_batches), 
                 "embedding-inprogress".to_string()
             );
             
             // Process this batch
             let batch_embeddings = self.embedding_service.embed_batch(batch_texts.to_vec()).await?;
             embeddings.extend(batch_embeddings);
        }

        debug!("Generated {} embeddings", embeddings.len());
        
        // 5. Build chunk data
        report_progress(0.8, "Preparing chunks for database...".to_string(), "saving".to_string());
        let chunks_len = chunks.len();
        let chunk_data: Vec<(String, Vector)> = chunks
            .iter()
            .cloned()
            .zip(embeddings.into_iter())
            .map(|(content, embedding)| (content, Vector::from(embedding)))
            .collect();
        
        // 6. Save chunks to rag_document_chunks
        report_progress(0.85, "Indexing chunks...".to_string(), "indexing".to_string());
        self.repository
            .insert_document_chunks(document_id, chunk_data)
            .await
            .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        // ============ NEW: Generate Auto-Summary ============
        report_progress(0.9, "Generating document summary...".to_string(), "summarizing".to_string());
        if let Err(e) = self.generate_document_summary(document_id, &chunks).await {
            warn!("Failed to generate auto-summary for document {}: {}", document_id, e);
        }
        // ============ END NEW ============
        
        report_progress(1.0, "Processing completed".to_string(), "completed".to_string());
        
        info!("Document {} processed successfully with {} chunks", document_id, chunks_len);
        
        Ok((document_id, chunks_len))
    }

    /// Phase 6: Sync method to creating document record only
    pub async fn create_initial_document(
        &self,
        user_id: i32,
        filename: String,
        file_data: Vec<u8>
    ) -> Result<(i32, String, Vec<u8>), ApiError> {
         // 1. Detect file type
        let file_type = self.detect_file_type(&filename)?;
        debug!("Detected file type: {}", file_type);
        
        // Phase 9: Security Validation
        self.validate_file_content(&file_data, &file_type)?;

        // 2. Create record & Save file
        let document_id = self.create_document_record(user_id, &filename, file_data.len() as i32, &file_type, &file_data).await?;
        info!("Created document record: id={}", document_id);
        
        Ok((document_id, file_type, file_data))
    }

    /// Phase 6: Background processing
    pub async fn process_document_background<F>(
        &self,
        document_id: i32,
        file_type: String,
        file_data: Vec<u8>,
        on_progress: F,
    ) -> Result<(i32, usize), ApiError>
    where
        F: Fn(i32, f64, String, String) + Send + Sync,
    {
        let update_status = |repo: Arc<Repository>, doc_id: i32, progress: f64, msg: String, flag: String| {
            let repo = repo.clone();
            let msg_clone = msg.clone();
            let flag_clone = flag.clone();
            tokio::spawn(async move {
                let _ = repo.upsert_document_processing_status(doc_id, &flag_clone, progress, Some(msg_clone)).await;
            });
        };

        let report_progress = |progress: f64, message: String, status_flag: String| {
            on_progress(document_id, progress, message.clone(), status_flag.clone());
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
        
        // 4. Generate embeddings (batch) or Fallback
        // 4. Generate embeddings (batch) or Fallback
        report_progress(0.6, "Generating embeddings (this might take a while)...".to_string(), "embedding-inprogress".to_string());
        let texts: Vec<String> = chunks.clone();
        
        // Use configured batch size
        let batch_size = self.embedding_batch_size.max(1);
        let mut embeddings = Vec::with_capacity(texts.len());
        let total_batches = (texts.len() + batch_size - 1) / batch_size;

        for (i, batch_texts) in texts.chunks(batch_size).enumerate() {
             // Report progress for this batch
             report_progress(
                 0.6 + (0.2 * (i as f64 / total_batches as f64)), 
                 format!("Embedding batch {}/{}...", i + 1, total_batches), 
                 "embedding-inprogress".to_string()
             );

             // Embed batch
             match self.embedding_service.embed_batch(batch_texts.to_vec()).await {
                Ok(batch_embs) => {
                    embeddings.extend(batch_embs);
                },
                Err(err) => {
                    warn!("Embedding failed for batch {}/{} of document {} (falling back to zerovec): {}", i + 1, total_batches, document_id, err);
                    // Fallback to zero vectors so Deep Scan can still work for this batch
                    let dim = self.embedding_service.dimension;
                    embeddings.extend(vec![vec![0.0; dim]; batch_texts.len()]);
                }
             }
        }
        // debug!("Generated {} embeddings", embeddings.len());
        
        // 5. Build chunk data
        report_progress(0.8, "Preparing chunks for database...".to_string(), "saving".to_string());
        let chunks_len = chunks.len();
        let chunk_data: Vec<(String, Vector)> = chunks
            .iter()
            .cloned()
            .zip(embeddings.into_iter())
            .map(|(content, embedding)| (content, Vector::from(embedding)))
            .collect();
        
        // 6. Save chunks to rag_document_chunks
        report_progress(0.85, "Indexing chunks...".to_string(), "indexing".to_string());
        self.repository
            .insert_document_chunks(document_id, chunk_data)
            .await
            .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        // ============ NEW: Generate Auto-Summary ============
        report_progress(0.9, "Generating document summary...".to_string(), "summarizing".to_string());
        if let Err(e) = self.generate_document_summary(document_id, &chunks).await {
            warn!("Failed to generate auto-summary for document {}: {}", document_id, e);
        }
        // ============ END NEW ============
        
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
        file_data: &[u8],
    ) -> Result<i32, ApiError> {
        // 1. Ensure category exists
        let category_id = self.repository
            .ensure_ai_upload_category(user_id)
            .await
            .map_err(|e| ApiError::DatabaseError(format!("Failed to ensure category: {}", e)))?;

        // 2. Persistent File Storage (Phase 5)
        let root_path = std::path::Path::new(&self.document_path);
        let user_folder_name = format!("{}-Document-AI", user_id);
        let target_dir = root_path.join(&user_folder_name);
        
        // Ensure directory exists
        tokio::fs::create_dir_all(&target_dir)
            .await
            .map_err(|e| ApiError::InternalError(format!("Failed to create document directory: {}", e)))?;
            
        // Phase 9: Secure Unique File Naming (UUID)
        let unique_id = uuid::Uuid::new_v4();
        let extension = std::path::Path::new(filename)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("bin");
            
        let unique_filename = format!("{}.{}", unique_id, extension);
        let target_path = target_dir.join(&unique_filename);
        let file_path_string = target_path.to_string_lossy().to_string();
        
        // Write file
        tokio::fs::write(&target_path, file_data)
            .await
            .map_err(|e| ApiError::InternalError(format!("Failed to save file physically: {}", e)))?;
            
        info!("Saved document physically to: {}", file_path_string);

        // 3. Create DB Record with physical path
        let doc_id = self.repository
            .create_document(user_id, filename, file_size, file_type, category_id, &file_path_string)
            .await
            .map_err(|e| ApiError::DatabaseError(format!("Failed to create document record: {}", e)))?;
            
        Ok(doc_id)
    }
    /// Generate auto-summary from first N chunks
    async fn generate_document_summary(
        &self,
        document_id: i32,
        all_chunks: &[String],
    ) -> Result<(), ApiError> {
        // Use first 10 chunks (or less if document is small)
        let summary_chunks: Vec<&String> = all_chunks
            .iter()
            .take(10)
            .collect();
        
        if summary_chunks.is_empty() {
            return Ok(());
        }
        
        // Build prompt for LLM
        let combined_text = summary_chunks
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("\n\n");
        
        let summary_prompt = format!(
            "Buatlah ringkasan singkat (maksimal 3 kalimat) dari dokumen berikut. \
             Fokus pada topik utama dan poin-poin penting:\n\n{}",
            combined_text
        );
        
        // Generate summary using LLM
        let auto_summary = match self.llm_service
            .generate_chat(vec![ChatMessage::user(&summary_prompt)])
            .await {
                Ok(s) => s,
                Err(e) => return Err(ApiError::InternalError(format!("LLM summary failed: {}", e))),
            };
        
        // Save to database
        self.repository
            .update_document_summary(document_id, auto_summary)
            .await
            .map_err(|e| ApiError::DatabaseError(e.to_string()))?;
        
        Ok(())
    }

    /// Phase 9: Security Validation
    fn validate_file_content(&self, data: &[u8], expected_type: &str) -> Result<(), ApiError> {
        // 1. Check size (Max 50MB)
        const MAX_SIZE: usize = 50 * 1024 * 1024;
        if data.len() > MAX_SIZE {
            return Err(ApiError::BadRequest(format!(
                "File too large. Max size is 50MB. Got {} bytes", 
                data.len()
            )));
        }

        // 2. Magic Number Check
        let kind = infer::get(data).ok_or_else(|| {
            ApiError::BadRequest("Could not determine file type from content (unknown magic numbers)".to_string())
        })?;
        
        let mime = kind.mime_type();
        debug!("Magic number detected MIME: {}", mime);

        // 3. Whitelist Validation
        let is_allowed = match expected_type {
            "pdf" => mime == "application/pdf",
            "docx" => mime == "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            "pptx" => mime == "application/vnd.openxmlformats-officedocument.presentationml.presentation",
            "xlsx" => mime == "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            "txt" | "md" => mime.starts_with("text/"), // infer often returns text/plain for code/md
            "html" => mime == "text/html" || mime == "text/xml",
            "png" => mime == "image/png",
            "jpg" | "jpeg" => mime == "image/jpeg",
            _ => false,
        };

        if !is_allowed {
            // Special handling for some text formats that `infer` might miss or misclassify
            if (expected_type == "txt" || expected_type == "md") && mime == "text/plain" {
                // Allow
            } else {
                 return Err(ApiError::BadRequest(format!(
                    "Security Validation Failed: Declared type '{}' does not match detected content type '{}'", 
                    expected_type, mime
                )));
            }
        }
        
        // 4. Blacklist Executables (Secondary Check)
        if mime == "application/x-executable" || mime == "application/x-msdownload" || mime == "application/x-elf" {
             return Err(ApiError::BadRequest("Security Alert: Executable files are strictly prohibited".to_string()));
        }

        Ok(())
    }
}

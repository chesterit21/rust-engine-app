use crate::config::Settings;
use crate::database::{DocumentChunk, IngestionLog, IngestionStatus, Repository};
use crate::document::{DocumentLoader, DocumentParser, TextChunker};
use crate::embedding::{EmbeddingProvider, EmbeddingRequest, LlamaServerManager};
use crate::utils::error::WorkerError;
use anyhow::Result;
use chrono::Utc;
use pgvector::Vector;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

pub struct DocumentProcessor {
    settings: Settings,
    repository: Arc<Repository>,
    llama_manager: Arc<RwLock<LlamaServerManager>>,
}

impl DocumentProcessor {
    pub fn new(
        settings: Settings,
        repository: Arc<Repository>,
        llama_manager: Arc<RwLock<LlamaServerManager>>,
    ) -> Self {
        Self {
            settings,
            repository,
            llama_manager,
        }
    }
    
    /// Process single document
    pub async fn process_document(&self, document_id: i32) -> Result<()> {
        info!("📄 Processing document {}", document_id);
        
        // Get document info from database
        let doc_file = self.repository.get_document_file(document_id).await?
            .ok_or_else(|| WorkerError::DocumentNotFound(document_id))?;
        
        let file_path = self.resolve_file_path(&doc_file.document_file_path)?;
        
        // Validate file
        DocumentLoader::validate_file(&file_path, 100)?; // max 100MB
        
        // Create or update ingestion log
        self.create_ingestion_log(document_id, &file_path).await?;
        
        // Update status to processing
        self.repository
            .update_ingestion_status(document_id, IngestionStatus::Processing, None)
            .await?;
        
        // Process document
        match self.process_document_internal(document_id, &file_path).await {
            Ok(_) => {
                // Update status to completed
                self.repository
                    .update_ingestion_status(document_id, IngestionStatus::Completed, None)
                    .await?;
                
                info!("✅ Document {} processed successfully", document_id);
                Ok(())
            }
            Err(e) => {
                error!("❌ Failed to process document {}: {}", document_id, e);
                
                // Update status to failed
                self.repository
                    .update_ingestion_status(
                        document_id,
                        IngestionStatus::Failed,
                        Some(e.to_string()),
                    )
                    .await?;
                
                Err(e)
            }
        }
    }
    
    /// Internal processing logic
    async fn process_document_internal(
        &self,
        document_id: i32,
        file_path: &PathBuf,
    ) -> Result<()> {
        // 1. Parse document
        info!("📖 Parsing document...");
        let parsed = DocumentParser::parse(file_path)?;
        
        if parsed.content.trim().is_empty() {
            warn!("Document {} has no extractable text", document_id);
            return Ok(());
        }
        
        // 2. Chunk text
        info!("✂️  Chunking text...");
        let chunker = TextChunker::new(
            self.settings.chunking.size,
            self.settings.chunking.overlap,
            self.settings.chunking.strategy.clone(),
        );
        
        let chunks = chunker.chunk(&parsed.content)?;
        
        if chunks.is_empty() {
            warn!("Document {} produced no chunks", document_id);
            return Ok(());
        }
        
        info!("Created {} chunks for document {}", chunks.len(), document_id);
        
        // 3. Generate embeddings (start llama-server on-demand)
        info!("🧠 Generating embeddings...");
        
        // Start llama-server
        {
            let mut llama = self.llama_manager.write().await;
            llama.start().await?;
        }
        
        // Generate embeddings
        let embeddings = {
            let llama = self.llama_manager.read().await;
            let texts: Vec<String> = chunks.iter().map(|c| c.content.clone()).collect();
            
            let request = EmbeddingRequest { texts };
            let response = llama.embed(request).await?;
            
            response.embeddings
        };
        
        // Stop llama-server to free memory
        {
            let mut llama = self.llama_manager.write().await;
            llama.stop().await?;
        }
        
        info!("✅ Generated {} embeddings", embeddings.len());
        
        // 4. Delete existing chunks (for re-indexing)
        let deleted = self.repository.delete_chunks_by_document(document_id).await?;
        if deleted > 0 {
            info!("Deleted {} existing chunks", deleted);
        }
        
        // 5. Save chunks to database
        info!("💾 Saving chunks to database...");
        
        let db_chunks: Vec<DocumentChunk> = chunks
            .into_iter()
            .zip(embeddings.into_iter())
            .map(|(chunk, embedding)| DocumentChunk {
                document_id,
                tenant_id: None, // TODO: extract from document if needed
                chunk_index: chunk.index as i32,
                content: chunk.content,
                char_count: chunk.char_count as i32,
                token_count: chunk.token_count.map(|t| t as i32),
                embedding: Vector::from(embedding),
                page_number: None, // TODO: extract page info if available
                section: None,
                tags: None,
            })
            .collect();
        
        self.repository.insert_chunks(db_chunks).await?;
        
        info!("✅ Saved chunks to database");
        
        Ok(())
    }
    
    /// Resolve file path (handle relative paths)
    fn resolve_file_path(&self, path_str: &str) -> Result<PathBuf> {
        let path = PathBuf::from(path_str);
        
        if path.is_absolute() {
            Ok(path)
        } else {
            // Relative to document root
            let full_path = self.settings.worker.document_root_path.join(path);
            Ok(full_path)
        }
    }
    
    /// Create ingestion log entry
    async fn create_ingestion_log(
        &self,
        document_id: i32,
        file_path: &PathBuf,
    ) -> Result<()> {
        let file_type = DocumentLoader::detect_file_type(file_path).ok();
        let file_size = std::fs::metadata(file_path).ok().map(|m| m.len() as i64);
        
        let log = IngestionLog {
            document_id,
            file_path: file_path.to_string_lossy().to_string(),
            file_size,
            file_type,
            embedding_model: self.settings.embedding.model.clone(),
            chunk_size: self.settings.chunking.size as i32,
            chunk_overlap: self.settings.chunking.overlap as i32,
            status: IngestionStatus::Pending.to_string(),
            total_chunks: 0,
            processed_chunks: 0,
            last_error: None,
            retry_count: 0,
            started_at: Some(Utc::now()),
            processed_at: None,
        };
        
        self.repository.upsert_ingestion_log(&log).await?;
        
        Ok(())
    }
}

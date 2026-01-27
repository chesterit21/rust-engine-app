use crate::config::RagConfig;
use crate::database::{DocumentChunk, Repository};
use crate::services::{EmbeddingService, LlmService};
use crate::utils::error::ApiError;
use anyhow::Result;
use pgvector::Vector;
use std::sync::Arc;
use tracing::{debug, info, warn};
use crate::database::models::{DocumentMetadata, DocumentOverview};
use crate::services::conversation::manager::{RetrievalProvider, RetrievalChunk};
use std::collections::HashMap;
use crate::utils::token_estimator;

/// Grouped chunks by document with metadata
#[derive(Debug, Clone)]
pub struct GroupedDocument {
    pub doc_id: i32,
    pub doc_title: String,
    pub chunks: Vec<DocumentChunk>,
    pub avg_similarity: f32,
    pub total_tokens: usize,
}

/// Context building metrics
#[derive(Debug, Default, Clone)]
pub struct ContextMetrics {
    pub total_tokens: usize,
    pub documents_included: usize,
    pub chunks_included: usize,
    pub truncated: bool,
}

use crate::utils::limiters::Limiters;
use std::time::Instant;

#[derive(Clone)]
pub struct RagService {
    pub repository: Arc<Repository>,
    pub embedding_service: Arc<EmbeddingService>,
    pub llm_service: Arc<LlmService>,
    pub config: RagConfig,
    pub limiters: Arc<Limiters>, // NEW
}

impl RagService {
    pub fn new(
        repository: Arc<Repository>,
        embedding_service: Arc<EmbeddingService>,
        llm_service: Arc<LlmService>,
        config: RagConfig,
        limiters: Arc<Limiters>, // NEW
    ) -> Self {
        Self {
            repository,
            embedding_service,
            llm_service,
            config,
            limiters,
        }
    }
    
    /// Retrieve relevant chunks untuk user query (Public API)
    pub async fn retrieve(
        &self,
        user_id: i32,
        query: &str,
        document_ids: Option<Vec<i32>>,
    ) -> Result<Vec<DocumentChunk>, ApiError> {
        info!("Retrieving context for user {} query: {}", user_id, query);
        
        // Generate query embedding with timeout
        let embedding_future = self.embedding_service.embed(query);
        
        let query_embedding = match tokio::time::timeout(
            std::time::Duration::from_secs(10),
            embedding_future
        ).await {
            Ok(Ok(emb)) => emb,
            Ok(Err(e)) => {
                warn!("Embedding generation failed: {}", e);
                return Err(ApiError::EmbeddingError(e.to_string()));
            }
            Err(_) => {
                warn!("Embedding generation timeout after 10s");
                return Err(ApiError::EmbeddingError("Timeout".to_string()));
            }
        };
        
        self.retrieve_with_embedding(user_id, query, query_embedding, document_ids).await
    }

    /// Retrieve relevant chunks with pre-calculated embedding
    pub async fn retrieve_with_embedding(
        &self,
        user_id: i32,
        query_text: &str,
        query_embedding: Vec<f32>,
        document_ids: Option<Vec<i32>>,
    ) -> Result<Vec<DocumentChunk>, ApiError> {
        info!("Retrieving context with embedding for user {}", user_id);
        
        let vector = Vector::from(query_embedding);

        // Acquire DB-search limiter (covers hybrid/vector search)
        let (_permit, wait) = Limiters::acquire_timed(
            self.limiters.db_search.clone(),
            self.limiters.acquire_timeout,
            "db_search",
        )
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        debug!(wait_ms = wait.as_millis() as u64, op = "db_search", "wait_queue");

        let exec_start = Instant::now();
        
        // Search with timeout protection
        let mut chunks = if self.config.rerank_enabled {
            // Hybrid search
            let search_future = self.repository.hybrid_search_user_documents(
                user_id,
                vector.clone(),
                query_text.to_string(),
                self.config.retrieval_top_k as i32,
                document_ids.clone(),
            );
            
            match tokio::time::timeout(std::time::Duration::from_secs(15), search_future).await {
                Ok(Ok(c)) => c,
                Ok(Err(e)) => {
                    warn!("Hybrid search failed: {}", e);
                    return Err(ApiError::DatabaseError(e.to_string()));
                }
                Err(_) => {
                    warn!("Hybrid search timeout after 15s");
                    return Err(ApiError::DatabaseError("Search timeout".to_string()));
                }
            }
        } else {
            // Pure vector search
            let search_future = self.repository.search_user_documents(
                user_id,
                vector.clone(),
                self.config.retrieval_top_k as i32,
                document_ids.clone(),
            );
            
            match tokio::time::timeout(std::time::Duration::from_secs(15), search_future).await {
                Ok(Ok(c)) => c,
                Ok(Err(e)) => {
                    warn!("Vector search failed: {}", e);
                    return Err(ApiError::DatabaseError(e.to_string()));
                }
                Err(_) => {
                    warn!("Vector search timeout after 15s");
                    return Err(ApiError::DatabaseError("Search timeout".to_string()));
                }
            }
        };

        debug!(exec_ms = exec_start.elapsed().as_millis() as u64, op = "db_search", "exec");
        
        // STRATEGY: "Introduction Context"
        // If specific document is targeted (singular), inject first chunk for overview
        if let Some(ids) = &document_ids {
            if ids.len() == 1 {
                let doc_id = ids[0];
                let has_intro = chunks.iter().any(|c| c.chunk_index == 0 && c.document_id == doc_id);
                
                if !has_intro {
                    let (_permit, wait) = Limiters::acquire_timed(
                        self.limiters.db_search.clone(),
                        self.limiters.acquire_timeout,
                        "db_search_intro_chunk",
                    )
                    .await
                    .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

                    debug!(wait_ms = wait.as_millis() as u64, op = "db_search_intro_chunk", "wait_queue");

                    let exec_start = Instant::now();
                    match self.repository.get_first_chunk(doc_id).await {
                        Ok(Some(intro_chunk)) => {
                            debug!("Injecting intro chunk (index 0) for doc {}", doc_id);
                            chunks.insert(0, intro_chunk);
                        }
                        Ok(None) => debug!("No intro chunk found for doc {}", doc_id),
                        Err(e) => warn!("Failed to fetch intro chunk: {}", e),
                    }
                    debug!(exec_ms = exec_start.elapsed().as_millis() as u64, op = "db_search_intro_chunk", "exec");
                }
            }
        }
        
        debug!("Retrieved {} chunks for user {}", chunks.len(), user_id);
        
        Ok(chunks)
    }
    
    /// Build STRUCTURED context dengan XML tags untuk multi-document clarity
    pub fn build_structured_context(
        &self,
        chunks: Vec<DocumentChunk>,
    ) -> (String, ContextMetrics) {
        if chunks.is_empty() {
            return (
                "Tidak ada konteks yang relevan ditemukan.".to_string(),
                ContextMetrics::default(),
            );
        }
        
        // Group chunks by document
        let grouped = self.group_chunks_by_document(chunks);
        
        // Sort documents by relevance (highest similarity first)
        let mut sorted_docs: Vec<GroupedDocument> = grouped.into_values().collect();
        sorted_docs.sort_by(|a, b| {
            b.avg_similarity.partial_cmp(&a.avg_similarity).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        // Build context with token-aware truncation
        self.format_grouped_context(sorted_docs)
    }
    
    /// Group chunks by document ID with similarity aggregation
    fn group_chunks_by_document(
        &self,
        chunks: Vec<DocumentChunk>,
    ) -> HashMap<i32, GroupedDocument> {
        let mut grouped: HashMap<i32, GroupedDocument> = HashMap::new();
        
        for chunk in chunks {
            let entry = grouped.entry(chunk.document_id)
                .or_insert_with(|| GroupedDocument {
                    doc_id: chunk.document_id,
                    doc_title: chunk.document_title.clone(),
                    chunks: Vec::new(),
                    avg_similarity: 0.0,
                    total_tokens: 0,
                });
            
            // Estimate tokens for this chunk
            entry.total_tokens += token_estimator::estimate_tokens(&chunk.content);
            entry.chunks.push(chunk);
        }
        
        // Calculate average similarity per document
        for doc in grouped.values_mut() {
            if doc.chunks.is_empty() {
                doc.avg_similarity = 0.0;
            } else {
                let sum: f32 = doc.chunks.iter()
                    .map(|c| c.similarity)
                    .sum();
                doc.avg_similarity = sum / doc.chunks.len() as f32;
            }
        }
        
        grouped
    }
    
    /// Format grouped documents dengan XML structure and token management
    fn format_grouped_context(
        &self,
        sorted_docs: Vec<GroupedDocument>,
    ) -> (String, ContextMetrics) {
        use std::fmt::Write;
        
        let max_tokens = self.config.max_context_tokens;
        
        // Pre-allocate buffer: Heuristic 4 chars per token, capped at reasonable size (e.g., 512KB)
        let estimated_chars = (max_tokens * 4).min(512 * 1024);
        let mut context = String::with_capacity(estimated_chars);
        
        context.push_str("DOKUMEN YANG TERSEDIA:\n\n");
        let mut metrics = ContextMetrics::default();
        let mut current_tokens = token_estimator::estimate_tokens(&context);
        
        for doc in sorted_docs {
            // Document header with metadata
            // Use write! to avoid intermediate String allocation
            let header_start = context.len();
            let _ = write!(
                context,
                "<document id=\"doc_{}\" title=\"{}\" relevance=\"{:.3}\">\n",
                doc.doc_id,
                doc.doc_title,
                doc.avg_similarity
            );
            let _header_len = context.len() - header_start;
            
            // Estimate tokens just for the added part
            // Note: optimization - we could estimate based on chars, but stick to tokenizer for correctness first
            let header_slice = &context[header_start..];
            let header_tokens = token_estimator::estimate_tokens(header_slice);
            
            // Check limits
            if current_tokens + header_tokens > max_tokens {
                // Rollback
                context.truncate(header_start);
                metrics.truncated = true;
                debug!(
                    "Context truncated at doc header: {} > {}",
                    current_tokens + header_tokens,
                    max_tokens
                );
                break;
            }
            
            current_tokens += header_tokens;
            metrics.documents_included += 1;
            
            // Add chunks for this document
            for chunk in &doc.chunks {
                let chunk_start = context.len();
                let _ = write!(
                    context,
                    "<chunk id=\"chunk_{}\" page=\"{}\" similarity=\"{:.3}\">\n{}\n</chunk>\n\n",
                    chunk.chunk_id,
                    chunk.page_number.unwrap_or(0),
                    chunk.similarity,
                    chunk.content.trim()
                );
                
                let chunk_slice = &context[chunk_start..];
                let chunk_tokens = token_estimator::estimate_tokens(chunk_slice);
                
                if current_tokens + chunk_tokens > max_tokens {
                    // Rollback
                    context.truncate(chunk_start);
                    metrics.truncated = true;
                    debug!(
                        "Context truncated at chunk: {} > {}",
                        current_tokens + chunk_tokens,
                        max_tokens
                    );
                    break;
                }
                
                current_tokens += chunk_tokens;
                metrics.chunks_included += 1;
            }
            
            // Closing tag
            if metrics.truncated {
                break;
            }
            
            context.push_str("</document>\n\n");
            current_tokens += 2; // Approx for closing tag
        }
        
        metrics.total_tokens = current_tokens;
        
        info!(
            "Built structured context: {} tokens, {} docs, {} chunks{}",
            metrics.total_tokens,
            metrics.documents_included,
            metrics.chunks_included,
            if metrics.truncated { " (TRUNCATED)" } else { "" }
        );
        
        (context, metrics)
    }
    
    /// Build context dari chunks (Legacy method for backward compatibility)
    pub fn build_context(&self, chunks: Vec<DocumentChunk>) -> String {
        let (context, _metrics) = self.build_structured_context(chunks);
        context
    }
    
    /// Build prompt dengan RAG context (Legacy method)
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
        query_text: &str,
        document_id: Option<i64>,
        document_ids: Option<Vec<i64>>,
    ) -> Result<Vec<RetrievalChunk>> {
        // Consolidate document_id and document_ids for retrieval
        let mut final_doc_ids = Vec::new();
        
        if let Some(ids) = document_ids {
            final_doc_ids.extend(ids.into_iter().map(|id| id as i32));
        }
        
        if let Some(single_id) = document_id {
            let single_id_i32 = single_id as i32;
            if !final_doc_ids.contains(&single_id_i32) {
                final_doc_ids.push(single_id_i32);
            }
        }
        
        let doc_ids_option = if final_doc_ids.is_empty() {
            None
        } else {
            Some(final_doc_ids)
        };

        // Use retrieve_with_embedding with ACTUAL query text (enables hybrid search)
        let chunks = self.retrieve_with_embedding(
            user_id as i32, 
            query_text, 
            embedding.to_vec(), 
            doc_ids_option
        ).await;

        match chunks {
            Ok(docs) => Ok(docs.into_iter().map(|d| RetrievalChunk {
                chunk_id: d.chunk_id,
                document_id: d.document_id as i64,
                document_title: Some(d.document_title),
                content: d.content,
                similarity: d.similarity,
            }).collect()),
            Err(e) => {
                warn!("Retrieval failed in RetrievalProvider::search: {}", e);
                anyhow::bail!("Retrieval failed: {}", e)
            }
        }
    }

    async fn get_document_metadata(&self, document_id: i32) -> Result<DocumentMetadata> {
        self.repository.get_document_metadata(document_id).await
    }

    async fn get_document_overview_chunks(&self, document_id: i32, limit: i32) -> Result<Vec<RetrievalChunk>> {
        let chunks = self.repository.get_document_overview_chunks(document_id, limit).await?;
        Ok(chunks.into_iter().map(|c| RetrievalChunk {
            chunk_id: c.chunk_id,
            document_id: c.document_id as i64,
            content: c.content,
            document_title: Some(c.document_title),
            similarity: c.similarity,
        }).collect())
    }

    async fn get_document_overview(&self, document_id: i32, chunk_limit: i32) -> Result<DocumentOverview> {
        self.repository.get_document_overview(document_id, chunk_limit).await
    }

    async fn persist_chat_event(
        &self, 
        user_id: i64, 
        session_id: i64, 
        role: &str, 
        message: &str, 
        doc_ids: Option<Vec<i64>>
    ) -> Result<()> {
        // 1. Ensure Session Header
        let history_id = self.repository.create_chat_session(user_id, session_id).await?;
        
        // 2. Save Message
        self.repository.save_chat_message(history_id, role, message).await?;
        
        // 3. Update Docs Used (if any)
        if let Some(ids) = doc_ids {
            self.repository.save_chat_docs(history_id, &ids).await?;
        }
        
        Ok(())
    }

    async fn persist_session_documents(
        &self,
        user_id: i64,
        session_id: i64,
        doc_ids: Vec<i64>
    ) -> Result<()> {
        if doc_ids.is_empty() {
            return Ok(());
        }
        
        // 1. Ensure Session Header
        let history_id = self.repository.create_chat_session(user_id, session_id).await?;
        
        // 2. Update Docs Used
        self.repository.save_chat_docs(history_id, &doc_ids).await?;
        
        Ok(())
    }

    async fn get_session_active_docs(&self, session_id: i64) -> Result<Vec<i64>> {
        self.repository.get_session_active_docs(session_id).await
    }

    async fn fetch_all_chunks(&self, doc_ids: &[i64]) -> Result<Vec<RetrievalChunk>> {
        let chunks = self.repository.get_chunks_by_document_ids(doc_ids).await?;
        
        Ok(chunks.into_iter().map(|c| RetrievalChunk {
            chunk_id: c.chunk_id,
            document_id: c.document_id as i64,
            document_title: Some(c.document_title),
            content: c.content,
            similarity: 1.0, 
        }).collect())
    }

    async fn fetch_chunks_from_file_fallback(&self, doc_id: i64) -> Result<Vec<RetrievalChunk>> {
        info!("Executing Direct Read Fallback for doc_id: {}", doc_id);
        
        // 1. Get File Path from DB
        #[derive(sqlx::FromRow)]
        struct DocPath {
            file_path: String,
            title: String,
        }
         
        let doc_info = sqlx::query_as::<_, DocPath>(
            r#"SELECT "DocumentFilePath" as file_path, "DocumentFileName" as title FROM "TblDocumentFiles" WHERE "DocumentID" = $1"#
        )
        .bind(doc_id as i32)
        .fetch_optional(self.repository.pool.get_pool())
        .await?;
         
        let (path_str, title) = match doc_info {
            Some(d) => (d.file_path, d.title),
            None => anyhow::bail!("Document {} not found in DB", doc_id),
        };
        
        // 2. Read File Content (Robustly)
        let path_buf = std::path::PathBuf::from(&path_str);
        if !path_buf.exists() {
             anyhow::bail!("File not found on disk: {}", path_str);
        }

        // Strategy: Loop/Retry mechanism as requested
        // Attempt 1: Proper Parsing (DocumentParser) - supports PDF, DOCX, etc.
        // Attempt 2: Lossy Text Read (Fallback for binary-ish text)
        
        let content_result = tokio::task::spawn_blocking(move || {
            // Attempt 1: Use DocumentParser
            match crate::document::parser::DocumentParser::parse(&path_buf) {
                Ok(parsed) => Ok(parsed.content),
                Err(e) => {
                    warn!("Attempt 1 (Parser) failed for {}: {}. Retrying with Lossy Read...", path_buf.display(), e);
                    
                    // Attempt 2: Lossy Read (Force read as text)
                    match std::fs::read(&path_buf) {
                        Ok(bytes) => {
                            let lossy = String::from_utf8_lossy(&bytes).to_string();
                            // If > 70% "unknown" chars, maybe abort? But let's return what we have
                            Ok(lossy)
                        }
                        Err(e2) => Err(anyhow::anyhow!("All attempts failed. Parser: {}. Read: {}", e, e2)),
                    }
                }
            }
        }).await?;

        let content = content_result?;
            
        // 3. Chunking (Simple)
        let chunk_size = 1500;
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        
        for line in content.lines() {
             if current_chunk.len() + line.len() > chunk_size {
                 chunks.push(current_chunk.trim().to_string());
                 current_chunk = String::new();
             }
             current_chunk.push_str(line);
             current_chunk.push('\n');
        }
        if !current_chunk.is_empty() {
            chunks.push(current_chunk.trim().to_string());
        }
        
        // 4. Map to RetrievalChunk
        Ok(chunks.into_iter().enumerate().map(|(i, text)| RetrievalChunk {
            chunk_id: -(i as i64), // Negative ID to indicate temporary/fallback
            document_id: doc_id,
            document_title: Some(title.clone()),
            content: text,
            similarity: 1.0,
        }).collect())
    }
}
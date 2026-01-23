use crate::config::Settings;
use crate::database::{IngestionStatus, Repository};
use crate::embedding::LlamaServerManager;
use crate::worker::DocumentProcessor;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

pub struct BulkIndexer {
    settings: Settings,
    repository: Arc<Repository>,
    llama_manager: Arc<RwLock<LlamaServerManager>>,
}

impl BulkIndexer {
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
    
    /// Run bulk indexing for all unprocessed documents
    pub async fn run(&self) -> Result<usize> {
        info!("🚀 Starting bulk indexing...");
        
        // Get all documents
        let all_docs = self.repository.get_all_document_files().await?;
        info!("Found {} total documents", all_docs.len());
        
        // Filter unprocessed documents
        let mut unprocessed = Vec::new();
        
        for doc in all_docs {
            // Check if already processed
            match self.repository.get_ingestion_log(doc.document_id).await? {
                Some(log) => {
                    // Re-index if failed
                    if log.status == IngestionStatus::Failed.to_string() {
                        info!("Document {} previously failed, will retry", doc.document_id);
                        unprocessed.push(doc);
                    } else if log.status != IngestionStatus::Completed.to_string() {
                        unprocessed.push(doc);
                    }
                }
                None => {
                    // Never processed
                    unprocessed.push(doc);
                }
            }
        }
        
        if unprocessed.is_empty() {
            info!("✅ All documents already processed");
            return Ok(0);
        }
        
        info!("📦 Found {} unprocessed documents", unprocessed.len());
        
        // Create processor
        let processor = DocumentProcessor::new(
            self.settings.clone(),
            self.repository.clone(),
            self.llama_manager.clone(),
        );
        
        // Process in batches
        let batch_size = self.settings.worker.bulk_batch_size;
        let total = unprocessed.len();
        let mut processed_count = 0;
        let mut success_count = 0;
        
        for (batch_idx, batch) in unprocessed.chunks(batch_size).enumerate() {
            info!(
                "Processing batch {}/{} ({} documents)",
                batch_idx + 1,
                (total + batch_size - 1) / batch_size,
                batch.len()
            );
            
            // Start llama-server once per batch
            {
                let mut llama = self.llama_manager.write().await;
                match llama.start().await {
                    Ok(_) => info!("✅ Llama-server started for batch"),
                    Err(e) => {
                        error!("Failed to start llama-server: {}", e);
                        continue;
                    }
                }
            }
            
            // Process documents in batch sequentially
            for doc in batch {
                processed_count += 1;
                
                info!(
                    "Processing document {}/{}: id={}",
                    processed_count, total, doc.document_id
                );
                
                match processor.process_document(doc.document_id).await {
                    Ok(_) => {
                        success_count += 1;
                        info!("✅ Successfully processed document {}", doc.document_id);
                    }
                    Err(e) => {
                        error!("❌ Failed to process document {}: {}", doc.document_id, e);
                    }
                }
            }
            
            // Stop llama-server to free memory
            {
                let mut llama = self.llama_manager.write().await;
                if let Err(e) = llama.stop().await {
                    warn!("Failed to stop llama-server: {}", e);
                }
            }
            
            info!("Batch {} completed, waiting before next batch...", batch_idx + 1);
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        }
        
        info!(
            "🎉 Bulk indexing completed: {}/{} documents successful",
            success_count, total
        );
        
        Ok(success_count)
    }
}

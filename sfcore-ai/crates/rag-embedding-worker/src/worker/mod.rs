pub mod processor;
pub mod bulk_indexer;
pub mod queue;

pub use processor::DocumentProcessor;
pub use bulk_indexer::BulkIndexer;
pub use queue::{TaskQueue, Task, TaskPriority};

use crate::config::Settings;
use crate::database::{DbPool, NotificationListener, Repository};
use crate::embedding::LlamaServerManager;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

pub struct Worker {
    settings: Settings,
    repository: Arc<Repository>,
    listener: NotificationListener,
    task_queue: Arc<TaskQueue>,
    llama_manager: Arc<RwLock<LlamaServerManager>>,
    processor: Arc<DocumentProcessor>,
}

impl Worker {
    pub async fn new(settings: Settings, db_pool: DbPool) -> Result<Self> {
        let repository = Arc::new(Repository::new(db_pool.clone()));
        
        let listener = NotificationListener::new(
            settings.database.clone(),
            settings.database.listen_channel.clone(),
        );
        
        let task_queue = Arc::new(TaskQueue::new(settings.worker.bulk_batch_size));
        
        let llama_manager = Arc::new(RwLock::new(LlamaServerManager::new(
            settings.llama_server.clone(),
        )));
        
        let processor = Arc::new(DocumentProcessor::new(
            settings.clone(),
            repository.clone(),
            llama_manager.clone(),
        ));
        
        Ok(Self {
            settings,
            repository,
            listener,
            task_queue,
            llama_manager,
            processor,
        })
    }
    
    /// Main worker loop
    pub async fn run(self) -> Result<()> {
        info!("🎯 Worker started");
        
        // Start notification listener
        let mut notification_rx = self.listener.start().await?;
        
        // Spawn task processor
        let processor_handle = {
            let task_queue = self.task_queue.clone();
            let processor = self.processor.clone();
            
            tokio::spawn(async move {
                loop {
                    // Get next task from queue
                    if let Some(task) = task_queue.dequeue().await {
                        info!("Processing task: document_id={}", task.document_id);
                        
                        match processor.process_document(task.document_id).await {
                            Ok(_) => {
                                info!("✅ Successfully processed document {}", task.document_id);
                            }
                            Err(e) => {
                                error!("❌ Failed to process document {}: {}", task.document_id, e);
                            }
                        }
                    } else {
                        // No tasks, sleep a bit
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    }
                }
            })
        };
        
        // Handle initial bulk indexing if needed
        info!("🔍 Checking for unprocessed documents...");
        match self.check_and_run_bulk_indexing().await {
            Ok(count) => {
                if count > 0 {
                    info!("📦 Bulk indexing completed: {} documents", count);
                }
            }
            Err(e) => {
                error!("Failed to run bulk indexing: {}", e);
            }
        }
        
        // Listen for notifications
        info!("👂 Listening for document changes...");
        loop {
            tokio::select! {
                // Handle notifications
                Some(notification) = notification_rx.recv() => {
                    info!(
                        "📬 Received notification: op={}, doc_id={}",
                        notification.operation,
                        notification.document_id
                    );
                    
                    // Enqueue task
                    self.task_queue.enqueue(Task {
                        document_id: notification.document_id,
                        priority: TaskPriority::Normal,
                        retry_count: 0,
                    }).await;
                }
                
                // Graceful shutdown signal
                _ = tokio::signal::ctrl_c() => {
                    info!("Received shutdown signal");
                    break;
                }
            }
        }
        
        // Cleanup
        info!("Shutting down worker...");
        processor_handle.abort();
        
        // Stop llama-server if running
        let mut llama = self.llama_manager.write().await;
        llama.stop().await?;
        
        info!("Worker stopped");
        Ok(())
    }
    
    /// Check for unprocessed documents and run bulk indexing if needed
    async fn check_and_run_bulk_indexing(&self) -> Result<usize> {
        let bulk_indexer = BulkIndexer::new(
            self.settings.clone(),
            self.repository.clone(),
            self.llama_manager.clone(),
        );
        
        bulk_indexer.run().await
    }
}

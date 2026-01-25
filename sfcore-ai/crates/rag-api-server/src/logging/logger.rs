use anyhow::Result;
use flume::{Sender, Receiver, bounded};
use sqlx::PgPool;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

use super::types::ActivityLog;

/// Logger configuration
#[derive(Debug, Clone)]
pub struct LoggerConfig {
    /// Queue capacity (max logs in memory before backpressure)
    pub queue_capacity: usize,
    
    /// Batch size for database inserts
    pub batch_size: usize,
    
    /// Max wait time before flushing batch (milliseconds)
    pub batch_timeout_ms: u64,
    
    /// Number of worker threads for database inserts
    pub worker_count: usize,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            queue_capacity: 10_000,     // 10K logs in memory
            batch_size: 100,             // Insert 100 logs per batch
            batch_timeout_ms: 1000,      // Flush every 1 second
            worker_count: 2,             // 2 parallel workers
        }
    }
}

/// Async activity logger with queue mechanism
#[derive(Clone)]
pub struct ActivityLogger {
    sender: Sender<ActivityLog>,
}

impl ActivityLogger {
    /// Initialize logger with background workers
    pub fn new(pool: PgPool, config: LoggerConfig) -> Self {
        let (sender, receiver) = bounded(config.queue_capacity);
        
        info!(
            "Initializing ActivityLogger: queue={}, batch={}, timeout={}ms, workers={}",
            config.queue_capacity,
            config.batch_size,
            config.batch_timeout_ms,
            config.worker_count
        );

        // Spawn worker tasks
        for worker_id in 0..config.worker_count {
            let pool = pool.clone();
            let receiver = receiver.clone();
            let config = config.clone();

            tokio::spawn(async move {
                Self::worker_loop(worker_id, pool, receiver, config).await;
            });
        }

        Self { sender }
    }

    /// Log activity (non-blocking, fire-and-forget)
    pub fn log(&self, activity: ActivityLog) {
        // Try to send, if queue full, drop with warning
        if let Err(e) = self.sender.try_send(activity) {
            warn!("Failed to enqueue log (queue full?): {}", e);
            // In production, you might want to increment a metric here
        }
    }

    /// Log activity async (waits if queue full, but doesn't block caller)
    pub fn log_async(&self, activity: ActivityLog) {
        let sender = self.sender.clone();
        tokio::spawn(async move {
            if let Err(e) = sender.send_async(activity).await {
                error!("Failed to send log to queue: {}", e);
            }
        });
    }

    /// Worker loop - processes logs in batches
    async fn worker_loop(
        worker_id: usize,
        pool: PgPool,
        receiver: Receiver<ActivityLog>,
        config: LoggerConfig,
    ) {
        info!("Logger worker {} started", worker_id);
        
        let mut batch: Vec<ActivityLog> = Vec::with_capacity(config.batch_size);
        let batch_timeout = Duration::from_millis(config.batch_timeout_ms);

        loop {
            // Collect batch
            let deadline = tokio::time::Instant::now() + batch_timeout;

            while batch.len() < config.batch_size {
                // Try to receive with timeout
                match tokio::time::timeout_at(deadline, receiver.recv_async()).await {
                    Ok(Ok(log)) => {
                        batch.push(log);
                    }
                    Ok(Err(_)) => {
                        // Channel closed, flush and exit
                        if !batch.is_empty() {
                            Self::flush_batch(&pool, &batch, worker_id).await;
                        }
                        info!("Logger worker {} shutting down (channel closed)", worker_id);
                        return;
                    }
                    Err(_) => {
                        // Timeout, flush what we have
                        break;
                    }
                }
            }

            // Flush batch if not empty
            if !batch.is_empty() {
                Self::flush_batch(&pool, &batch, worker_id).await;
                batch.clear();
            } else {
                // No logs received, sleep a bit to avoid busy loop
                sleep(Duration::from_millis(100)).await;
            }
        }
    }

    /// Flush batch to database
    async fn flush_batch(pool: &PgPool, batch: &[ActivityLog], worker_id: usize) {
        let start = std::time::Instant::now();
        let batch_size = batch.len();

        debug!("Worker {} flushing {} logs to database", worker_id, batch_size);

        match Self::insert_batch(pool, batch).await {
            Ok(inserted) => {
                let duration = start.elapsed();
                debug!(
                    "Worker {} inserted {} logs in {:?} ({:.2} logs/sec)",
                    worker_id,
                    inserted,
                    duration,
                    inserted as f64 / duration.as_secs_f64()
                );
            }
            Err(e) => {
                error!("Worker {} failed to insert batch: {}", worker_id, e);
                // In production, you might want to:
                // - Retry with exponential backoff
                // - Write to fallback file storage
                // - Send alert to monitoring system
            }
        }
    }

    /// Batch insert to database
    async fn insert_batch(pool: &PgPool, logs: &[ActivityLog]) -> Result<usize> {
        // Build bulk insert query
        let mut query_builder = sqlx::QueryBuilder::new(
            r#"
            INSERT INTO tbl_activity_logs (
                session_id, user_id, activity_type, activity_status,
                document_id, message_content, response_content,
                token_count, retrieval_skipped, similarity_score,
                processing_time_ms, llm_call_duration_ms, retrieval_duration_ms,
                error_message, error_type, user_agent, ip_address, created_at
            )
            "#
        );

        query_builder.push_values(logs, |mut b, log| {
            b.push_bind(log.session_id)
                .push_bind(log.user_id)
                .push_bind(log.activity_type.as_str())
                .push_bind(log.activity_status.as_str())
                .push_bind(log.document_id)
                .push_bind(&log.message_content)
                .push_bind(&log.response_content)
                .push_bind(log.token_count)
                .push_bind(log.retrieval_skipped)
                .push_bind(log.similarity_score)
                .push_bind(log.processing_time_ms)
                .push_bind(log.llm_call_duration_ms)
                .push_bind(log.retrieval_duration_ms)
                .push_bind(&log.error_message)
                .push_bind(&log.error_type)
                .push_bind(&log.user_agent)
                .push_bind(log.ip_address)
                .push_bind(log.created_at);
        });

        let query = query_builder.build();
        let result = query.execute(pool).await?;

        Ok(result.rows_affected() as usize)
    }

    /// Get queue statistics (for monitoring)
    pub fn queue_len(&self) -> usize {
        self.sender.len()
    }

    pub fn is_queue_full(&self) -> bool {
        self.sender.is_full()
    }
}

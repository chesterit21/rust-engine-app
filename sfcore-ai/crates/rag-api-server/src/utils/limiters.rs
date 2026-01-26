use anyhow::Result;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

#[derive(Clone)]
pub struct Limiters {
    pub embedding: Arc<Semaphore>,
    pub db_search: Arc<Semaphore>,
    pub llm_generate: Arc<Semaphore>,
    pub llm_stream: Arc<Semaphore>,
    pub acquire_timeout: Duration,
}

impl Limiters {
    pub fn new(cfg: &crate::config::settings::LimitsConfig) -> Self {
        Self {
            embedding: Arc::new(Semaphore::new(cfg.embedding_concurrency.max(1))),
            db_search: Arc::new(Semaphore::new(cfg.db_search_concurrency.max(1))),
            llm_generate: Arc::new(Semaphore::new(cfg.llm_generate_concurrency.max(1))),
            llm_stream: Arc::new(Semaphore::new(cfg.llm_stream_concurrency.max(1))),
            acquire_timeout: Duration::from_millis(cfg.acquire_timeout_ms.max(1)),
        }
    }

    pub async fn acquire_timed(
        sem: Arc<Semaphore>,
        acquire_timeout: Duration,
        op: &'static str,
    ) -> Result<(OwnedSemaphorePermit, Duration)> {
        let start = Instant::now();

        let permit = tokio::time::timeout(acquire_timeout, sem.acquire_owned())
            .await
            .map_err(|_| anyhow::anyhow!("Limiter acquire timeout for op={}", op))??;

        Ok((permit, start.elapsed()))
    }
}

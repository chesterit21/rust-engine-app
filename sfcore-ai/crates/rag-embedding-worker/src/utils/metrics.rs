use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct Metrics {
    inner: Arc<MetricsInner>,
}

struct MetricsInner {
    documents_processed: AtomicU64,
    documents_failed: AtomicU64,
    chunks_created: AtomicU64,
    total_processing_time_ms: AtomicU64,
    embeddings_generated: AtomicU64,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(MetricsInner {
                documents_processed: AtomicU64::new(0),
                documents_failed: AtomicU64::new(0),
                chunks_created: AtomicU64::new(0),
                total_processing_time_ms: AtomicU64::new(0),
                embeddings_generated: AtomicU64::new(0),
            }),
        }
    }
    
    pub fn increment_documents_processed(&self) {
        self.inner.documents_processed.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn increment_documents_failed(&self) {
        self.inner.documents_failed.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn add_chunks_created(&self, count: u64) {
        self.inner.chunks_created.fetch_add(count, Ordering::Relaxed);
    }
    
    pub fn add_processing_time(&self, duration: Duration) {
        self.inner
            .total_processing_time_ms
            .fetch_add(duration.as_millis() as u64, Ordering::Relaxed);
    }
    
    pub fn add_embeddings_generated(&self, count: u64) {
        self.inner.embeddings_generated.fetch_add(count, Ordering::Relaxed);
    }
    
    pub fn get_documents_processed(&self) -> u64 {
        self.inner.documents_processed.load(Ordering::Relaxed)
    }
    
    pub fn get_documents_failed(&self) -> u64 {
        self.inner.documents_failed.load(Ordering::Relaxed)
    }
    
    pub fn get_chunks_created(&self) -> u64 {
        self.inner.chunks_created.load(Ordering::Relaxed)
    }
    
    pub fn get_total_processing_time_ms(&self) -> u64 {
        self.inner.total_processing_time_ms.load(Ordering::Relaxed)
    }
    
    pub fn get_embeddings_generated(&self) -> u64 {
        self.inner.embeddings_generated.load(Ordering::Relaxed)
    }
    
    pub fn get_average_processing_time_ms(&self) -> f64 {
        let processed = self.get_documents_processed();
        if processed == 0 {
            return 0.0;
        }
        
        let total_time = self.get_total_processing_time_ms();
        total_time as f64 / processed as f64
    }
    
    pub fn print_summary(&self) {
        println!("\n📊 === METRICS SUMMARY ===");
        println!("Documents Processed: {}", self.get_documents_processed());
        println!("Documents Failed: {}", self.get_documents_failed());
        println!("Chunks Created: {}", self.get_chunks_created());
        println!("Embeddings Generated: {}", self.get_embeddings_generated());
        println!(
            "Average Processing Time: {:.2}ms",
            self.get_average_processing_time_ms()
        );
        println!(
            "Total Processing Time: {:.2}s",
            self.get_total_processing_time_ms() as f64 / 1000.0
        );
        println!("=========================\n");
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Timer helper untuk measure duration
pub struct Timer {
    start: Instant,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }
    
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}

use crate::time::now_ms;
use std::sync::atomic::{AtomicU64, Ordering};

pub struct Metrics {
    start_ms: u64,
    pub evictions_total: AtomicU64,
    pub events_published_total: AtomicU64,
    pub events_lagged_total: AtomicU64,
    pub invalid_key_total: AtomicU64,
    pub hits_total: AtomicU64,
    pub misses_total: AtomicU64,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            start_ms: now_ms(),
            evictions_total: AtomicU64::new(0),
            events_published_total: AtomicU64::new(0),
            events_lagged_total: AtomicU64::new(0),
            invalid_key_total: AtomicU64::new(0),
            hits_total: AtomicU64::new(0),
            misses_total: AtomicU64::new(0),
        }
    }

    pub fn uptime_ms(&self) -> u64 {
        now_ms().saturating_sub(self.start_ms)
    }

    pub fn inc_evictions(&self, n: u64) {
        self.evictions_total.fetch_add(n, Ordering::Relaxed);
    }
    pub fn inc_published(&self) {
        self.events_published_total.fetch_add(1, Ordering::Relaxed);
    }
    pub fn inc_lagged(&self) {
        self.events_lagged_total.fetch_add(1, Ordering::Relaxed);
    }
    pub fn inc_invalid_key(&self) {
        self.invalid_key_total.fetch_add(1, Ordering::Relaxed);
    }
    pub fn inc_hit(&self) {
        self.hits_total.fetch_add(1, Ordering::Relaxed);
    }
    pub fn inc_miss(&self) {
        self.misses_total.fetch_add(1, Ordering::Relaxed);
    }
}

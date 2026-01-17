use parking_lot::Mutex;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

use crate::config::{Config, RuntimeConfig};
use crate::metrics::Metrics;
use crate::store::KvStore;
use crate::sys::meminfo::{read_meminfo, pressure};

pub struct Evictor {
    ring: Mutex<VecDeque<String>>,
    store: Arc<KvStore>,
    metrics: Arc<Metrics>,
    cfg: Config,
    runtime_cfg: Arc<RuntimeConfig>,
}

impl Evictor {
    pub fn new(store: Arc<KvStore>, metrics: Arc<Metrics>, cfg: Config, runtime_cfg: Arc<RuntimeConfig>) -> Self {
        Self { ring: Mutex::new(VecDeque::new()), store, metrics, cfg, runtime_cfg }
    }

    pub fn on_write(&self, key: &str) {
        self.ring.lock().push_back(key.to_string());
    }

    /// Force eviction until cache memory is below target percentage of available memory
    /// target_bp = basis points (e.g., 5000 = 50%)
    /// Returns number of keys evicted
    /// Force eviction until cache memory is below target percentage of available memory
    /// target_bp = basis points (e.g., 5000 = 50%)
    /// Returns number of keys evicted
    pub fn force_evict_to_target(&self, target_bp: u16) -> usize {
        let mi = match read_meminfo() { Ok(x) => x, Err(_) => return 0 };
        let available_bytes = mi.mem_available_kb * 1024;
        let target_bytes = (available_bytes as f64 * (target_bp as f64 / 10000.0)) as u64;
        
        let mut evicted = 0;
        
        while self.store.approx_mem_bytes() > target_bytes {
            if self.evict_sampled_lru() {
                evicted += 1;
                self.metrics.inc_evictions(1);
            } else {
                break; // Limit reached or empty
            }
        }
        
        evicted
    }

    /// Try to evict one key using Sampled LRU strategy
    fn evict_sampled_lru(&self) -> bool {
        let mut guard = self.ring.lock();
        if guard.is_empty() {
            return false;
        }

        // Sample up to 5 keys from front
        const SAMPLE_SIZE: usize = 5;
        let len = guard.len();
        let sample_count = std::cmp::min(len, SAMPLE_SIZE);
        
        let mut best_idx = 0;
        let mut min_touched = u64::MAX;
        let mut found = false;

        // Check candidates
        // Note: we might encounter keys that are already deleted from store (race), handle gracefully
        let mut i = 0;
        while i < sample_count {
             // If we find a key that is "hot", we should move it to back?
             // For simplicity/perf in v1: just find LRU in the sample. 
             // Ideally we move hot keys to back, but that modifies the deque while iterating.
             // Let's just peer.
             if let Some(key) = guard.get(i) {
                 if let Some(touched) = self.store.peek_touched_at(key) {
                     if touched < min_touched {
                         min_touched = touched;
                         best_idx = i;
                         found = true;
                     }
                 } else {
                     // Key not in store? Data race or inconsistent. Remove it.
                     guard.remove(i);
                     continue; // Don't increment i, next item shifted down
                 }
             }
             i += 1;
        }

        if found {
            if let Some(key) = guard.remove(best_idx) {
                return self.store.del(&key);
            }
        } else if !guard.is_empty() {
            // Fallback: just pop front if we couldn't find stats (shouldn't happen often)
             if let Some(key) = guard.pop_front() {
                return self.store.del(&key);
             }
        }
        
        false
    }

    pub async fn run(self: Arc<Self>) {
        loop {
            sleep(Duration::from_millis(self.cfg.pressure_poll_ms)).await;

            let mi = match read_meminfo() { Ok(x) => x, Err(_) => continue };
            let p = pressure(mi);
            
            // Use runtime config for threshold (allows dynamic updates)
            let threshold = self.runtime_cfg.get_pressure_hot();
            
            if p > threshold {
                // Evict aggressive until cool
                // We do it in batches to release lock
                let mut evicted_batch = 0;
                while evicted_batch < 100 {
                     if self.evict_sampled_lru() {
                         self.metrics.inc_evictions(1);
                         evicted_batch += 1;
                     } else {
                         break;
                     }
                }
                
                // Check if we need to continue?
                // The loop will sleep and check again shortly.
                // If we evicted 100, we probably should check pressure again or yield.
                // The sleep interval will handle yielding.
            } else {
                 // Trim ring buffer if too large (opsional, biar gak leak memory di keys list)
                 let mut guard = self.ring.lock();
                 if guard.len() > 100_000 {
                     guard.drain(..1000);
                 }
            }
        }
    }
}


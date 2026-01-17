use crate::store::entry::Entry;
use bytes::Bytes;
use dashmap::DashMap;
use localcached_proto::ValueFormat;

pub struct KvStore {
    map: DashMap<String, Entry>,
}

impl Default for KvStore {
    fn default() -> Self {
        Self {
            map: DashMap::new(),
        }
    }
}

impl KvStore {
    pub fn set(
        &self,
        key: String,
        format: ValueFormat,
        value: Bytes,
        expires_at_ms: u64,
        now_ms: u64,
    ) {
        let size_bytes = key.len() + value.len();
        let e = Entry {
            format,
            value,
            expires_at_ms,
            touched_ms: now_ms.into(),
            size_bytes,
        };
        self.map.insert(key, e);
    }

    pub fn get(&self, key: &str, now_ms: u64) -> Option<(ValueFormat, Bytes, u64)> {
        let g = self.map.get(key)?;
        if g.is_expired(now_ms) {
            drop(g);
            self.map.remove(key);
            return None;
        }
        g.touch(now_ms);
        let ttl_rem = if g.expires_at_ms == 0 {
            0
        } else {
            g.expires_at_ms.saturating_sub(now_ms)
        };
        Some((g.format, g.value.clone(), ttl_rem))
    }

    pub fn del(&self, key: &str) -> bool {
        self.map.remove(key).is_some()
    }

    pub fn len(&self) -> u64 {
        self.map.len() as u64
    }

    pub fn approx_mem_bytes(&self) -> u64 {
        // Estimasi: sum size_bytes + overhead 64 bytes per entry (konstanta kasar)
        let mut sum = 0u64;
        for r in self.map.iter() {
            sum = sum.saturating_add(r.size_bytes as u64 + 64);
        }
        sum
    }

    /// List all keys matching the given prefix (empty = all keys)
    pub fn keys(&self, prefix: &str, now_ms: u64) -> Vec<String> {
        self.map
            .iter()
            .filter(|r| r.key().starts_with(prefix) && !r.is_expired(now_ms))
            .map(|r| r.key().clone())
            .collect()
    }

    /// Peek at the touched_ms of a key without updating it.
    /// Used by Evictor for LRU sampling.
    pub fn peek_touched_at(&self, key: &str) -> Option<u64> {
        self.map
            .get(key)
            .map(|e| e.touched_ms.load(std::sync::atomic::Ordering::Relaxed))
    }
}

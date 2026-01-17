use bytes::Bytes;
use localcached_proto::ValueFormat;
use std::sync::atomic::{AtomicU64, Ordering};

pub struct Entry {
    pub format: ValueFormat,
    pub value: Bytes,
    pub expires_at_ms: u64, // 0 none
    pub touched_ms: AtomicU64,
    pub size_bytes: usize,
}

impl Entry {
    pub fn is_expired(&self, now: u64) -> bool {
        self.expires_at_ms != 0 && now >= self.expires_at_ms
    }
    pub fn touch(&self, now: u64) {
        self.touched_ms.store(now, Ordering::Relaxed);
    }
}

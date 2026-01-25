use dashmap::DashMap;
use std::sync::Arc;
use sysinfo::System;
use tracing::{debug, info, warn};
use crate::models::chat::SessionId;
use super::types::ConversationState;

/// Thread-safe in-memory conversation cache
/// Uses DashMap for lock-free concurrent access
#[derive(Clone)]
pub struct ConversationCache {
    /// Session storage: session_id -> ConversationState
    storage: Arc<DashMap<SessionId, ConversationState>>,
    
    /// System info for RAM monitoring
    system: Arc<parking_lot::Mutex<System>>,
}

impl ConversationCache {
    /// Create new cache instance
    pub fn new() -> Self {
        info!("Initializing conversation cache with DashMap");
        Self {
            storage: Arc::new(DashMap::new()),
            system: Arc::new(parking_lot::Mutex::new(System::new_all())),
        }
    }

    /// Get conversation state by session_id
    /// Returns None if not found or expired
    pub fn get(&self, session_id: SessionId) -> Option<ConversationState> {
        let entry = self.storage.get(&session_id)?;
        let state = entry.value().clone();

        // Check expiration (lazy deletion)
        if state.is_expired() {
            drop(entry); // Release read lock
            self.remove(session_id);
            debug!("Session {} expired, removed from cache", session_id);
            return None;
        }

        debug!("Retrieved session {} from cache (age: {:?})", 
            session_id, state.created_at.elapsed());
        Some(state)
    }

    /// Insert or update conversation state
    pub fn set(&self, session_id: SessionId, state: ConversationState) {
        self.storage.insert(session_id, state);
        debug!("Updated session {} in cache", session_id);
    }

    /// Remove conversation from cache
    pub fn remove(&self, session_id: SessionId) -> Option<ConversationState> {
        self.storage.remove(&session_id).map(|(_, state)| state)
    }

    /// Get number of active sessions
    pub fn len(&self) -> usize {
        self.storage.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.storage.is_empty()
    }

    /// Check if we can create new session (RAM limit: 90%)
    pub fn can_create_new_session(&self) -> bool {
        let mut sys = self.system.lock();
        sys.refresh_memory();

        let total_memory = sys.total_memory();
        let used_memory = sys.used_memory();
        let usage_percent = (used_memory as f64 / total_memory as f64) * 100.0;

        if usage_percent >= 90.0 {
            warn!(
                "Memory usage at {:.2}% (used: {} MB, total: {} MB), rejecting new session",
                usage_percent,
                used_memory / 1024 / 1024,
                total_memory / 1024 / 1024
            );
            return false;
        }

        debug!("Memory usage: {:.2}%, can create new session", usage_percent);
        true
    }

    /// Cleanup expired sessions (manual trigger)
    /// Returns number of sessions removed
    pub fn cleanup_expired(&self) -> usize {
        let start_len = self.storage.len();
        self.storage.retain(|_, state: &mut ConversationState| !state.is_expired());
        let end_len = self.storage.len();
        
        let count = start_len.saturating_sub(end_len);
        
        if count > 0 {
            info!("Cleaned up {} expired sessions", count);
        }
        
        count
    }

    /// Get cache statistics for monitoring
    pub fn stats(&self) -> CacheStats {
        let mut sys = self.system.lock();
        sys.refresh_memory();

        CacheStats {
            active_sessions: self.len(),
            memory_usage_mb: sys.used_memory() / 1024 / 1024,
            memory_total_mb: sys.total_memory() / 1024 / 1024,
            memory_usage_percent: (sys.used_memory() as f64 / sys.total_memory() as f64) * 100.0,
        }
    }
}

impl Default for ConversationCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub active_sessions: usize,
    pub memory_usage_mb: u64,
    pub memory_total_mb: u64,
    pub memory_usage_percent: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_basic_operations() {
        let cache = ConversationCache::new();
        let session_id = 20260125040900123;
        let state = ConversationState::new(session_id, 123, None);

        // Insert
        cache.set(session_id, state.clone());
        assert_eq!(cache.len(), 1);

        // Get
        let retrieved = cache.get(session_id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().session_id, session_id);

        // Remove
        cache.remove(session_id);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_can_create_new_session() {
        let cache = ConversationCache::new();
        // Should always be true in test environment
        assert!(cache.can_create_new_session());
    }

    #[test]
    fn test_stats() {
        let cache = ConversationCache::new();
        let stats = cache.stats();
        assert!(stats.memory_total_mb > 0);
        assert!(stats.memory_usage_percent >= 0.0);
    }
}

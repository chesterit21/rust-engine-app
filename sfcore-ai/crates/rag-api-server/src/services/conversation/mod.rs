//! Conversation memory management module
//! 
//! Provides in-memory conversation state management with:
//! - Thread-safe caching (DashMap)
//! - Dynamic system context building
//! - Token management and sliding window
//! - Context-aware retrieval decisions

mod cache;
mod context_builder;
pub mod manager;
mod token_counter;
pub mod types;

pub use cache::{ConversationCache, CacheStats};
pub use context_builder::ContextBuilder;
pub use manager::ConversationManager;
pub use token_counter::TokenCounter;
pub use types::{
    ConversationState, RetrievalDecision,
    SystemContextComponents, WeightedEmbeddingConfig,
};

// Re-export common types for convenience if needed, but ChatMessage/SessionId are in models
pub use crate::models::chat::{ChatMessage, SessionId};

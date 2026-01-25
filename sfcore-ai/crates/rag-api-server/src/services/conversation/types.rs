use std::time::Instant;
use crate::models::chat::{ChatMessage, SessionId};

/// Complete conversation state stored in memory cache
#[derive(Debug, Clone)]
pub struct ConversationState {
    /// Session identifier
    pub session_id: SessionId,
    
    /// User who owns this conversation
    pub user_id: i64,
    
    /// Current document context (None if general chat)
    pub document_id: Option<i64>,
    
    /// Message history (max 10 items = 5 user+assistant pairs)
    pub messages: Vec<ChatMessage>,
    
    /// Current System prompt (includes base instruction + retrieval context)
    pub system_context: String,
    
    /// Cached summarized retrieval results (for reuse when skipping retrieval)
    pub last_retrieval_summary: String,
    
    /// Embedding of last query (for similarity comparison)
    pub last_query_embedding: Option<Vec<f32>>,
    
    /// Session creation time (for 6-hour absolute expiration)
    pub created_at: Instant,
    
    /// Last activity timestamp (for monitoring)
    pub last_activity: Instant,
    
    /// Metadata for analytics
    pub metadata: ConversationMetadata,
}

impl ConversationState {
    /// Create new conversation session
    pub fn new(session_id: SessionId, user_id: i64, document_id: Option<i64>) -> Self {
        let now = Instant::now();
        Self {
            session_id,
            user_id,
            document_id,
            messages: Vec::with_capacity(10), // Pre-allocate for 5 pairs
            system_context: String::new(),
            last_retrieval_summary: String::new(),
            last_query_embedding: None,
            created_at: now,
            last_activity: now,
            metadata: ConversationMetadata::default(),
        }
    }

    /// Check if session is expired (6 hours from creation)
    pub fn is_expired(&self) -> bool {
        const SIX_HOURS_SECS: u64 = 6 * 60 * 60;
        self.created_at.elapsed().as_secs() > SIX_HOURS_SECS
    }

    /// Update last activity timestamp
    pub fn touch(&mut self) {
        self.last_activity = Instant::now();
    }

    /// Get number of message pairs (user + assistant = 1 pair)
    pub fn message_pair_count(&self) -> usize {
        self.messages.len() / 2
    }

    /// Check if we can enforce sliding window (>= 5 pairs)
    pub fn needs_window_enforcement(&self) -> bool {
        self.message_pair_count() >= 5
    }
}

/// Conversation metadata for analytics
#[derive(Debug, Clone, Default)]
pub struct ConversationMetadata {
    /// Total messages exchanged (user + assistant)
    pub total_messages: usize,
    
    /// Total retrieval operations performed
    pub total_retrievals: usize,
    
    /// How many times retrieval was skipped (reused context)
    pub retrieval_skipped_count: usize,
    
    /// Last known total token count
    pub total_tokens_last: usize,
}

/// System context components (for building dynamic System message)
#[derive(Debug, Clone)]
pub struct SystemContextComponents {
    /// Fixed base instruction (doesn't change)
    pub base_instruction: String,
    
    /// Dynamic retrieval context (changes per query or reused)
    pub retrieval_context: String,
    
    /// Optional metadata section (document info)
    pub metadata_section: Option<String>,
}

impl SystemContextComponents {
    /// Build complete System message content
    pub fn build(&self) -> String {
        let mut parts = vec![
            self.base_instruction.clone(),
            String::new(), // Empty line
            self.retrieval_context.clone(),
        ];

        if let Some(metadata) = &self.metadata_section {
            parts.push(String::new());
            parts.push(metadata.clone());
        }

        parts.join("\n")
    }
}

/// Token counting result
#[derive(Debug, Clone)]
pub struct TokenCount {
    pub total: usize,
    pub system_tokens: usize,
    pub history_tokens: usize,
    pub current_message_tokens: usize,
}

impl TokenCount {
    pub fn is_over_soft_limit(&self) -> bool {
        self.total > 20_000
    }

    pub fn is_over_hard_limit(&self) -> bool {
        self.total > 23_000
    }
}

/// Retrieval decision result
#[derive(Debug, Clone)]
pub enum RetrievalDecision {
    /// Need to perform new retrieval
    Retrieve {
        reason: RetrievalReason,
        context_aware: bool,  // Use weighted embedding?
    },
    /// Skip retrieval, reuse previous context
    Skip {
        reason: SkipReason,
    },
}

#[derive(Debug, Clone)]
pub enum RetrievalReason {
    FirstMessage,
    DocumentIdChanged,
    LowSimilarity(f32),  // Similarity score
}

#[derive(Debug, Clone)]
pub enum SkipReason {
    SameDocumentAndHighSimilarity(f32),
}

/// Weighted embedding configuration
#[derive(Debug, Clone)]
pub struct WeightedEmbeddingConfig {
    pub current_weight: f32,   // 0.7
    pub history_weight: f32,   // 0.3
    pub max_history_messages: usize, // Max 5
}

impl Default for WeightedEmbeddingConfig {
    fn default() -> Self {
        Self {
            current_weight: 0.7,
            history_weight: 0.3,
            max_history_messages: 5,
        }
    }
}

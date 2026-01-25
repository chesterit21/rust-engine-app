use serde::{Deserialize, Serialize};
use unicode_segmentation::UnicodeSegmentation;

/// OpenAI-compatible message format (SHARED across all modules)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,      // "user" | "assistant" | "system"
    pub content: String,
}

impl ChatMessage {
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
        }
    }

    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: content.into(),
        }
    }

    /// Estimate token count for this message
    pub fn estimate_tokens(&self) -> usize {
        let role_chars = self.role.graphemes(true).count();
        let content_chars = self.content.graphemes(true).count();
        let total_chars = role_chars + content_chars;
        
        // Random 2-3 chars per token + overhead (deterministic for same text roughly?) 
        // Logic copied from token_counter but simplified or we can rely on token_counter
        // For simple estimation:
        ((total_chars + 2) / 3).max(1) + 3
    }
}

/// Session ID type
pub type SessionId = i64;

/// Chat request payload
#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub user_id: i64,
    pub session_id: SessionId,
    pub message: String,
    pub document_id: Option<i64>,
}

/// Chat response (for non-streaming)
#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub session_id: SessionId,
    pub message: String,
    pub sources: Vec<SourceInfo>,
}

/// Source information for citations
#[derive(Debug, Serialize, Clone)]
pub struct SourceInfo {
    pub document_id: i64,
    pub document_title: String,
    pub chunk_id: i64,
    pub similarity: f32,
}

/// Streaming event types for SSE
#[derive(Debug, Serialize)]
#[serde(tag = "event", content = "data")]
pub enum StreamEvent {
    #[serde(rename = "sources")]
    Sources(Vec<SourceInfo>),
    
    #[serde(rename = "message")]
    Message(String),
    
    #[serde(rename = "done")]
    Done,
    
    #[serde(rename = "error")]
    Error { message: String },
}

/// Generate new session ID for user
#[derive(serde::Deserialize)]
pub struct NewSessionRequest {
    pub user_id: i64,
}

#[derive(serde::Serialize)]
pub struct NewSessionResponse {
    pub session_id: i64,
}

/// Cache statistics response
#[derive(serde::Serialize)]
pub struct CacheStatsResponse {
    pub active_sessions: usize,
    pub memory_usage_mb: u64,
    pub memory_total_mb: u64,
    pub memory_usage_percent: f64,
}

/// Cleanup response
#[derive(serde::Serialize)]
pub struct CleanupResponse {
    pub sessions_removed: usize,
}

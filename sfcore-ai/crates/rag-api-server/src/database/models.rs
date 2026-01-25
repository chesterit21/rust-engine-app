use chrono::{DateTime, Utc};
use pgvector::Vector;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct UserDocument {
    pub document_id: i32,
    pub owner_user_id: i32,
    pub document_title: String,
    pub created_at: DateTime<Utc>,
    pub user_id: i32,
    pub permission_level: String,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct DocumentChunk {
    pub chunk_id: i64,
    pub document_id: i32,
    pub document_title: String,
    pub content: String,
    pub similarity: f32,
    pub chunk_index: i32,
    pub page_number: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub chunk_id: i64,
    pub document_id: i32,
    pub document_title: String,
    pub content: String,
    pub score: f32,
    pub chunk_index: i32,
    pub page_number: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String, // "user" atau "assistant"
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSession {
    pub session_id: String,
    pub user_id: i32,
    pub document_id: Option<i32>,
    pub messages: Vec<ChatMessage>,
    pub created_at: DateTime<Utc>,
}
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct DocumentProcessingStatus {
    pub document_id: i32,
    pub status: String,
    pub progress: f32,
    pub message: Option<String>,
    pub updated_at: DateTime<Utc>,
}

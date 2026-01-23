use chrono::{DateTime, Utc};
use pgvector::Vector;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentNotification {
    pub operation: String,  // INSERT, UPDATE
    pub document_id: i32,
    pub file_path: String,
    pub timestamp: f64,
}

#[derive(Debug, Clone, FromRow)]
pub struct DocumentFile {
    #[sqlx(rename = "DocumentID")]
    pub document_id: i32,
    #[sqlx(rename = "DocumentFilePath")]
    pub document_file_path: String,
}

#[derive(Debug, Clone)]
pub struct DocumentChunk {
    pub document_id: i32,
    pub tenant_id: Option<i32>,
    pub chunk_index: i32,
    pub content: String,
    pub char_count: i32,
    pub token_count: Option<i32>,
    pub embedding: Vector,
    pub page_number: Option<i32>,
    pub section: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, FromRow)]
pub struct IngestionLog {
    pub document_id: i32,
    pub file_path: String,
    pub file_size: Option<i64>,
    pub file_type: Option<String>,
    pub embedding_model: String,
    pub chunk_size: i32,
    pub chunk_overlap: i32,
    pub status: String,
    pub total_chunks: i32,
    pub processed_chunks: i32,
    pub last_error: Option<String>,
    pub retry_count: i32,
    pub started_at: Option<DateTime<Utc>>,
    pub processed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IngestionStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

impl ToString for IngestionStatus {
    fn to_string(&self) -> String {
        match self {
            Self::Pending => "pending".to_string(),
            Self::Processing => "processing".to_string(),
            Self::Completed => "completed".to_string(),
            Self::Failed => "failed".to_string(),
        }
    }
}

impl From<String> for IngestionStatus {
    fn from(s: String) -> Self {
        match s.as_str() {
            "pending" => Self::Pending,
            "processing" => Self::Processing,
            "completed" => Self::Completed,
            "failed" => Self::Failed,
            _ => Self::Pending,
        }
    }
}

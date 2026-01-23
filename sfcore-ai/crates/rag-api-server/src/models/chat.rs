use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

// ===== REQUEST MODELS =====

#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub user_id: String,
    pub message: String,
    #[serde(default)]
    pub document_upload: Option<Vec<DocumentUpload>>,
    #[serde(default)]
    pub document_selected: Option<Vec<String>>,  // document_id array
    #[serde(default)]
    pub session_id: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DocumentUpload {
    pub file_name: String,
    pub file_base64: String,
    pub file_type: String,
}

// ===== RESPONSE EVENT MODELS =====

#[derive(Debug, Serialize)]
pub struct SessionInfo {
    pub session_id: String,
    pub user_id: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct StatusInfo {
    pub stage: String,  // uploading, parsing, embedding, retrieving, generating
    pub message: String,
    pub progress: u8,  // 0-100
}

#[derive(Debug, Serialize)]
pub struct UploadedDocInfo {
    pub document_id: i32,
    pub file_name: String,
    pub status: String,  // success, failed
    pub chunks_created: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SourceInfo {
    pub document_id: i32,
    pub document_name: String,
    pub chunk_id: i64,
    pub similarity: f32,
    pub page_number: Option<i32>,
    pub preview: String,  // first 150 chars
    pub download_url: String,
    pub view_url: String,
}

#[derive(Debug, Serialize)]
pub struct MessageChunk {
    pub delta: String,  // streaming text chunk
}

#[derive(Debug, Serialize)]
pub struct CompletionInfo {
    pub session_id: String,
    pub message_id: String,
    pub sources_count: usize,
    pub processing_time_ms: u64,
}

#[derive(Debug, Serialize)]
pub struct ErrorInfo {
    pub code: String,
    pub message: String,
}

use crate::services::event_bus::SystemEvent;
use crate::state::AppState;
use crate::utils::error::ApiError;
use axum::{
    extract::{Multipart, State},
    Json,
};
use serde::Serialize;
use std::sync::Arc;
use tracing::{error, info};

#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub success: bool,
    pub message: String,
    pub document_id: Option<i32>,
    pub chunks_created: usize,
}

pub async fn upload_handler(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, ApiError> {
    info!("File upload request received");
    
    let mut user_id: Option<i32> = None;
    let mut session_id: Option<i64> = None;
    let mut file_data: Option<Vec<u8>> = None;
    let mut filename: Option<String> = None;
    
    // Parse multipart form
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::BadRequest(format!("Failed to read field: {}", e)))?
    {
        let field_name = field.name().unwrap_or("").to_string();
        
        match field_name.as_str() {
            "user_id" => {
                let text = field
                    .text()
                    .await
                    .map_err(|e| ApiError::BadRequest(format!("Invalid user_id: {}", e)))?;
                user_id = Some(
                    text.parse()
                        .map_err(|_| ApiError::BadRequest("user_id must be integer".to_string()))?,
                );
            }
            "session_id" => {
                let text = field
                    .text()
                    .await
                    .map_err(|e| ApiError::BadRequest(format!("Invalid session_id: {}", e)))?;
                session_id = Some(
                    text.parse()
                        .map_err(|_| ApiError::BadRequest("session_id must be integer".to_string()))?,
                );
            }
            "file" => {
                filename = field.file_name().map(|s| s.to_string());
                file_data = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|e| ApiError::BadRequest(format!("Failed to read file: {}", e)))?
                        .to_vec(),
                );
            }
            _ => {}
        }
    }
    
    let user_id = user_id.ok_or_else(|| ApiError::BadRequest("user_id required".to_string()))?;
    let session_id = session_id.ok_or_else(|| ApiError::BadRequest("session_id required".to_string()))?;
    let file_data =
        file_data.ok_or_else(|| ApiError::BadRequest("file required".to_string()))?;
    let filename =
        filename.ok_or_else(|| ApiError::BadRequest("filename required".to_string()))?;
    
    info!("Starting background processing for user {} (session {}): {}", user_id, session_id, filename);
    
    // Create repositories and services for the background task
    let repository = Arc::new(crate::database::Repository::new(state.db_pool.clone()));
    let embedding_service = state.embedding_service.clone();
    let event_bus = state.event_bus.clone();
    let doc_service = crate::services::DocumentService::new(repository, embedding_service);
    
    // Spawn background task
    tokio::spawn(async move {
        // 1. Notify start
        event_bus.publish(session_id, SystemEvent::ProcessingStarted { 
            document_id: 0, // Not yet known
            filename: filename.clone(),
        });

        // 2. Process
        let eb_clone = event_bus.clone();
        let on_progress = move |progress: f32, message: String, status_flag: String| {
            eb_clone.publish(session_id, SystemEvent::ProcessingProgress { 
                document_id: 0, 
                progress, 
                message, 
                status_flag 
            });
        };

        match doc_service.process_upload(user_id, filename.clone(), file_data, on_progress).await {
            Ok((doc_id, chunks_count)) => {
                info!("Background processing completed for doc {}", doc_id);
                event_bus.publish(session_id, SystemEvent::ProcessingCompleted { 
                    document_id: doc_id, 
                    chunks_count 
                });
            }
            Err(e) => {
                error!("Background processing failed for {}: {}", filename, e);
                event_bus.publish(session_id, SystemEvent::ProcessingError { 
                    document_id: 0, 
                    error: e.to_string() 
                });
            }
        }
    });
    
    Ok(Json(UploadResponse {
        success: true,
        message: "File received and is being processed in the background".to_string(),
        document_id: None, // Will be provided via SSE once ready
        chunks_created: 0,
    }))
}

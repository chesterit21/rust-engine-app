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
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::time::Duration;

#[derive(Debug, Serialize)]
pub struct UploadResponse {
    #[serde(rename = "documentId")]
    pub document_id: i32,
    #[serde(rename = "documentName")]
    pub document_name: String,
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
    let doc_service = crate::services::DocumentService::new(
        repository.clone(),
        embedding_service,
        state.llm_service.clone(),
        &state.settings.rag,
        &state.settings.limits, // Fix: Add missing argument
    );
    
    // 1. Create document record & save file (Sync)
    let (doc_id, file_type, file_data) = match doc_service.create_initial_document(user_id, filename.clone(), file_data).await {
        Ok(res) => res,
        Err(e) => return Err(e),
    };

    // 2. Spawn background task for heavy processing
    let filename_clone = filename.clone();
    let repo_clone = repository.clone(); // Clone for error handling
    let cm_clone = state.conversation_manager.clone();
    
    // ATTACH DOCUMENT TO SESSION (IMPLICIT CONTEXT)
    // This ensures subsequent chat requests without explicit document_ids will use this document.
    tokio::spawn(async move {
        if let Err(e) = cm_clone.attach_document_to_session(session_id, user_id as i64, doc_id as i64).await {
            error!("Failed to attach document {} to session {}: {}", doc_id, session_id, e);
        }
    });

    tokio::spawn(async move {
        // Notify start
        event_bus.publish(session_id, SystemEvent::ProcessingStarted { 
            document_id: doc_id,
            filename: filename_clone.clone(),
        });

        // 3. Process Logic with REAL Progress Reporting
        // We capture event_bus to publish actual progress from the doc service
        let eb_progress = event_bus.clone();
        
        let on_progress_real = move |doc_id_cb: i32, progress: f64, message: String, status_flag: String| {
            // Log for debug
            // info!("Progress Update: Doc {}, {:.1}%, {}", doc_id_cb, progress * 100.0, message);
            
            // Publish to EventBus so frontend receives SSE
            eb_progress.publish(session_id, SystemEvent::ProcessingProgress { 
                document_id: doc_id, 
                progress: progress * 100.0, // Scale 0.0-1.0 to 0-100
                message: message, 
                status_flag: status_flag 
            });
        };

        match doc_service.process_document_background(doc_id, file_type, file_data, on_progress_real).await {
            Ok((_, chunks_count)) => {
                info!("Background processing completed for doc {}", doc_id);
                // Immediately send 100%
                event_bus.publish(session_id, SystemEvent::ProcessingCompleted { 
                    document_id: doc_id, 
                    chunks_count 
                });
            }
            Err(e) => {
                error!("Background processing failed for {}: {}", filename_clone, e);
                // Update DB status to failed so it disappears from progress bar
                let _ = repo_clone.upsert_document_processing_status(doc_id, "failed", 0.0, Some(e.to_string())).await;
                
                event_bus.publish(session_id, SystemEvent::ProcessingError { 
                    document_id: doc_id, 
                    error: e.to_string() 
                });
            }
        }
    });
    
    Ok(Json(UploadResponse {
        document_id: doc_id,
        document_name: filename,
    }))
}

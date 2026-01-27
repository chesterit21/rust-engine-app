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

        // Shared flag to stop the fake progress ticker
        let is_processing = Arc::new(AtomicBool::new(true));
        let is_processing_clone = is_processing.clone();
        
        // Spawn Fake Progress Ticker (Simulated)
        let eb_ticker = event_bus.clone();
        let _filename_ticker = filename_clone.clone();
        
        tokio::spawn(async move {
            let mut fake_progress = 0.0;
            // Loop until processing is done, maxing out at 90%
            while is_processing_clone.load(Ordering::Relaxed) && fake_progress < 90.0 {
                fake_progress += 5.0; // Naik 5% setiap tick
                if fake_progress > 90.0 { fake_progress = 90.0; }
                
                eb_ticker.publish(session_id, SystemEvent::ProcessingProgress { 
                    document_id: doc_id, 
                    progress: fake_progress, 
                    message: format!("Processing... {:.0}%", fake_progress), 
                    status_flag: "processing".to_string() 
                });
                
                // Sleep 500ms
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        });

        // 3. Process Logic (Silent - No real progress reported to EventBus)
        // We pass a dummy closure because we rely on the fake ticker for UI feedback
        let on_progress_silent = |_, _, _, _| {
             // Do nothing (Silent)
        };

        match doc_service.process_document_background(doc_id, file_type, file_data, on_progress_silent).await {
            Ok((_, chunks_count)) => {
                // Stop ticker
                is_processing.store(false, Ordering::Relaxed);
                
                info!("Background processing completed for doc {}", doc_id);
                // Immediately send 100%
                event_bus.publish(session_id, SystemEvent::ProcessingCompleted { 
                    document_id: doc_id, 
                    chunks_count 
                });
            }
            Err(e) => {
                // Stop ticker
                is_processing.store(false, Ordering::Relaxed);
                
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

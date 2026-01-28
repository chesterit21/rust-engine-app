use crate::services::gemini::GeminiService;
use crate::services::gemini_document::GeminiDocumentService;
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
    #[serde(rename = "documentId")]
    pub document_id: i32,
    #[serde(rename = "documentName")]
    pub document_name: String,
}

pub async fn upload_handler_gemini(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, ApiError> {
    info!("(Gemini) File upload request received");
    
    let mut user_id: Option<i32> = None;
    let mut session_id: Option<i64> = None;
    let mut file_data: Option<Vec<u8>> = None;
    let mut filename: Option<String> = None;
    
    // Parse multipart form
    while let Some(field) = multipart.next_field().await.map_err(|e| ApiError::BadRequest(format!("Failed to read field: {}", e)))? {
        let field_name = field.name().unwrap_or("").to_string();
        match field_name.as_str() {
            "user_id" => {
                let text = field.text().await.map_err(|e| ApiError::BadRequest(format!("Invalid user_id: {}", e)))?;
                user_id = Some(text.parse().map_err(|_| ApiError::BadRequest("user_id must be integer".to_string()))?);
            }
            "session_id" => {
                let text = field.text().await.map_err(|e| ApiError::BadRequest(format!("Invalid session_id: {}", e)))?;
                session_id = Some(text.parse().map_err(|_| ApiError::BadRequest("session_id must be integer".to_string()))?);
            }
            "file" => {
                filename = field.file_name().map(|s| s.to_string());
                file_data = Some(field.bytes().await.map_err(|e| ApiError::BadRequest(format!("Failed to read file: {}", e)))?.to_vec());
            }
            _ => {}
        }
    }
    
    let user_id = user_id.ok_or_else(|| ApiError::BadRequest("user_id required".to_string()))?;
    let session_id = session_id.ok_or_else(|| ApiError::BadRequest("session_id required".to_string()))?;
    let file_data = file_data.ok_or_else(|| ApiError::BadRequest("file required".to_string()))?;
    let filename = filename.ok_or_else(|| ApiError::BadRequest("filename required".to_string()))?;
    
    info!("Starting Gemini background processing for user {} (session {}): {}", user_id, session_id, filename);

    // Initialize Services
    let repository = Arc::new(crate::database::Repository::new(state.db_pool.clone()));
    let limiters = state.limiters.clone(); // Re-use global limiters
    
    // Check if Gemini Config exists (SAFETY CHECK)
    let gemini_config = state.settings.gemini.clone()
        .ok_or_else(|| ApiError::InternalError("Gemini handlers called but Gemini config is missing!".to_string()))?;

    let gemini_service = Arc::new(GeminiService::new(gemini_config, limiters));
    
    let doc_service = GeminiDocumentService::new(
        repository.clone(),
        gemini_service,
        state.settings.rag.chunk_size,
        (state.settings.rag.chunk_size as f32 * state.settings.rag.chunk_overlap_percentage) as usize,
        state.settings.rag.document_path.clone(),
    );

    // 1. Create Initial Record (Sync)
    // We need to expose create_document_record logic or just use process_upload directly.
    // However, GeminiDocumentService::process_upload does everything including creation.
    // AND it takes ownership of file_data.
    // BUT we need doc_id immediately to return to user.
    // SO, process_upload must support a callback for "Created" or we call create_initial separate.
    // Looking at `GeminiDocumentService`, `process_upload` calls `create_document_record` internally.
    // It returns Result<(i32, usize)>.
    // If we await `process_upload` here, the REQUEST blocks until embedding is done (potentially slow).
    // We MUST spawn it. But we need `doc_id` to return.
    
    // SOLUTION: We'll modify `GeminiDocumentService` (or just use it differently) in a future iteration if needed,
    // BUT for now, let's replicate the flow from `handlers/upload.rs`:
    // 1. Create record Sync.
    // 2. Spawn Async.
    
    // To do this strictly, we need to access `create_initial_document` or similar from `GeminiDocumentService`.
    // Wait, the plan was strict separation.
    // Implementation: I'll use the `process_upload` in background. 
    // BUT I can't return the ID before creating it.
    // Let's create the record manually here using Repository? No, that violates abstraction.
    // I will call `doc_service.create_document_record` (I need to make it public? It's currently private in my impl).
    // Let me check my previous file write for `services/gemini_document.rs`.
    // It has `create_document_record` as private (no `pub`).
    
    // FIX: I will instantiate `GeminiDocumentService` inside the task, but for the ID...
    // I made a small mistake in the plan by not exposing `create_initial_document`.
    // I will add a `create_initial_document` public method to `GeminiDocumentService` quickly via a patch, OR
    // simpler: I'll just clone the service (it's internal deps are Arc) and use it.
    // For now, I will use a placeholder ID or better: Update `GeminiDocumentService` to expose creation.
    
    // Actually, to avoid editing the file I just wrote unnecessarily, I will use a different pattern:
    // I will spawn the task, and return specific "Pending" ID? No, frontend expects ID.
    // Okay, I will define `create_initial_document` in `GeminiDocumentService` to remain consistent with legacy.
    
    // Let's assume I will patch it in the next step. I'll write the handler assuming `create_initial_document` exists.
    
    // WAIT! I can use `doc_service.process_upload` directly IF I accept that this handler blocks?
    // User hates waiting. 
    // I MUST patch `gemini_document.rs` to make `create_document_record` public OR split the logic.
    
    // Let's write the handler assuming `create_initial_document` exists, and then I will apply a patch to `gemini_document.rs`.
    
    // Wait, `handlers/upload.rs` uses `doc_service.create_initial_document`.
    // So I should implement that in `GeminiDocumentService`.
    
    // Let's write the handler code now.
    
    // Clone for background
    let event_bus = state.event_bus.clone();
    let filename_clone = filename.clone();
    
    // We need to initialize doc_service inside the handler scope to call create_initial
    // Then move it (or a clone) to the task.
    let doc_service = Arc::new(doc_service);
    
    // 1. Create Initial (I'll patch the service to add this method)
    let (doc_id, total_size) = doc_service.create_initial_document(user_id, &filename, &file_data).await?; // Assuming (id, size)
    
    // 2. Spawn Background
    let doc_service_bg = doc_service.clone();
    let file_data_bg = file_data.clone(); // Clone data for bg task
    
    tokio::spawn(async move {
        // Event: Started
        event_bus.publish(session_id, SystemEvent::ProcessingStarted { 
            document_id: doc_id,
            filename: filename_clone,
        });

        // Callback for REAL progress
        let eb_clone = event_bus.clone();
        let on_progress = move |_id, progress, msg, status| {
            eb_clone.publish(session_id, SystemEvent::ProcessingProgress { 
                document_id: doc_id, 
                progress: progress * 100.0, // Scale 0.0-1.0 to 0-100
                message: msg, 
                status_flag: status 
            });
        };
        
        // Execute - we use a new method `process_existing_document` or `resume_processing`?
        // My `process_upload` in `GeminiDocumentService` does EVERYTHING (Create -> Chunk -> Embed).
        // I should stick to `process_upload` but I need the ID first.
        // If I use `process_upload` in background, I can't return ID to user.
        
        // Refined Plan:
        // I will MODIFY `GeminiDocumentService` to split `process_upload` into `create_initial` and `process_background`.
        // This mirrors the legacy service and solves the ID problem.
        
        match doc_service_bg.process_document_background(doc_id, &file_data_bg, on_progress).await {
             Ok((_, count)) => {
                 event_bus.publish(session_id, SystemEvent::ProcessingCompleted { 
                    document_id: doc_id, 
                    chunks_count: count 
                });
             }
             Err(e) => {
                 error!("Gemini processing failed (Swallowed for UI): {}", e);
                 // USER REQUEST: Always report success to UI once file is saved
                 event_bus.publish(session_id, SystemEvent::ProcessingCompleted { 
                    document_id: doc_id, 
                    chunks_count: 0 // Indicate 0 chunks but "success" status
                });
             }
        }
    });

    Ok(Json(UploadResponse {
        document_id: doc_id,
        document_name: filename,
    }))
}

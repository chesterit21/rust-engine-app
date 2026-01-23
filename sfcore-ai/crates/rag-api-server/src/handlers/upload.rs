use crate::database::Repository;
use crate::security::DocumentAuthorization;
use crate::services::{DocumentService, EmbeddingService};
use crate::utils::error::ApiError;
use axum::{
    extract::{Extension, Multipart},
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
    Extension(repository): Extension<Arc<Repository>>,
    Extension(embedding_service): Extension<Arc<EmbeddingService>>,
    Extension(_doc_auth): Extension<Arc<DocumentAuthorization>>,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, ApiError> {
    info!("File upload request received");
    
    let mut user_id: Option<i32> = None;
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
    let file_data =
        file_data.ok_or_else(|| ApiError::BadRequest("file required".to_string()))?;
    let filename =
        filename.ok_or_else(|| ApiError::BadRequest("filename required".to_string()))?;
    
    info!("Processing upload from user {}: {}", user_id, filename);
    
    // Process document
    let document_service = DocumentService::new(repository.clone(), embedding_service.clone());
    
    match document_service
        .process_upload(user_id, filename, file_data)
        .await
    {
        Ok((document_id, chunks_count)) => {
            info!(
                "Successfully processed document {} with {} chunks",
                document_id, chunks_count
            );
            
            Ok(Json(UploadResponse {
                success: true,
                message: "Document processed successfully".to_string(),
                document_id: Some(document_id),
                chunks_created: chunks_count,
            }))
        }
        Err(e) => {
            error!("Failed to process upload: {}", e);
            Err(ApiError::InternalError(format!(
                "Failed to process document: {}",
                e
            )))
        }
    }
}

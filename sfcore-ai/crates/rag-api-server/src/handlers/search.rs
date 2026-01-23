use crate::database::Repository;
use crate::security::DocumentAuthorization;
use crate::services::EmbeddingService;
use crate::utils::error::ApiError;
use axum::{extract::Extension, Json};
use pgvector::Vector;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;

#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    pub user_id: i32,
    pub query: String,
    pub document_id: Option<i32>,
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub total: usize,
}

#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub document_id: i32,
    pub document_title: String,
    pub chunk_id: i64,
    pub content: String,
    pub similarity: f32,
    pub page_number: Option<i32>,
}

pub async fn search_handler(
    Extension(embedding_service): Extension<Arc<EmbeddingService>>,
    Extension(repository): Extension<Arc<Repository>>,
    Extension(doc_auth): Extension<Arc<DocumentAuthorization>>,
    Json(request): Json<SearchRequest>,
) -> Result<Json<SearchResponse>, ApiError> {
    info!("Search request from user {}: {}", request.user_id, request.query);
    
    // Validate document access jika specified
    if let Some(doc_id) = request.document_id {
        doc_auth.require_access(request.user_id, doc_id).await?;
    }
    
    // Generate query embedding
    let query_embedding = embedding_service.embed(&request.query).await?;
    let vector = Vector::from(query_embedding);
    
    // Search
    let limit = request.limit.unwrap_or(10).min(50) as i32;
    let chunks = repository
        .search_user_documents(request.user_id, vector, limit, request.document_id)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;
    
    // Convert to response
    let results: Vec<SearchResult> = chunks
        .into_iter()
        .map(|chunk| SearchResult {
            document_id: chunk.document_id,
            document_title: chunk.document_title,
            chunk_id: chunk.chunk_id,
            content: chunk.content,
            similarity: chunk.similarity,
            page_number: chunk.page_number,
        })
        .collect();
    
    let total = results.len();
    
    Ok(Json(SearchResponse { results, total }))
}

#[derive(Debug, Deserialize)]
pub struct ListDocumentsRequest {
    pub user_id: i32,
}

#[derive(Debug, Serialize)]
pub struct ListDocumentsResponse {
    pub documents: Vec<DocumentInfo>,
    pub total: usize,
}

#[derive(Debug, Serialize)]
pub struct DocumentInfo {
    pub document_id: i32,
    pub title: String,
    pub owner_user_id: i32,
    pub permission_level: String,
    pub created_at: String,
}

pub async fn list_documents_handler(
    Extension(repository): Extension<Arc<Repository>>,
    Json(request): Json<ListDocumentsRequest>,
) -> Result<Json<ListDocumentsResponse>, ApiError> {
    info!("List documents request from user {}", request.user_id);
    
    let docs = repository
        .get_user_documents(request.user_id)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;
    
    let documents: Vec<DocumentInfo> = docs
        .into_iter()
        .map(|doc| DocumentInfo {
            document_id: doc.document_id,
            title: doc.document_title,
            owner_user_id: doc.owner_user_id,
            permission_level: doc.permission_level,
            created_at: doc.created_at.to_rfc3339(),
        })
        .collect();
    
    let total = documents.len();
    
    Ok(Json(ListDocumentsResponse { documents, total }))
}

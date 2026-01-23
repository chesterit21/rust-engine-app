use crate::database::Repository;
use crate::utils::error::ApiError;
use anyhow::Result;
use std::sync::Arc;
use tracing::{debug, warn};

/// Document authorization service
pub struct DocumentAuthorization {
    repository: Arc<Repository>,
}

impl DocumentAuthorization {
    pub fn new(repository: Arc<Repository>) -> Self {
        Self { repository }
    }
    
    /// Check if user has access to document
    pub async fn check_access(&self, user_id: i32, document_id: i32) -> Result<bool, ApiError> {
        let has_access = self
            .repository
            .check_user_document_access(user_id, document_id)
            .await
            .map_err(|e| ApiError::DatabaseError(e.to_string()))?;
        
        if !has_access {
            warn!(
                "User {} denied access to document {}",
                user_id, document_id
            );
        } else {
            debug!("User {} has access to document {}", user_id, document_id);
        }
        
        Ok(has_access)
    }
    
    /// Get all document IDs accessible by user
    pub async fn get_user_document_ids(&self, user_id: i32) -> Result<Vec<i32>, ApiError> {
        let document_ids = self
            .repository
            .get_user_document_ids(user_id)
            .await
            .map_err(|e| ApiError::DatabaseError(e.to_string()))?;
        
        debug!("User {} has access to {} documents", user_id, document_ids.len());
        
        Ok(document_ids)
    }
    
    /// Enforce document access (throw error if denied)
    pub async fn require_access(&self, user_id: i32, document_id: i32) -> Result<(), ApiError> {
        if !self.check_access(user_id, document_id).await? {
            return Err(ApiError::Forbidden(format!(
                "Access denied to document {}",
                document_id
            )));
        }
        
        Ok(())
    }
}

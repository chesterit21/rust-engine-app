// ============================================================================
// SSO Core - Group App Entity
// File: crates/sso-core/src/domain/group_app.rs
// Description: Group entity for RBAC
// ============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Group App entity
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct GroupApp {
    pub id: Uuid,
    pub client_app_id: Uuid,
    
    #[validate(length(min = 2, max = 100, message = "Group name must be between 2 and 100 characters"))]
    pub name: String,
    
    #[validate(length(max = 1000, message = "Description too long"))]
    pub description: Option<String>,
    
    pub is_active: bool,
    
    // Audit fields
    pub created_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub modified_at: Option<DateTime<Utc>>,
    pub modified_by: Option<Uuid>,
    pub removed_at: Option<DateTime<Utc>>,
    pub removed_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub approved_by: Option<Uuid>,
}

impl GroupApp {
    pub fn new(
        client_app_id: Uuid,
        name: String,
        description: Option<String>,
        created_by: Option<Uuid>,
    ) -> Result<Self, validator::ValidationErrors> {
        let group = Self {
            id: Uuid::new_v4(),
            client_app_id,
            name: name.trim().to_string(),
            description: description.map(|d| d.trim().to_string()),
            is_active: true,
            created_at: Utc::now(),
            created_by,
            modified_at: None,
            modified_by: None,
            removed_at: None,
            removed_by: None,
            approved_at: None,
            approved_by: None,
        };

        group.validate()?;
        Ok(group)
    }

    pub fn soft_delete(&mut self, deleted_by: Uuid) {
        self.removed_at = Some(Utc::now());
        self.removed_by = Some(deleted_by);
        self.is_active = false;
    }

    pub fn is_deleted(&self) -> bool {
        self.removed_at.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_group() {
        let group = GroupApp::new(
            Uuid::new_v4(),
            "Admin Group".to_string(),
            Some("Administrator group".to_string()),
            None,
        );
        assert!(group.is_ok());
    }
}

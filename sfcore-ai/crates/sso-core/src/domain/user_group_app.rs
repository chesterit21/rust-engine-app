// ============================================================================
// SSO Core - User Group App Entity
// File: crates/sso-core/src/domain/user_group_app.rs
// Description: User-Group assignment
// ============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// User Group App entity (User-Group Assignment)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserGroupApp {
    pub id: Uuid,
    pub client_app_id: Uuid,
    pub member_user_id: Uuid,
    pub group_app_id: Uuid,
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

impl UserGroupApp {
    pub fn new(
        client_app_id: Uuid,
        member_user_id: Uuid,
        group_app_id: Uuid,
        created_by: Option<Uuid>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            client_app_id,
            member_user_id,
            group_app_id,
            is_active: true,
            created_at: Utc::now(),
            created_by,
            modified_at: None,
            modified_by: None,
            removed_at: None,
            removed_by: None,
            approved_at: None,
            approved_by: None,
        }
    }

    pub fn deactivate(&mut self, deactivated_by: Uuid) {
        self.is_active = false;
        self.modified_at = Some(Utc::now());
        self.modified_by = Some(deactivated_by);
    }

    pub fn activate(&mut self, activated_by: Uuid) {
        self.is_active = true;
        self.modified_at = Some(Utc::now());
        self.modified_by = Some(activated_by);
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
    fn test_create_user_group() {
        let ug = UserGroupApp::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
        );
        assert!(ug.is_active);
        assert!(!ug.is_deleted());
    }

    #[test]
    fn test_deactivate_user_group() {
        let mut ug = UserGroupApp::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
        );
        ug.deactivate(Uuid::new_v4());
        assert!(!ug.is_active);
    }
}

// ============================================================================
// SSO Core - Group Menu App Entity (Permission Matrix)
// File: crates/sso-core/src/domain/group_menu_app.rs
// Description: Permission matrix linking groups to menus
// ============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Menu permission flags
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MenuPermissions {
    pub is_view: bool,
    pub is_add: bool,
    pub is_edit: bool,
    pub is_delete: bool,
    pub is_approve: bool,
    pub is_download: bool,
    pub is_upload: bool,
    pub is_print: bool,
}

impl MenuPermissions {
    pub fn full_access() -> Self {
        Self {
            is_view: true,
            is_add: true,
            is_edit: true,
            is_delete: true,
            is_approve: true,
            is_download: true,
            is_upload: true,
            is_print: true,
        }
    }

    pub fn read_only() -> Self {
        Self {
            is_view: true,
            is_add: false,
            is_edit: false,
            is_delete: false,
            is_approve: false,
            is_download: true,
            is_upload: false,
            is_print: true,
        }
    }

    pub fn no_access() -> Self {
        Self::default()
    }
}

/// Group Menu App entity (Permission Matrix)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMenuApp {
    pub id: Uuid,
    pub client_app_id: Uuid,
    pub group_app_id: Uuid,
    pub menu_app_id: Uuid,
    
    // Permission flags
    pub is_view: bool,
    pub is_add: bool,
    pub is_edit: bool,
    pub is_delete: bool,
    pub is_approve: bool,
    pub is_download: bool,
    pub is_upload: bool,
    pub is_print: bool,
    
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

impl GroupMenuApp {
    pub fn new(
        client_app_id: Uuid,
        group_app_id: Uuid,
        menu_app_id: Uuid,
        permissions: MenuPermissions,
        created_by: Option<Uuid>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            client_app_id,
            group_app_id,
            menu_app_id,
            is_view: permissions.is_view,
            is_add: permissions.is_add,
            is_edit: permissions.is_edit,
            is_delete: permissions.is_delete,
            is_approve: permissions.is_approve,
            is_download: permissions.is_download,
            is_upload: permissions.is_upload,
            is_print: permissions.is_print,
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

    pub fn has_any_permission(&self) -> bool {
        self.is_view || self.is_add || self.is_edit || self.is_delete ||
        self.is_approve || self.is_download || self.is_upload || self.is_print
    }

    pub fn update_permissions(&mut self, permissions: MenuPermissions, modified_by: Uuid) {
        self.is_view = permissions.is_view;
        self.is_add = permissions.is_add;
        self.is_edit = permissions.is_edit;
        self.is_delete = permissions.is_delete;
        self.is_approve = permissions.is_approve;
        self.is_download = permissions.is_download;
        self.is_upload = permissions.is_upload;
        self.is_print = permissions.is_print;
        self.modified_at = Some(Utc::now());
        self.modified_by = Some(modified_by);
    }

    pub fn to_permissions(&self) -> MenuPermissions {
        MenuPermissions {
            is_view: self.is_view,
            is_add: self.is_add,
            is_edit: self.is_edit,
            is_delete: self.is_delete,
            is_approve: self.is_approve,
            is_download: self.is_download,
            is_upload: self.is_upload,
            is_print: self.is_print,
        }
    }

    pub fn soft_delete(&mut self, deleted_by: Uuid) {
        self.removed_at = Some(Utc::now());
        self.removed_by = Some(deleted_by);
    }

    pub fn is_deleted(&self) -> bool {
        self.removed_at.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_access_permissions() {
        let perms = MenuPermissions::full_access();
        assert!(perms.is_view);
        assert!(perms.is_add);
        assert!(perms.is_edit);
        assert!(perms.is_delete);
    }

    #[test]
    fn test_read_only_permissions() {
        let perms = MenuPermissions::read_only();
        assert!(perms.is_view);
        assert!(!perms.is_add);
        assert!(!perms.is_edit);
        assert!(!perms.is_delete);
    }

    #[test]
    fn test_create_group_menu() {
        let gm = GroupMenuApp::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            MenuPermissions::full_access(),
            None,
        );
        assert!(gm.has_any_permission());
    }
}

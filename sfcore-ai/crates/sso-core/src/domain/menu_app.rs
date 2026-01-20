// ============================================================================
// SSO Core - Menu App Entity
// File: crates/sso-core/src/domain/menu_app.rs
// Description: Menu hierarchy entity
// ============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Menu App entity
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct MenuApp {
    pub id: Uuid,
    pub client_app_id: Uuid,
    
    #[validate(length(min = 2, max = 100, message = "Menu name must be between 2 and 100 characters"))]
    pub menu_name: String,
    
    #[validate(length(min = 1, max = 255, message = "Menu URL must be between 1 and 255 characters"))]
    pub menu_url: String,
    
    pub parent_menu_id: Option<Uuid>,
    
    #[validate(length(max = 100, message = "Menu icon too long"))]
    pub menu_icon: Option<String>,
    
    #[validate(range(min = 1, max = 5, message = "Menu level must be between 1 and 5"))]
    pub menu_level: i32,
    
    pub menu_order: i32,
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

impl MenuApp {
    pub fn new(
        client_app_id: Uuid,
        menu_name: String,
        menu_url: String,
        parent_menu_id: Option<Uuid>,
        menu_icon: Option<String>,
        menu_level: i32,
        menu_order: i32,
        created_by: Option<Uuid>,
    ) -> Result<Self, validator::ValidationErrors> {
        let menu = Self {
            id: Uuid::new_v4(),
            client_app_id,
            menu_name: menu_name.trim().to_string(),
            menu_url: menu_url.trim().to_string(),
            parent_menu_id,
            menu_icon: menu_icon.map(|i| i.trim().to_string()),
            menu_level,
            menu_order,
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

        menu.validate()?;
        Ok(menu)
    }

    pub fn is_root_menu(&self) -> bool {
        self.parent_menu_id.is_none() && self.menu_level == 1
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
    fn test_create_menu() {
        let menu = MenuApp::new(
            Uuid::new_v4(),
            "Dashboard".to_string(),
            "/dashboard".to_string(),
            None,
            Some("home".to_string()),
            1,
            1,
            None,
        );
        assert!(menu.is_ok());
        assert!(menu.unwrap().is_root_menu());
    }
}

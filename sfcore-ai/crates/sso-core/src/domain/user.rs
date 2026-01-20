//! User domain entity

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct User {
    pub id: Uuid,
    pub tenant_id: Uuid,
    
    #[validate(email)]
    pub email: String,
    pub password_hash: String,
    
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub display_name: Option<String>,
    
    pub is_active: bool,
    pub is_verified: bool,
    pub is_locked: bool,
    
    pub last_login_at: Option<DateTime<Utc>>,
    pub failed_login_attempts: i32,
    
    // Audit fields
    pub created_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub modified_at: Option<DateTime<Utc>>,
    pub modified_by: Option<Uuid>,
    pub removed_at: Option<DateTime<Utc>>,
    pub removed_by: Option<Uuid>,
}

impl User {
    pub fn is_deleted(&self) -> bool {
        self.removed_at.is_some()
    }
    
    pub fn full_name(&self) -> String {
        match (&self.first_name, &self.last_name) {
            (Some(f), Some(l)) => format!("{} {}", f, l),
            (Some(f), None) => f.clone(),
            (None, Some(l)) => l.clone(),
            _ => self.email.clone(),
        }
    }
}

//! Tenant domain entity

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
    
    // Audit
    pub created_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub modified_at: Option<DateTime<Utc>>,
    pub removed_at: Option<DateTime<Utc>>,
}

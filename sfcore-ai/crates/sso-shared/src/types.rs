//! Common types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type EntityId = Uuid;

pub fn new_id() -> EntityId {
    Uuid::new_v4()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    pub page: u32,
    pub per_page: u32,
}

impl Default for Pagination {
    fn default() -> Self {
        Self { page: 1, per_page: super::constants::DEFAULT_PAGE_SIZE }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditFields {
    pub created_at: DateTime<Utc>,
    pub created_by: Option<EntityId>,
    pub modified_at: Option<DateTime<Utc>>,
    pub modified_by: Option<EntityId>,
    pub removed_at: Option<DateTime<Utc>>,
    pub removed_by: Option<EntityId>,
}

impl Default for AuditFields {
    fn default() -> Self {
        Self {
            created_at: Utc::now(),
            created_by: None,
            modified_at: None,
            modified_by: None,
            removed_at: None,
            removed_by: None,
        }
    }
}

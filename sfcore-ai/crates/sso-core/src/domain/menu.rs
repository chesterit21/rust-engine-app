//! Menu entity (placeholder)

use uuid::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Menu {
    pub id: Uuid,
    pub client_app_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub route: Option<String>,
    pub icon: Option<String>,
    pub sort_order: i32,
    pub is_active: bool,
}

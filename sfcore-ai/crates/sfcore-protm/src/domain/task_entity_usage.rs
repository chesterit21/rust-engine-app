use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::NaiveDateTime;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TaskEntityUsage {
    pub id: String,
    pub task_id: String,
    pub entity_id: String,
    pub operation: String,
    pub attributes_used: Option<String>,
    pub created_at: Option<NaiveDateTime>,
}

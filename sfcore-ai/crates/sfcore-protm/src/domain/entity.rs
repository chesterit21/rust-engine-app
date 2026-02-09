use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::NaiveDateTime;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Entity {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub table_name: String,
    pub description: Option<String>,
    pub is_aggregate_root: bool,
    pub created_at: Option<NaiveDateTime>,
}

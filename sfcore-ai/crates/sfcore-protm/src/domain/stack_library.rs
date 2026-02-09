use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::NaiveDateTime;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct StackLibrary {
    pub id: String,
    pub stack_id: String,
    pub name: String,
    pub npm_package: Option<String>,
    pub version: Option<String>,
    pub category: Option<String>,
    pub description: Option<String>,
    pub is_required: bool,
    pub created_at: Option<NaiveDateTime>,
}

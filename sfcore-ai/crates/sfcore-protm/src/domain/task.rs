use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::NaiveDateTime;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Task {
    pub id: String,
    pub use_case_id: String,
    pub name: String,
    pub priority: String,
    pub status: String,
    pub description: Option<String>,
    pub validation_rules: Option<String>,
    pub order_index: Option<i32>,
    pub created_at: Option<NaiveDateTime>,
    pub completed_at: Option<NaiveDateTime>,
}

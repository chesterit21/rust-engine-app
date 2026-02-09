use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::NaiveDateTime;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FlowStep {
    pub id: String,
    pub task_id: String,
    pub order_index: i32,
    #[sqlx(rename = "type")]
    pub step_type: String,
    pub description: String,
    pub code_snippet: Option<String>,
    pub validation_rules: Option<String>,
    pub created_at: Option<NaiveDateTime>,
}

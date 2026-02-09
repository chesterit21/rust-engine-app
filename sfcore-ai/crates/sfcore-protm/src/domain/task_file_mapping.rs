use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::NaiveDateTime;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TaskFileMapping {
    pub id: String,
    pub task_id: String,
    pub template_id: String,
    pub file_path: String,
    pub class_name: String,
    pub method_names: Option<String>,
    pub dependencies: Option<String>,
    pub implementation_notes: Option<String>,
    pub created_at: Option<NaiveDateTime>,
}

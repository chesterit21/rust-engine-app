use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::NaiveDateTime;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FileTemplate {
    pub id: String,
    pub layer_id: String,
    pub name: String,
    pub file_naming: String,
    pub class_naming: String,
    pub code_template: Option<String>,
    pub required_imports: Option<String>,
    pub required_methods: Option<String>,
    pub description: Option<String>,
    pub created_at: Option<NaiveDateTime>,
}

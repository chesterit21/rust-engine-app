use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TechStack {
    pub id: String,
    pub name: String,
    #[sqlx(rename = "type")]
    pub stack_type: String,
    pub language: String,
    pub description: Option<String>,
}

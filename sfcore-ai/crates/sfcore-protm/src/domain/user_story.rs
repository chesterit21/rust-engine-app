use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::NaiveDateTime;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserStory {
    pub id: String,
    pub module_id: String,
    pub name: String,
    pub description: Option<String>,
    pub order_index: Option<i32>,
    pub created_at: Option<NaiveDateTime>,
}

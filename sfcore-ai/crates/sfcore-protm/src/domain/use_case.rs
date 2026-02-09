use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::NaiveDateTime;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UseCase {
    pub id: String,
    pub user_story_id: String,
    pub name: String,
    pub actor: Option<String>,
    pub goal: Option<String>,
    pub success_criteria: Option<String>,
    pub order_index: Option<i32>,
    pub created_at: Option<NaiveDateTime>,
}

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::NaiveDateTime;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PatternLayer {
    pub id: String,
    pub pattern_id: String,
    pub name: String,
    pub path: String,
    pub rules: Option<String>,
    pub order_index: i32,
    pub created_at: Option<NaiveDateTime>,
}

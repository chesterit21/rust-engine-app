use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::NaiveDateTime;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Attribute {
    pub id: String,
    pub entity_id: String,
    pub name: String,
    pub data_type: String,
    pub is_primary_key: bool,
    pub is_foreign_key: bool,
    pub is_nullable: bool,
    pub is_unique: bool,
    pub max_length: Option<i32>,
    pub validation_rules: Option<String>,
    pub business_rules: Option<String>,
    pub source_description: Option<String>,
    pub order_index: Option<i32>,
    pub created_at: Option<NaiveDateTime>,
}

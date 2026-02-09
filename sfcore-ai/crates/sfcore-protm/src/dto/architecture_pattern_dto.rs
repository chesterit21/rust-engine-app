use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Deserialize, Clone)]
pub struct CreateArchitecturePatternDto {
    pub id: Option<String>,
    pub parent_id: Option<String>,
    pub stack_id: String,
    pub name: String,
    pub version: String, // 'LITE', 'STANDAR', 'PRODUCTION GRADE'
    #[serde(rename = "type")]
    pub pattern_type: String, // 'BE', 'FE', 'FULLSTACK'
    pub layer_rules: Option<String>,
    pub order_index: i32,
    pub naming_conventions: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateArchitecturePatternDto {
    pub parent_id: Option<String>,
    pub stack_id: String,
    pub name: String,
    pub version: String,
    #[serde(rename = "type")]
    pub pattern_type: String,
    pub layer_rules: Option<String>,
    pub order_index: i32,
    pub naming_conventions: Option<String>,
}

#[derive(Serialize, FromRow)]
pub struct ArchitecturePatternGroup {
    pub stack_id: String,
    pub stack_name: String,
    pub stack_type: String,
    pub version: String,
    pub item_count: i32,
}

#[derive(Deserialize)]
pub struct FilterParams {
    pub stack_id: Option<String>,
    pub version: Option<String>,
}

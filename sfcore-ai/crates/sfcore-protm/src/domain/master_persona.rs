use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::NaiveDateTime;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MasterPersona {
    pub id: String,
    pub name: String,
    pub description: String,
    pub created_at: Option<NaiveDateTime>,
}

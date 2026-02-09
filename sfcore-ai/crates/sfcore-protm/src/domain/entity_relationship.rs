use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::NaiveDateTime;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EntityRelationship {
    pub id: String,
    pub entity_id: String,
    pub related_entity_id: String,
    pub relationship_type: String,
    pub foreign_key_attribute_id: Option<String>,
    #[sqlx(rename = "fk_description")] 
    pub fk_description: Option<String>, 
    pub created_at: Option<NaiveDateTime>,
}

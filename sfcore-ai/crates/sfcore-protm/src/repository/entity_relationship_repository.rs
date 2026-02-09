use crate::domain::EntityRelationship;
use sqlx::SqlitePool;

pub struct EntityRelationshipRepository;

impl EntityRelationshipRepository {
    pub async fn find_all(pool: &SqlitePool) -> Result<Vec<EntityRelationship>, sqlx::Error> {
        sqlx::query_as::<_, EntityRelationship>(
            "SELECT id, entity_id, related_entity_id, relationship_type, foreign_key_attribute_id, fk_description, created_at FROM entity_relationships"
        )
        .fetch_all(pool)
        .await
    }
    
    pub async fn delete(pool: &SqlitePool, id: &str) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM entity_relationships WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }

    pub async fn find_by_id(pool: &SqlitePool, id: &str) -> Result<Option<EntityRelationship>, sqlx::Error> {
        sqlx::query_as::<_, EntityRelationship>(
            "SELECT id, entity_id, related_entity_id, relationship_type, foreign_key_attribute_id, fk_description, created_at FROM entity_relationships WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }
}

use crate::domain::Attribute;
use sqlx::SqlitePool;

pub struct AttributeRepository;

impl AttributeRepository {
    pub async fn find_all(pool: &SqlitePool) -> Result<Vec<Attribute>, sqlx::Error> {
        sqlx::query_as::<_, Attribute>(
            "SELECT id, entity_id, name, data_type, is_primary_key, is_foreign_key, is_nullable, is_unique, max_length, validation_rules, business_rules, source_description, order_index, created_at FROM attributes"
        )
        .fetch_all(pool)
        .await
    }
    
    pub async fn delete(pool: &SqlitePool, id: &str) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM attributes WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }

    pub async fn find_by_id(pool: &SqlitePool, id: &str) -> Result<Option<Attribute>, sqlx::Error> {
        sqlx::query_as::<_, Attribute>(
            "SELECT id, entity_id, name, data_type, is_primary_key, is_foreign_key, is_nullable, is_unique, max_length, validation_rules, business_rules, source_description, order_index, created_at FROM attributes WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }
}

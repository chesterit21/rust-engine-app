use crate::domain::Entity;
use sqlx::SqlitePool;

pub struct EntityRepository;

impl EntityRepository {
    pub async fn find_all(pool: &SqlitePool) -> Result<Vec<Entity>, sqlx::Error> {
        sqlx::query_as::<_, Entity>(
            "SELECT id, project_id, name, table_name, description, is_aggregate_root, created_at FROM entities"
        )
        .fetch_all(pool)
        .await
    }
    
    pub async fn delete(pool: &SqlitePool, id: &str) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM entities WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }

    pub async fn find_by_id(pool: &SqlitePool, id: &str) -> Result<Option<Entity>, sqlx::Error> {
        sqlx::query_as::<_, Entity>(
            "SELECT id, project_id, name, table_name, description, is_aggregate_root, created_at FROM entities WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }
}

use crate::domain::TaskEntityUsage;
use sqlx::SqlitePool;

pub struct TaskEntityUsageRepository;

impl TaskEntityUsageRepository {
    pub async fn find_all(pool: &SqlitePool) -> Result<Vec<TaskEntityUsage>, sqlx::Error> {
        sqlx::query_as::<_, TaskEntityUsage>(
            "SELECT id, task_id, entity_id, operation, attributes_used, created_at FROM task_entity_usage"
        )
        .fetch_all(pool)
        .await
    }
    
    pub async fn delete(pool: &SqlitePool, id: &str) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM task_entity_usage WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }

    pub async fn find_by_id(pool: &SqlitePool, id: &str) -> Result<Option<TaskEntityUsage>, sqlx::Error> {
        sqlx::query_as::<_, TaskEntityUsage>(
            "SELECT id, task_id, entity_id, operation, attributes_used, created_at FROM task_entity_usage WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }
}

use crate::domain::Task;
use sqlx::SqlitePool;

pub struct TaskRepository;

impl TaskRepository {
    pub async fn find_all(pool: &SqlitePool) -> Result<Vec<Task>, sqlx::Error> {
        sqlx::query_as::<_, Task>(
            "SELECT id, use_case_id, name, priority, status, description, validation_rules, order_index, created_at, completed_at FROM tasks"
        )
        .fetch_all(pool)
        .await
    }
    
    pub async fn delete(pool: &SqlitePool, id: &str) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM tasks WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }

    pub async fn find_by_id(pool: &SqlitePool, id: &str) -> Result<Option<Task>, sqlx::Error> {
        sqlx::query_as::<_, Task>(
            "SELECT id, use_case_id, name, priority, status, description, validation_rules, order_index, created_at, completed_at FROM tasks WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }
}

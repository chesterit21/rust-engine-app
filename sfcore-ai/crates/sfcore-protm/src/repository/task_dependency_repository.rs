use crate::domain::TaskDependency;
use sqlx::SqlitePool;

pub struct TaskDependencyRepository;

impl TaskDependencyRepository {
    pub async fn find_all(pool: &SqlitePool) -> Result<Vec<TaskDependency>, sqlx::Error> {
        sqlx::query_as::<_, TaskDependency>(
            "SELECT id, task_id, depends_on_task_id, dependency_type, created_at FROM task_dependencies"
        )
        .fetch_all(pool)
        .await
    }
    
    pub async fn delete(pool: &SqlitePool, id: &str) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM task_dependencies WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }

    pub async fn find_by_id(pool: &SqlitePool, id: &str) -> Result<Option<TaskDependency>, sqlx::Error> {
        sqlx::query_as::<_, TaskDependency>(
            "SELECT id, task_id, depends_on_task_id, dependency_type, created_at FROM task_dependencies WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }
}

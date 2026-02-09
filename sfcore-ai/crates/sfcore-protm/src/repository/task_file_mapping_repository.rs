use crate::domain::TaskFileMapping;
use sqlx::SqlitePool;

pub struct TaskFileMappingRepository;

impl TaskFileMappingRepository {
    pub async fn find_all(pool: &SqlitePool) -> Result<Vec<TaskFileMapping>, sqlx::Error> {
        sqlx::query_as::<_, TaskFileMapping>(
            "SELECT id, task_id, template_id, file_path, class_name, method_names, dependencies, implementation_notes, created_at FROM task_file_mappings"
        )
        .fetch_all(pool)
        .await
    }
    
    pub async fn delete(pool: &SqlitePool, id: &str) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM task_file_mappings WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }

    pub async fn find_by_id(pool: &SqlitePool, id: &str) -> Result<Option<TaskFileMapping>, sqlx::Error> {
        sqlx::query_as::<_, TaskFileMapping>(
            "SELECT id, task_id, template_id, file_path, class_name, method_names, dependencies, implementation_notes, created_at FROM task_file_mappings WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }
}

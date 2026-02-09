use crate::domain::StackLibrary;
use sqlx::SqlitePool;

pub struct StackLibraryRepository;

impl StackLibraryRepository {
    pub async fn find_all(pool: &SqlitePool) -> Result<Vec<StackLibrary>, sqlx::Error> {
        sqlx::query_as::<_, StackLibrary>(
            "SELECT id, stack_id, name, npm_package, version, category, description, is_required, created_at FROM stack_libraries"
        )
        .fetch_all(pool)
        .await
    }
    
    pub async fn delete(pool: &SqlitePool, id: &str) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM stack_libraries WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }

    pub async fn find_by_id(pool: &SqlitePool, id: &str) -> Result<Option<StackLibrary>, sqlx::Error> {
        sqlx::query_as::<_, StackLibrary>(
            "SELECT id, stack_id, name, npm_package, version, category, description, is_required, created_at FROM stack_libraries WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }
}

use crate::domain::Module;
use sqlx::SqlitePool;

pub struct ModuleRepository;

impl ModuleRepository {
    pub async fn find_all(pool: &SqlitePool) -> Result<Vec<Module>, sqlx::Error> {
        sqlx::query_as::<_, Module>(
            "SELECT id, project_id, name, description, order_index, created_at FROM modules"
        )
        .fetch_all(pool)
        .await
    }
    
    pub async fn delete(pool: &SqlitePool, id: &str) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM modules WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }

    pub async fn find_by_id(pool: &SqlitePool, id: &str) -> Result<Option<Module>, sqlx::Error> {
        sqlx::query_as::<_, Module>(
            "SELECT id, project_id, name, description, order_index, created_at FROM modules WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }
}

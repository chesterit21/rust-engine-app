use crate::domain::UserStory;
use sqlx::SqlitePool;

pub struct UserStoryRepository;

impl UserStoryRepository {
    pub async fn find_all(pool: &SqlitePool) -> Result<Vec<UserStory>, sqlx::Error> {
        sqlx::query_as::<_, UserStory>(
            "SELECT id, module_id, name, description, order_index, created_at FROM user_storys"
        )
        .fetch_all(pool)
        .await
    }
    
    pub async fn delete(pool: &SqlitePool, id: &str) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM user_storys WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }

    pub async fn find_by_id(pool: &SqlitePool, id: &str) -> Result<Option<UserStory>, sqlx::Error> {
        sqlx::query_as::<_, UserStory>(
            "SELECT id, module_id, name, description, order_index, created_at FROM user_storys WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }
}

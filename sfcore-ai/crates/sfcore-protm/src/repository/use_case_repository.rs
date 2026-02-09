use crate::domain::UseCase;
use sqlx::SqlitePool;

pub struct UseCaseRepository;

impl UseCaseRepository {
    pub async fn find_all(pool: &SqlitePool) -> Result<Vec<UseCase>, sqlx::Error> {
        sqlx::query_as::<_, UseCase>(
            "SELECT id, user_story_id, name, actor, goal, success_criteria, order_index, created_at FROM use_cases"
        )
        .fetch_all(pool)
        .await
    }
    
    pub async fn delete(pool: &SqlitePool, id: &str) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM use_cases WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }

    pub async fn find_by_id(pool: &SqlitePool, id: &str) -> Result<Option<UseCase>, sqlx::Error> {
        sqlx::query_as::<_, UseCase>(
            "SELECT id, user_story_id, name, actor, goal, success_criteria, order_index, created_at FROM use_cases WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }
}

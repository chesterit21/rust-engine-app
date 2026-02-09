use crate::domain::PatternLayer;
use sqlx::SqlitePool;

pub struct PatternLayerRepository;

impl PatternLayerRepository {
    pub async fn find_all(pool: &SqlitePool) -> Result<Vec<PatternLayer>, sqlx::Error> {
        sqlx::query_as::<_, PatternLayer>(
            "SELECT id, pattern_id, name, path, rules, order_index, created_at FROM pattern_layers"
        )
        .fetch_all(pool)
        .await
    }
    
    pub async fn delete(pool: &SqlitePool, id: &str) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM pattern_layers WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }

    pub async fn find_by_id(pool: &SqlitePool, id: &str) -> Result<Option<PatternLayer>, sqlx::Error> {
        sqlx::query_as::<_, PatternLayer>(
            "SELECT id, pattern_id, name, path, rules, order_index, created_at FROM pattern_layers WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }
}

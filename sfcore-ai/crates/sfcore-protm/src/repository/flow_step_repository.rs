use crate::domain::FlowStep;
use sqlx::SqlitePool;

pub struct FlowStepRepository;

impl FlowStepRepository {
    pub async fn find_all(pool: &SqlitePool) -> Result<Vec<FlowStep>, sqlx::Error> {
        sqlx::query_as::<_, FlowStep>(
            "SELECT id, task_id, order_index, type, description, code_snippet, validation_rules, created_at FROM flow_steps"
        )
        .fetch_all(pool)
        .await
    }
    
    pub async fn delete(pool: &SqlitePool, id: &str) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM flow_steps WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }

    pub async fn find_by_id(pool: &SqlitePool, id: &str) -> Result<Option<FlowStep>, sqlx::Error> {
        sqlx::query_as::<_, FlowStep>(
            "SELECT id, task_id, order_index, type, description, code_snippet, validation_rules, created_at FROM flow_steps WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }
}

use crate::domain::Persona;
use sqlx::SqlitePool;

pub struct PersonaRepository;

impl PersonaRepository {
    pub async fn find_all(pool: &SqlitePool) -> Result<Vec<Persona>, sqlx::Error> {
        sqlx::query_as::<_, Persona>(
            "SELECT id, project_id, name, description, created_at FROM personas"
        )
        .fetch_all(pool)
        .await
    }
    
    pub async fn delete(pool: &SqlitePool, id: &str) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM personas WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }

    pub async fn find_by_id(pool: &SqlitePool, id: &str) -> Result<Option<Persona>, sqlx::Error> {
        sqlx::query_as::<_, Persona>(
            "SELECT id, project_id, name, description, created_at FROM personas WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }
}

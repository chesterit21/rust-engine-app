use crate::domain::MasterPersona;
use sqlx::SqlitePool;

pub struct MasterPersonaRepository;

impl MasterPersonaRepository {
    pub async fn find_all(pool: &SqlitePool) -> Result<Vec<MasterPersona>, sqlx::Error> {
        sqlx::query_as::<_, MasterPersona>(
            "SELECT id, name, description, created_at FROM master_personas"
        )
        .fetch_all(pool)
        .await
    }
    
    pub async fn delete(pool: &SqlitePool, id: &str) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM master_personas WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }

    pub async fn find_by_id(pool: &SqlitePool, id: &str) -> Result<Option<MasterPersona>, sqlx::Error> {
        sqlx::query_as::<_, MasterPersona>(
            "SELECT id, name, description, created_at FROM master_personas WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn create(pool: &SqlitePool, name: &str, description: &str) -> Result<MasterPersona, sqlx::Error> {
        let id = uuid::Uuid::new_v4().to_string();
        sqlx::query_as::<sqlx::Sqlite, MasterPersona>(
            "INSERT INTO master_personas (id, name, description) VALUES (?, ?, ?) RETURNING id, name, description, created_at"
        )
        .bind(id)
        .bind(name)
        .bind(description)
        .fetch_one(pool)
        .await
    }

    pub async fn update(pool: &SqlitePool, id: &str, name: &str, description: &str) -> Result<MasterPersona, sqlx::Error> {
        sqlx::query_as::<sqlx::Sqlite, MasterPersona>(
            "UPDATE master_personas SET name = ?, description = ? WHERE id = ? RETURNING id, name, description, created_at"
        )
        .bind(name)
        .bind(description)
        .bind(id)
        .fetch_one(pool)
        .await
    }
}

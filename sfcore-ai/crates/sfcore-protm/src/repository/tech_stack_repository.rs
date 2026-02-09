use crate::domain::TechStack;
use sqlx::SqlitePool;

pub struct TechStackRepository;

impl TechStackRepository {
    pub async fn find_all(pool: &SqlitePool) -> Result<Vec<TechStack>, sqlx::Error> {
        sqlx::query_as::<_, TechStack>(
            "SELECT id, name, type, language, description FROM tech_stacks"
        )
        .fetch_all(pool)
        .await
    }
    
    pub async fn delete(pool: &SqlitePool, id: &str) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM tech_stacks WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }

    pub async fn find_by_id(pool: &SqlitePool, id: &str) -> Result<Option<TechStack>, sqlx::Error> {
        sqlx::query_as::<_, TechStack>(
            "SELECT id, name, type, language, description FROM tech_stacks WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn create(pool: &SqlitePool, name: &str, stack_type: &str, language: &str, description: Option<&str>) -> Result<TechStack, sqlx::Error> {
         let id = uuid::Uuid::new_v4().to_string();
         sqlx::query_as::<sqlx::Sqlite, TechStack>(
             "INSERT INTO tech_stacks (id, name, type, language, description) VALUES (?, ?, ?, ?, ?) RETURNING id, name, type, language, description"
         )
         .bind(id)
         .bind(name)
         .bind(stack_type)
         .bind(language)
         .bind(description)
         .fetch_one(pool)
         .await
    }

    pub async fn update(pool: &SqlitePool, id: &str, name: &str, stack_type: &str, language: &str, description: Option<&str>) -> Result<TechStack, sqlx::Error> {
        sqlx::query_as::<sqlx::Sqlite, TechStack>(
            "UPDATE tech_stacks SET name = ?, type = ?, language = ?, description = ? WHERE id = ? RETURNING id, name, type, language, description"
        )
        .bind(name)
        .bind(stack_type)
        .bind(language)
        .bind(description)
        .bind(id)
        .fetch_one(pool)
        .await
    }

    pub async fn find_by_name(pool: &SqlitePool, name: &str) -> Result<Option<TechStack>, sqlx::Error> {
        sqlx::query_as::<sqlx::Sqlite, TechStack>(
            "SELECT id, name, type, language, description FROM tech_stacks WHERE name = ?"
        )
        .bind(name)
        .fetch_optional(pool)
        .await
    }
}

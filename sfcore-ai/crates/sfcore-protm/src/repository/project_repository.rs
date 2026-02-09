use crate::domain::Project;
use sqlx::SqlitePool;

pub struct ProjectRepository;

impl ProjectRepository {
    pub async fn find_all(pool: &SqlitePool) -> Result<Vec<Project>, sqlx::Error> {
        sqlx::query_as::<_, Project>(
            "SELECT id, name, description, created_at, updated_at FROM projects"
        )
        .fetch_all(pool)
        .await
    }
    
    pub async fn delete(pool: &SqlitePool, id: &str) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM projects WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }

    pub async fn find_by_id(pool: &SqlitePool, id: &str) -> Result<Option<Project>, sqlx::Error> {
        sqlx::query_as::<_, Project>(
            "SELECT id, name, description, created_at, updated_at FROM projects WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn create(pool: &SqlitePool, name: &str, description: Option<&str>) -> Result<Project, sqlx::Error> {
        let id = uuid::Uuid::new_v4().to_string();
        sqlx::query_as::<sqlx::Sqlite, Project>(
            "INSERT INTO projects (id, name, description) VALUES (?, ?, ?) RETURNING id, name, description, created_at, updated_at"
        )
        .bind(id)
        .bind(name)
        .bind(description)
        .fetch_one(pool)
        .await
    }

    pub async fn update(pool: &SqlitePool, id: &str, name: &str, description: Option<&str>) -> Result<Project, sqlx::Error> {
        sqlx::query_as::<sqlx::Sqlite, Project>(
            "UPDATE projects SET name = ?, description = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ? RETURNING id, name, description, created_at, updated_at"
        )
        .bind(name)
        .bind(description)
        .bind(id)
        .fetch_one(pool)
        .await
    }
}

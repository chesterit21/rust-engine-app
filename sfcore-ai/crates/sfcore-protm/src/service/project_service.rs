use crate::repository::ProjectRepository;
use crate::domain::Project;
use sqlx::SqlitePool;

pub struct ProjectService;

impl ProjectService {
    pub async fn get_all(pool: &SqlitePool) -> Result<Vec<Project>, sqlx::Error> {
        ProjectRepository::find_all(pool).await
    }

    pub async fn delete(pool: &SqlitePool, id: &str) -> Result<u64, sqlx::Error> {
        ProjectRepository::delete(pool, id).await
    }

    pub async fn find_by_id(pool: &SqlitePool, id: &str) -> Result<Option<Project>, sqlx::Error> {
        ProjectRepository::find_by_id(pool, id).await
    }

    pub async fn create(pool: &SqlitePool, name: String, description: Option<String>) -> Result<Project, sqlx::Error> {
        ProjectRepository::create(pool, &name, description.as_deref()).await
    }

    pub async fn update(pool: &SqlitePool, id: String, name: String, description: Option<String>) -> Result<Project, sqlx::Error> {
        ProjectRepository::update(pool, &id, &name, description.as_deref()).await
    }
}

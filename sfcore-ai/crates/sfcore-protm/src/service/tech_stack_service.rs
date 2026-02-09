use crate::repository::TechStackRepository;
use crate::domain::TechStack;
use sqlx::SqlitePool;

pub struct TechStackService;

impl TechStackService {
    pub async fn get_all(pool: &SqlitePool) -> Result<Vec<TechStack>, sqlx::Error> {
        TechStackRepository::find_all(pool).await
    }

    pub async fn delete(pool: &SqlitePool, id: &str) -> Result<u64, sqlx::Error> {
        TechStackRepository::delete(pool, id).await
    }

    pub async fn find_by_id(pool: &SqlitePool, id: &str) -> Result<Option<TechStack>, sqlx::Error> {
        TechStackRepository::find_by_id(pool, id).await
    }

    pub async fn create(pool: &SqlitePool, name: String, stack_type: String, language: String, description: Option<String>) -> Result<TechStack, sqlx::Error> {
        // Check if exists
        if let Some(existing) = TechStackRepository::find_by_name(pool, &name).await? {
            // Update
            TechStackRepository::update(pool, &existing.id, &name, &stack_type, &language, description.as_deref()).await
        } else {
            // Create
            TechStackRepository::create(pool, &name, &stack_type, &language, description.as_deref()).await
        }
    }

    pub async fn update(pool: &SqlitePool, id: String, name: String, stack_type: String, language: String, description: Option<String>) -> Result<TechStack, sqlx::Error> {
        TechStackRepository::update(pool, &id, &name, &stack_type, &language, description.as_deref()).await
    }
}

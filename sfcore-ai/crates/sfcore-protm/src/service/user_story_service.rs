use crate::repository::UserStoryRepository;
use crate::domain::UserStory;
use sqlx::SqlitePool;

pub struct UserStoryService;

impl UserStoryService {
    pub async fn get_all(pool: &SqlitePool) -> Result<Vec<UserStory>, sqlx::Error> {
        UserStoryRepository::find_all(pool).await
    }

    pub async fn delete(pool: &SqlitePool, id: &str) -> Result<u64, sqlx::Error> {
        UserStoryRepository::delete(pool, id).await
    }

    pub async fn find_by_id(pool: &SqlitePool, id: &str) -> Result<Option<UserStory>, sqlx::Error> {
        UserStoryRepository::find_by_id(pool, id).await
    }
}

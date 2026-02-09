use crate::repository::TaskEntityUsageRepository;
use crate::domain::TaskEntityUsage;
use sqlx::SqlitePool;

pub struct TaskEntityUsageService;

impl TaskEntityUsageService {
    pub async fn get_all(pool: &SqlitePool) -> Result<Vec<TaskEntityUsage>, sqlx::Error> {
        TaskEntityUsageRepository::find_all(pool).await
    }

    pub async fn delete(pool: &SqlitePool, id: &str) -> Result<u64, sqlx::Error> {
        TaskEntityUsageRepository::delete(pool, id).await
    }

    pub async fn find_by_id(pool: &SqlitePool, id: &str) -> Result<Option<TaskEntityUsage>, sqlx::Error> {
        TaskEntityUsageRepository::find_by_id(pool, id).await
    }
}

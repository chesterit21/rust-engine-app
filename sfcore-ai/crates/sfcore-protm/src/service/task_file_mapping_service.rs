use crate::repository::TaskFileMappingRepository;
use crate::domain::TaskFileMapping;
use sqlx::SqlitePool;

pub struct TaskFileMappingService;

impl TaskFileMappingService {
    pub async fn get_all(pool: &SqlitePool) -> Result<Vec<TaskFileMapping>, sqlx::Error> {
        TaskFileMappingRepository::find_all(pool).await
    }

    pub async fn delete(pool: &SqlitePool, id: &str) -> Result<u64, sqlx::Error> {
        TaskFileMappingRepository::delete(pool, id).await
    }

    pub async fn find_by_id(pool: &SqlitePool, id: &str) -> Result<Option<TaskFileMapping>, sqlx::Error> {
        TaskFileMappingRepository::find_by_id(pool, id).await
    }
}

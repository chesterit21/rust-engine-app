use crate::repository::TaskDependencyRepository;
use crate::domain::TaskDependency;
use sqlx::SqlitePool;

pub struct TaskDependencyService;

impl TaskDependencyService {
    pub async fn get_all(pool: &SqlitePool) -> Result<Vec<TaskDependency>, sqlx::Error> {
        TaskDependencyRepository::find_all(pool).await
    }

    pub async fn delete(pool: &SqlitePool, id: &str) -> Result<u64, sqlx::Error> {
        TaskDependencyRepository::delete(pool, id).await
    }

    pub async fn find_by_id(pool: &SqlitePool, id: &str) -> Result<Option<TaskDependency>, sqlx::Error> {
        TaskDependencyRepository::find_by_id(pool, id).await
    }
}

use crate::repository::TaskRepository;
use crate::domain::Task;
use sqlx::SqlitePool;

pub struct TaskService;

impl TaskService {
    pub async fn get_all(pool: &SqlitePool) -> Result<Vec<Task>, sqlx::Error> {
        TaskRepository::find_all(pool).await
    }

    pub async fn delete(pool: &SqlitePool, id: &str) -> Result<u64, sqlx::Error> {
        TaskRepository::delete(pool, id).await
    }

    pub async fn find_by_id(pool: &SqlitePool, id: &str) -> Result<Option<Task>, sqlx::Error> {
        TaskRepository::find_by_id(pool, id).await
    }
}

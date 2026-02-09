use crate::repository::FlowStepRepository;
use crate::domain::FlowStep;
use sqlx::SqlitePool;

pub struct FlowStepService;

impl FlowStepService {
    pub async fn get_all(pool: &SqlitePool) -> Result<Vec<FlowStep>, sqlx::Error> {
        FlowStepRepository::find_all(pool).await
    }

    pub async fn delete(pool: &SqlitePool, id: &str) -> Result<u64, sqlx::Error> {
        FlowStepRepository::delete(pool, id).await
    }

    pub async fn find_by_id(pool: &SqlitePool, id: &str) -> Result<Option<FlowStep>, sqlx::Error> {
        FlowStepRepository::find_by_id(pool, id).await
    }
}

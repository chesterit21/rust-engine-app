use crate::repository::PatternLayerRepository;
use crate::domain::PatternLayer;
use sqlx::SqlitePool;

pub struct PatternLayerService;

impl PatternLayerService {
    pub async fn get_all(pool: &SqlitePool) -> Result<Vec<PatternLayer>, sqlx::Error> {
        PatternLayerRepository::find_all(pool).await
    }

    pub async fn delete(pool: &SqlitePool, id: &str) -> Result<u64, sqlx::Error> {
        PatternLayerRepository::delete(pool, id).await
    }

    pub async fn find_by_id(pool: &SqlitePool, id: &str) -> Result<Option<PatternLayer>, sqlx::Error> {
        PatternLayerRepository::find_by_id(pool, id).await
    }
}

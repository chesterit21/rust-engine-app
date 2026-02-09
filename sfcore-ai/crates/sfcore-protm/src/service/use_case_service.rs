use crate::repository::UseCaseRepository;
use crate::domain::UseCase;
use sqlx::SqlitePool;

pub struct UseCaseService;

impl UseCaseService {
    pub async fn get_all(pool: &SqlitePool) -> Result<Vec<UseCase>, sqlx::Error> {
        UseCaseRepository::find_all(pool).await
    }

    pub async fn delete(pool: &SqlitePool, id: &str) -> Result<u64, sqlx::Error> {
        UseCaseRepository::delete(pool, id).await
    }

    pub async fn find_by_id(pool: &SqlitePool, id: &str) -> Result<Option<UseCase>, sqlx::Error> {
        UseCaseRepository::find_by_id(pool, id).await
    }
}

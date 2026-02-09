use crate::repository::EntityRepository;
use crate::domain::Entity;
use sqlx::SqlitePool;

pub struct EntityService;

impl EntityService {
    pub async fn get_all(pool: &SqlitePool) -> Result<Vec<Entity>, sqlx::Error> {
        EntityRepository::find_all(pool).await
    }

    pub async fn delete(pool: &SqlitePool, id: &str) -> Result<u64, sqlx::Error> {
        EntityRepository::delete(pool, id).await
    }

    pub async fn find_by_id(pool: &SqlitePool, id: &str) -> Result<Option<Entity>, sqlx::Error> {
        EntityRepository::find_by_id(pool, id).await
    }
}

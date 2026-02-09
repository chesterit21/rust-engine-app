use crate::repository::AttributeRepository;
use crate::domain::Attribute;
use sqlx::SqlitePool;

pub struct AttributeService;

impl AttributeService {
    pub async fn get_all(pool: &SqlitePool) -> Result<Vec<Attribute>, sqlx::Error> {
        AttributeRepository::find_all(pool).await
    }

    pub async fn delete(pool: &SqlitePool, id: &str) -> Result<u64, sqlx::Error> {
        AttributeRepository::delete(pool, id).await
    }

    pub async fn find_by_id(pool: &SqlitePool, id: &str) -> Result<Option<Attribute>, sqlx::Error> {
        AttributeRepository::find_by_id(pool, id).await
    }
}

use crate::repository::EntityRelationshipRepository;
use crate::domain::EntityRelationship;
use sqlx::SqlitePool;

pub struct EntityRelationshipService;

impl EntityRelationshipService {
    pub async fn get_all(pool: &SqlitePool) -> Result<Vec<EntityRelationship>, sqlx::Error> {
        EntityRelationshipRepository::find_all(pool).await
    }

    pub async fn delete(pool: &SqlitePool, id: &str) -> Result<u64, sqlx::Error> {
        EntityRelationshipRepository::delete(pool, id).await
    }

    pub async fn find_by_id(pool: &SqlitePool, id: &str) -> Result<Option<EntityRelationship>, sqlx::Error> {
        EntityRelationshipRepository::find_by_id(pool, id).await
    }
}

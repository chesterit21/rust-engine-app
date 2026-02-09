use crate::repository::MasterPersonaRepository;
use crate::domain::MasterPersona;
use sqlx::SqlitePool;

pub struct MasterPersonaService;

impl MasterPersonaService {
    pub async fn get_all(pool: &SqlitePool) -> Result<Vec<MasterPersona>, sqlx::Error> {
        MasterPersonaRepository::find_all(pool).await
    }

    pub async fn delete(pool: &SqlitePool, id: &str) -> Result<u64, sqlx::Error> {
        MasterPersonaRepository::delete(pool, id).await
    }

    pub async fn find_by_id(pool: &SqlitePool, id: &str) -> Result<Option<MasterPersona>, sqlx::Error> {
        MasterPersonaRepository::find_by_id(pool, id).await
    }

    pub async fn create(pool: &SqlitePool, name: String, description: String) -> Result<MasterPersona, sqlx::Error> {
        MasterPersonaRepository::create(pool, &name, &description).await
    }

    pub async fn update(pool: &SqlitePool, id: String, name: String, description: String) -> Result<MasterPersona, sqlx::Error> {
        MasterPersonaRepository::update(pool, &id, &name, &description).await
    }
}

use crate::repository::PersonaRepository;
use crate::domain::Persona;
use sqlx::SqlitePool;

pub struct PersonaService;

impl PersonaService {
    pub async fn get_all(pool: &SqlitePool) -> Result<Vec<Persona>, sqlx::Error> {
        PersonaRepository::find_all(pool).await
    }

    pub async fn delete(pool: &SqlitePool, id: &str) -> Result<u64, sqlx::Error> {
        PersonaRepository::delete(pool, id).await
    }

    pub async fn find_by_id(pool: &SqlitePool, id: &str) -> Result<Option<Persona>, sqlx::Error> {
        PersonaRepository::find_by_id(pool, id).await
    }
}

use crate::repository::ModuleRepository;
use crate::domain::Module;
use sqlx::SqlitePool;

pub struct ModuleService;

impl ModuleService {
    pub async fn get_all(pool: &SqlitePool) -> Result<Vec<Module>, sqlx::Error> {
        ModuleRepository::find_all(pool).await
    }

    pub async fn delete(pool: &SqlitePool, id: &str) -> Result<u64, sqlx::Error> {
        ModuleRepository::delete(pool, id).await
    }

    pub async fn find_by_id(pool: &SqlitePool, id: &str) -> Result<Option<Module>, sqlx::Error> {
        ModuleRepository::find_by_id(pool, id).await
    }
}

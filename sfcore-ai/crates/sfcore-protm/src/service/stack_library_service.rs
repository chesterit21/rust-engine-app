use crate::repository::StackLibraryRepository;
use crate::domain::StackLibrary;
use sqlx::SqlitePool;

pub struct StackLibraryService;

impl StackLibraryService {
    pub async fn get_all(pool: &SqlitePool) -> Result<Vec<StackLibrary>, sqlx::Error> {
        StackLibraryRepository::find_all(pool).await
    }

    pub async fn delete(pool: &SqlitePool, id: &str) -> Result<u64, sqlx::Error> {
        StackLibraryRepository::delete(pool, id).await
    }

    pub async fn find_by_id(pool: &SqlitePool, id: &str) -> Result<Option<StackLibrary>, sqlx::Error> {
        StackLibraryRepository::find_by_id(pool, id).await
    }
}

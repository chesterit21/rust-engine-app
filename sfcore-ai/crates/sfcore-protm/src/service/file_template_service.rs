use crate::repository::FileTemplateRepository;
use crate::domain::FileTemplate;
use sqlx::SqlitePool;

pub struct FileTemplateService;

impl FileTemplateService {
    pub async fn get_all(pool: &SqlitePool) -> Result<Vec<FileTemplate>, sqlx::Error> {
        FileTemplateRepository::find_all(pool).await
    }

    pub async fn delete(pool: &SqlitePool, id: &str) -> Result<u64, sqlx::Error> {
        FileTemplateRepository::delete(pool, id).await
    }

    pub async fn find_by_id(pool: &SqlitePool, id: &str) -> Result<Option<FileTemplate>, sqlx::Error> {
        FileTemplateRepository::find_by_id(pool, id).await
    }
}

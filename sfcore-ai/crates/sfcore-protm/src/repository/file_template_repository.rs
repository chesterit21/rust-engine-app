use crate::domain::FileTemplate;
use sqlx::SqlitePool;

pub struct FileTemplateRepository;

impl FileTemplateRepository {
    pub async fn find_all(pool: &SqlitePool) -> Result<Vec<FileTemplate>, sqlx::Error> {
        sqlx::query_as::<_, FileTemplate>(
            "SELECT id, layer_id, name, file_naming, class_naming, code_template, required_imports, required_methods, description, created_at FROM file_templates"
        )
        .fetch_all(pool)
        .await
    }
    
    pub async fn delete(pool: &SqlitePool, id: &str) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM file_templates WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }

    pub async fn find_by_id(pool: &SqlitePool, id: &str) -> Result<Option<FileTemplate>, sqlx::Error> {
        sqlx::query_as::<_, FileTemplate>(
            "SELECT id, layer_id, name, file_naming, class_naming, code_template, required_imports, required_methods, description, created_at FROM file_templates WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }
}

use sqlx::{SqlitePool, Error as SqlxError};
use game_models::{SiteMaster, CreateSiteMaster, UpdateSiteMaster};  // <-- FIX: semua dari game_models

pub struct SiteMasterRepository {
    pool: SqlitePool,
}

impl SiteMasterRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn find_by_group_name(&self, group_name: &str) -> Result<Vec<SiteMaster>, SqlxError> {
        sqlx::query_as::<_, SiteMaster>(
            r#"
            SELECT "Id", "GroupName", "ProviderName", "LinkSite" 
            FROM "SiteMaster" 
            WHERE "GroupName" = ?1 
            ORDER BY "Id"
            "#
        )
        .bind(group_name)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn find_by_provider(&self, provider_name: &str) -> Result<Vec<SiteMaster>, SqlxError> {
        sqlx::query_as::<_, SiteMaster>(
            r#"
            SELECT "Id", "GroupName", "ProviderName", "LinkSite" 
            FROM "SiteMaster" 
            WHERE "ProviderName" = ?1 
            ORDER BY "Id"
            "#
        )
        .bind(provider_name)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Option<SiteMaster>, SqlxError> {
        sqlx::query_as::<_, SiteMaster>(
            r#"SELECT "Id", "GroupName", "ProviderName", "LinkSite" FROM "SiteMaster" WHERE "Id" = ?1"#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn find_all(&self) -> Result<Vec<SiteMaster>, SqlxError> {
        sqlx::query_as::<_, SiteMaster>(
            r#"SELECT "Id", "GroupName", "ProviderName", "LinkSite" FROM "SiteMaster" ORDER BY "GroupName""#
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn create(&self, data: &CreateSiteMaster) -> Result<SiteMaster, SqlxError> {  // <-- FIX: tambahin "data: "
        sqlx::query_as::<_, SiteMaster>(
            r#"
            INSERT INTO "SiteMaster" ("GroupName", "ProviderName", "LinkSite")
            VALUES (?1, ?2, ?3)
            RETURNING "Id", "GroupName", "ProviderName", "LinkSite"
            "#
        )
        .bind(&data.group_name)
        .bind(&data.provider_name)
        .bind(&data.link_site)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn update(&self, data: &UpdateSiteMaster) -> Result<SiteMaster, SqlxError> {  // <-- FIX: tambahin "data: "
        sqlx::query_as::<_, SiteMaster>(
            r#"
            UPDATE "SiteMaster" SET
                "GroupName" = ?2,
                "ProviderName" = ?3,
                "LinkSite" = ?4
            WHERE "Id" = ?1
            RETURNING "Id", "GroupName", "ProviderName", "LinkSite"
            "#
        )
        .bind(data.id)
        .bind(&data.group_name)
        .bind(&data.provider_name)
        .bind(&data.link_site)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn delete(&self, id: i64) -> Result<bool, SqlxError> {
        let result = sqlx::query(
            r#"DELETE FROM "SiteMaster" WHERE "Id" = ?1"#
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}
use sqlx::{SqlitePool, Error as SqlxError};
use game_models::{SetupLinkGame, CreateSetupLinkGame, UpdateSetupLinkGame};  // <-- FIX: semua dari game_models

pub struct SetupLinkGameRepository {
    pool: SqlitePool,
}

impl SetupLinkGameRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn find_by_game_code(&self, game_code: &str) -> Result<Vec<SetupLinkGame>, SqlxError> {
        sqlx::query_as::<_, SetupLinkGame>(
            r#"
            SELECT "Id", "LinkGame", "LinkType", "GameCode" 
            FROM "SetupLinkGame" 
            WHERE "GameCode" = ?1 
            ORDER BY "Id"
            "#
        )
        .bind(game_code)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn find_by_link_type(&self, link_type: &str) -> Result<Vec<SetupLinkGame>, SqlxError> {
        sqlx::query_as::<_, SetupLinkGame>(
            r#"
            SELECT "Id", "LinkGame", "LinkType", "GameCode" 
            FROM "SetupLinkGame" 
            WHERE "LinkType" = ?1 
            ORDER BY "Id"
            "#
        )
        .bind(link_type)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Option<SetupLinkGame>, SqlxError> {
        sqlx::query_as::<_, SetupLinkGame>(
            r#"SELECT "Id", "LinkGame", "LinkType", "GameCode" FROM "SetupLinkGame" WHERE "Id" = ?1"#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn find_all(&self) -> Result<Vec<SetupLinkGame>, SqlxError> {
        sqlx::query_as::<_, SetupLinkGame>(
            r#"SELECT "Id", "LinkGame", "LinkType", "GameCode" FROM "SetupLinkGame" ORDER BY "Id""#
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn create(&self, data: &CreateSetupLinkGame) -> Result<SetupLinkGame, SqlxError> {  // <-- FIX: tambahin "data: "
        sqlx::query_as::<_, SetupLinkGame>(
            r#"
            INSERT INTO "SetupLinkGame" ("LinkGame", "LinkType", "GameCode")
            VALUES (?1, ?2, ?3)
            RETURNING "Id", "LinkGame", "LinkType", "GameCode"
            "#
        )
        .bind(&data.link_game)
        .bind(&data.link_type)
        .bind(&data.game_code)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn update(&self, data: &UpdateSetupLinkGame) -> Result<SetupLinkGame, SqlxError> {  // <-- FIX: tambahin "data: "
        sqlx::query_as::<_, SetupLinkGame>(
            r#"
            UPDATE "SetupLinkGame" SET
                "LinkGame" = ?2,
                "LinkType" = ?3,
                "GameCode" = ?4
            WHERE "Id" = ?1
            RETURNING "Id", "LinkGame", "LinkType", "GameCode"
            "#
        )
        .bind(data.id)
        .bind(&data.link_game)
        .bind(&data.link_type)
        .bind(&data.game_code)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn delete(&self, id: i64) -> Result<bool, SqlxError> {
        let result = sqlx::query(
            r#"DELETE FROM "SetupLinkGame" WHERE "Id" = ?1"#
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}
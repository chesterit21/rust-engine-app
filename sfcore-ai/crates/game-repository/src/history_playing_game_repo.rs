use sqlx::{SqlitePool, Error as SqlxError};
use game_models::{HistoryPlayingGame, CreateHistoryPlayingGame, HistoryPlayingGameFilter};  // <-- FIX: import dari game_models

pub struct HistoryPlayingGameRepository {
    pool: SqlitePool,
}

impl HistoryPlayingGameRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn find_by_game_code(
        &self,
        game_code: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<HistoryPlayingGame>, SqlxError> {
        sqlx::query_as::<_, HistoryPlayingGame>(
            r#"
            SELECT 
                "Id", "TransCode", "GameId", "GameCode", "CreatedBy", "CreatedDate",
                "TemplateNumberId", "TypePick", "Number"
            FROM "HistoryPlayingGame" 
            WHERE "GameCode" = ?1
            ORDER BY "Id" DESC
            LIMIT ?2 OFFSET ?3
            "#
        )
        .bind(game_code)
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn find_by_trans_code(&self, trans_code: &str) -> Result<Option<HistoryPlayingGame>, SqlxError> {
        sqlx::query_as::<_, HistoryPlayingGame>(
            r#"
            SELECT 
                "Id", "TransCode", "GameId", "GameCode", "CreatedBy", "CreatedDate",
                "TemplateNumberId", "TypePick", "Number"
            FROM "HistoryPlayingGame" 
            WHERE "TransCode" = ?1
            "#
        )
        .bind(trans_code)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn find_by_created_by(
        &self,
        created_by: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<HistoryPlayingGame>, SqlxError> {
        sqlx::query_as::<_, HistoryPlayingGame>(
            r#"
            SELECT 
                "Id", "TransCode", "GameId", "GameCode", "CreatedBy", "CreatedDate",
                "TemplateNumberId", "TypePick", "Number"
            FROM "HistoryPlayingGame" 
            WHERE "CreatedBy" = ?1
            ORDER BY "Id" DESC
            LIMIT ?2 OFFSET ?3
            "#
        )
        .bind(created_by)
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn find_with_filter(
        &self,
        filter: &HistoryPlayingGameFilter,
    ) -> Result<Vec<HistoryPlayingGame>, SqlxError> {
        if let Some(ref game_code) = filter.game_code {
            self.find_by_game_code(
                game_code,
                filter.page_size.unwrap_or(50),
                filter.page.unwrap_or(1).saturating_sub(1) * filter.page_size.unwrap_or(50),
            ).await
        } else if let Some(ref created_by) = filter.created_by {
            self.find_by_created_by(
                created_by,
                filter.page_size.unwrap_or(50),
                filter.page.unwrap_or(1).saturating_sub(1) * filter.page_size.unwrap_or(50),
            ).await
        } else {
            // Default: get latest records
            self.find_by_game_code("", 50, 0).await
        }
    }

    pub async fn create(&self, data: &CreateHistoryPlayingGame) -> Result<HistoryPlayingGame, SqlxError> {  // <-- FIX: tambahin "data: "
        sqlx::query_as::<_, HistoryPlayingGame>(
            r#"
            INSERT INTO "HistoryPlayingGame" 
                ("TransCode", "GameId", "GameCode", "CreatedBy", "CreatedDate",
                 "TemplateNumberId", "TypePick", "Number")
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            RETURNING 
                "Id", "TransCode", "GameId", "GameCode", "CreatedBy", "CreatedDate",
                "TemplateNumberId", "TypePick", "Number"
            "#
        )
        .bind(&data.trans_code)
        .bind(data.game_id)
        .bind(&data.game_code)
        .bind(&data.created_by)
        .bind(&data.created_date)
        .bind(data.template_number_id)
        .bind(&data.type_pick)
        .bind(&data.number)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn count_by_game_code(&self, game_code: &str) -> Result<i64, SqlxError> {
        let result = sqlx::query_scalar::<_, i64>(
            r#"SELECT COUNT(*) FROM "HistoryPlayingGame" WHERE "GameCode" = ?1"#
        )
        .bind(game_code)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn count_by_player(&self, player_name: &str) -> Result<i64, SqlxError> {
        let result = sqlx::query_scalar::<_, i64>(
            r#"SELECT COUNT(*) FROM "HistoryPlayingGame" WHERE "CreatedBy" = ?1"#
        )
        .bind(player_name)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn get_history_summary(&self) -> Result<Vec<game_models::HistorySummary>, SqlxError> {
        sqlx::query_as::<_, game_models::HistorySummary>(
            r#"
            SELECT 
                Q.GameCode,
                Q.TransCode,
                printf('IDR %d', 340000 - COUNT(H.Id) * 34) AS BUYS,
                printf('IDR %d', COUNT(H.Id) * 34) AS TOTAL_COLLECT
            FROM PlayingGameQueue Q
            LEFT JOIN HistoryPlayingGame H 
                ON Q.TransCode = H.TransCode 
                AND Q.GameCode = H.GameCode
            GROUP BY Q.GameCode, Q.TransCode
            ORDER BY Q.TransCode DESC
            "#
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn delete_by_game_code(&self, game_code: &str) -> Result<(), SqlxError> {
        let mut tx = self.pool.begin().await?;

        // 1. Delete from PlayingGameQueue
        sqlx::query(r#"DELETE FROM "PlayingGameQueue" WHERE "GameCode" = ?1"#)
            .bind(game_code)
            .execute(&mut *tx)
            .await?;

        // 2. Delete from HistoryPlayingGame
        sqlx::query(r#"DELETE FROM "HistoryPlayingGame" WHERE "GameCode" = ?1"#)
            .bind(game_code)
            .execute(&mut *tx)
            .await?;

        // 3. Delete from DataPlaying
        sqlx::query(r#"DELETE FROM "DataPlaying" WHERE "GameCode" = ?1"#)
            .bind(game_code)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn truncate_all(&self) -> Result<(), SqlxError> {
        let mut tx = self.pool.begin().await?;

        // Delete from all tables
        sqlx::query(r#"DELETE FROM "DataPlaying""#).execute(&mut *tx).await?;
        sqlx::query(r#"DELETE FROM "HistoryPlayingGame""#).execute(&mut *tx).await?;
        sqlx::query(r#"DELETE FROM "PlayingGameQueue""#).execute(&mut *tx).await?;
        sqlx::query(r#"DELETE FROM "PlayingGame""#).execute(&mut *tx).await?;

        // Reset Sequences (Only if sqlite_sequence exists - but since it errors, we skip it)
        // If INTEGER PRIMARY KEY is used without AUTOINCREMENT, IDs reset automatically when empty.
        // sqlx::query(r#"DELETE FROM sqlite_sequence WHERE name IN (...)"#)... 
        
        tx.commit().await?;
        Ok(())
    }

    pub async fn find_missing_numbers(&self, trans_code: &str) -> Result<String, SqlxError> {
        let result: (String,) = sqlx::query_as(
            r#"
            SELECT IFNULL(GROUP_CONCAT(TheNumber, '*'), '') AS TheNumber
            FROM (
                SELECT TheNumber
                FROM TemplateNumberFourDigit
                WHERE Id NOT IN (
                    SELECT TemplateNumberId
                    FROM HistoryPlayingGame
                    WHERE TransCode = ?
                )
                ORDER BY Id ASC
            )
            "#
        )
        .bind(trans_code)
        .fetch_one(&self.pool)
        .await?;

        Ok(result.0)
    }
}
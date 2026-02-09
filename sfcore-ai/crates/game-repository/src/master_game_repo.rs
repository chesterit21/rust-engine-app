use sqlx::{SqlitePool, Error as SqlxError};
use game_models::{MasterGame, CreateMasterGame, UpdateMasterGame};  // <-- FIX: dari game_models, bukan crate::models

pub struct MasterGameRepository {
    pool: SqlitePool,
}

impl MasterGameRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn find_by_game_code(&self, game_code: &str) -> Result<Option<MasterGame>, SqlxError> {
        sqlx::query_as::<_, MasterGame>(
            r#"
            SELECT 
                "Id", "GameCode", "GameName", "GameHour", "GameMinute", 
                "StartBetHour", "StartBetMinute", "LastResult", "LastPeriodeInRealGame", 
                "DateResult", "InputResultDate", "Holiday"
            FROM "MasterGame" 
            WHERE "GameCode" = ?1
            "#
        )
        .bind(game_code)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Option<MasterGame>, SqlxError> {
        sqlx::query_as::<_, MasterGame>(
            r#"
            SELECT 
                "Id", "GameCode", "GameName", "GameHour", "GameMinute", 
                "StartBetHour", "StartBetMinute", "LastResult", "LastPeriodeInRealGame", 
                "DateResult", "InputResultDate", "Holiday"
            FROM "MasterGame" 
            WHERE "Id" = ?1
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn find_all(&self) -> Result<Vec<MasterGame>, SqlxError> {
        sqlx::query_as::<_, MasterGame>(
            r#"
            SELECT 
                "Id", "GameCode", "GameName", "GameHour", "GameMinute", 
                "StartBetHour", "StartBetMinute", "LastResult", "LastPeriodeInRealGame", 
                "DateResult", "InputResultDate", "Holiday"
            FROM "MasterGame" 
            ORDER BY "StartBetHour", "StartBetMinute"
            "#
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn find_active_games(&self) -> Result<Vec<MasterGame>, SqlxError> {
        sqlx::query_as::<_, MasterGame>(
            r#"
            SELECT 
                "Id", "GameCode", "GameName", "GameHour", "GameMinute", 
                "StartBetHour", "StartBetMinute", "LastResult", "LastPeriodeInRealGame", 
                "DateResult", "InputResultDate", "Holiday"
            FROM "MasterGame" 
            ORDER BY "StartBetHour", "StartBetMinute"
            "#
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn create(&self, data: &CreateMasterGame) -> Result<MasterGame, SqlxError> {  // <-- FIX: tambahin "data: "
        sqlx::query_as::<_, MasterGame>(
            r#"
            INSERT INTO "MasterGame" 
                ("GameCode", "GameName", "GameHour", "GameMinute", 
                 "StartBetHour", "StartBetMinute", "LastResult", 
                 "LastPeriodeInRealGame", "DateResult", "InputResultDate", "Holiday")
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            RETURNING 
                "Id", "GameCode", "GameName", "GameHour", "GameMinute", 
                "StartBetHour", "StartBetMinute", "LastResult", "LastPeriodeInRealGame", 
                "DateResult", "InputResultDate", "Holiday"
            "#
        )
        .bind(&data.game_code)
        .bind(&data.game_name)
        .bind(data.game_hour)
        .bind(data.game_minute)
        .bind(data.start_bet_hour)
        .bind(data.start_bet_minute)
        .bind(&data.last_result)
        .bind(data.last_periode_in_real_game)
        .bind(&data.date_result)
        .bind(&data.input_result_date)
        .bind(&data.holiday)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn update(&self, data: &UpdateMasterGame) -> Result<MasterGame, SqlxError> {  // <-- FIX: tambahin "data: "
        sqlx::query_as::<_, MasterGame>(
            r#"
            UPDATE "MasterGame" SET
                "GameCode" = ?2,
                "GameName" = ?3,
                "GameHour" = ?4,
                "GameMinute" = ?5,
                "StartBetHour" = ?6,
                "StartBetMinute" = ?7,
                "LastResult" = ?8,
                "LastPeriodeInRealGame" = ?9,
                "DateResult" = ?10,
                "InputResultDate" = ?11,
                "Holiday" = ?12
            WHERE "Id" = ?1
            RETURNING 
                "Id", "GameCode", "GameName", "GameHour", "GameMinute", 
                "StartBetHour", "StartBetMinute", "LastResult", "LastPeriodeInRealGame", 
                "DateResult", "InputResultDate", "Holiday"
            "#
        )
        .bind(data.id)
        .bind(&data.game_code)
        .bind(&data.game_name)
        .bind(data.game_hour)
        .bind(data.game_minute)
        .bind(data.start_bet_hour)
        .bind(data.start_bet_minute)
        .bind(&data.last_result)
        .bind(data.last_periode_in_real_game)
        .bind(&data.date_result)
        .bind(&data.input_result_date)
        .bind(&data.holiday)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn delete(&self, id: i64) -> Result<bool, SqlxError> {
        let result = sqlx::query(
            r#"DELETE FROM "MasterGame" WHERE "Id" = ?1"#
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Custom query for Dashboard: Active games with specific formatting
    pub async fn get_dashboard_active_games(&self) -> Result<Vec<game_models::templates::DashboardGameResult>, SqlxError> {
        sqlx::query_as::<_, game_models::templates::DashboardGameResult>(
            r#"
            SELECT 
                "GameCode", 
                "LastPeriodeInRealGame" As "Periode", 
                "GameHour", 
                "GameMinute", 
                "DateResult",
                CASE 
                    WHEN "LastResult" IS NOT NULL AND TRIM("LastResult") != '' 
                    THEN SUBSTR(TRIM("LastResult"), 1, 1) || '.' || SUBSTR(TRIM("LastResult"), 2, 3)
                    ELSE NULL 
                END AS "LastResult",
                "Holiday", 
                "InputResultDate"
            FROM "MasterGame"
            WHERE "LastResult" IS NOT NULL 
              AND "Periode" > 0 
              AND TRIM("LastResult") != ''
            ORDER BY "GameHour", "GameMinute"
            "#
        )
        .fetch_all(&self.pool)
        .await
    }
}
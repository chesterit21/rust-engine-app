use sqlx::{SqlitePool, Error as SqlxError};
use game_models::{LogGame, CreateLogGame, LogGameFilter, LogAnalysisResult};

pub struct LogGameRepository {
    pool: SqlitePool,
}

impl LogGameRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn find_by_game_code(
        &self,
        game_code: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<LogGame>, SqlxError> {
        sqlx::query_as::<_, LogGame>(
            r#"
            SELECT 
                "Id", "GameCode", "Periode", "LogResult", "As", "Kop", 
                "Kepala", "Ekor", "CreatedDate", "DateResultInGame"
            FROM "LogGame" 
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

    pub async fn find_by_periode(&self, periode: i64) -> Result<Vec<LogGame>, SqlxError> {
        sqlx::query_as::<_, LogGame>(
            r#"
            SELECT 
                "Id", "GameCode", "Periode", "LogResult", "As", "Kop", 
                "Kepala", "Ekor", "CreatedDate", "DateResultInGame"
            FROM "LogGame" 
            WHERE "Periode" = ?1 ORDER BY "Id"
            "#
        )
        .bind(periode)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Option<LogGame>, SqlxError> {
        sqlx::query_as::<_, LogGame>(
            r#"
            SELECT 
                "Id", "GameCode", "Periode", "LogResult", "As", "Kop", 
                "Kepala", "Ekor", "CreatedDate", "DateResultInGame"
            FROM "LogGame" 
            WHERE "Id" = ?1
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    // Simplified filter - no dynamic query building for now
    pub async fn find_with_filter(
        &self,
        filter: &LogGameFilter,
    ) -> Result<Vec<LogGame>, SqlxError> {
        if let Some(ref game_code) = filter.game_code {
            self.find_by_game_code(
                game_code,
                filter.page_size.unwrap_or(50),
                filter.page.unwrap_or(1).saturating_sub(1) * filter.page_size.unwrap_or(50),
            ).await
        } else if let Some(periode) = filter.periode {
            self.find_by_periode(periode).await
        } else {
            // Default: get latest logs
            self.find_by_game_code("", 50, 0).await
        }
    }

    pub async fn count_by_game_code(&self, game_code: &str) -> Result<i64, SqlxError> {
        let result = sqlx::query_scalar::<_, i64>(
            r#"SELECT COUNT(*) FROM "LogGame" WHERE "GameCode" = ?1"#
        )
        .bind(game_code)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn create(&self, data: &CreateLogGame) -> Result<LogGame, SqlxError> {  // <-- FIX: tambahin "data: "
        sqlx::query_as::<_, LogGame>(
            r#"
            INSERT INTO "LogGame" 
                ("GameCode", "Periode", "LogResult", "As", "Kop", "Kepala", "Ekor", 
                 "CreatedDate", "DateResultInGame")
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            RETURNING 
                "Id", "GameCode", "Periode", "LogResult", "As", "Kop", 
                "Kepala", "Ekor", "CreatedDate", "DateResultInGame"
            "#
        )
        .bind(&data.game_code)
        .bind(data.periode)
        .bind(&data.log_result)
        .bind(data.as_digit)
        .bind(data.kop)
        .bind(data.kepala)
        .bind(data.ekor)
        .bind(&data.created_date)
        .bind(&data.date_result_in_game)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn find_latest_by_game_code(&self, game_code: &str) -> Result<Option<LogGame>, SqlxError> {
        sqlx::query_as::<_, LogGame>(
            r#"
            SELECT 
                "Id", "GameCode", "Periode", "LogResult", "As", "Kop", 
                "Kepala", "Ekor", "CreatedDate", "DateResultInGame"
            FROM "LogGame" 
            WHERE "GameCode" = ?1
            ORDER BY "Id" DESC
            LIMIT 1
            "#
        )
        .bind(game_code)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn find_by_date_range(
        &self,
        game_code: &str,
        from_date: &str,
        to_date: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<LogGame>, SqlxError> {
        sqlx::query_as::<_, LogGame>(
            r#"
            SELECT 
                "Id", "GameCode", "Periode", "LogResult", "As", "Kop", 
                "Kepala", "Ekor", "CreatedDate", "DateResultInGame"
            FROM "LogGame" 
            WHERE "GameCode" = ?1 
                AND "CreatedDate" >= ?2 
                AND "CreatedDate" <= ?3
            ORDER BY "Id" DESC
            LIMIT ?4 OFFSET ?5
            "#
        )
        .bind(game_code)
        .bind(from_date)
        .bind(to_date)
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn find_analysis_by_game_code(&self, game_code: &str) -> Result<Vec<LogAnalysisResult>, SqlxError> {
        sqlx::query_as::<_, LogAnalysisResult>(
            r#"
            SELECT 
                Periode,
                SUBSTR(TRIM(LogResult), 1, 1) || '.' || SUBSTR(TRIM(LogResult), 2, 3) AS formatted_result,
                CASE 
                    WHEN prev_logresult IS NULL THEN 'N/A'
                    WHEN current_num > prev_num THEN 'UP'
                    WHEN current_num < prev_num THEN 'DOWN'
                    ELSE 'SAME'
                END AS trend,
                SUBSTR(TRIM(prev_logresult), 1, 1) || '.' || SUBSTR(TRIM(prev_logresult), 2, 3) AS prev_formatted
            FROM (
                SELECT 
                    "Periode" as Periode,
                    "LogResult" as LogResult,
                    CAST(TRIM("LogResult") AS INTEGER) AS current_num,
                    LAG(TRIM("LogResult")) OVER (ORDER BY "Periode" ASC) AS prev_logresult,
                    LAG(CAST(TRIM("LogResult") AS INTEGER)) OVER (ORDER BY "Periode" ASC) AS prev_num
                FROM "LogGame"
                WHERE "GameCode" = ?1 
                  AND "LogResult" IS NOT NULL 
                  AND TRIM("LogResult") != ''
                  AND LENGTH(TRIM("LogResult")) = 4
            ) sub
            ORDER BY Periode DESC
            "#
        )
        .bind(game_code)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn find_previous_logs(
        &self,
        game_code: &str,
        periode: i64,
        limit: u32,
    ) -> Result<Vec<LogGame>, SqlxError> {
        sqlx::query_as::<_, LogGame>(
            r#"
            SELECT 
                "Id", "GameCode", "Periode", "LogResult", "As", "Kop", 
                "Kepala", "Ekor", "CreatedDate", "DateResultInGame"
            FROM "LogGame" 
            WHERE "GameCode" = ?1 AND "Periode" < ?2
            ORDER BY "Periode" DESC
            LIMIT ?3
            "#
        )
        .bind(game_code)
        .bind(periode)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
    }
}
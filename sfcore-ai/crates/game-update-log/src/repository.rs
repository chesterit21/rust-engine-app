use anyhow::{Context, Result};
use sqlx::SqlitePool;
use crate::models::{LogGame, MasterGame, SetupLinkGame};

pub struct GameRepository {
    pool: SqlitePool,
}

impl GameRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    // ========== LogGame Methods ==========
    
    pub async fn save_log(&self, log: &LogGame) -> Result<i64> {
        let result = sqlx::query(
            r#"
            INSERT INTO LogGame 
            (GameCode, Periode, LogResult, DateResultInGame, As, Kop, Kepala, Ekor)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
        )
        .bind(&log.game_code)
        .bind(log.periode)
        .bind(&log.log_result)
        .bind(&log.date_result_in_game)
        .bind(log.as_)
        .bind(log.kop)
        .bind(log.kepala)
        .bind(log.ekor)
        .execute(&self.pool)
        .await
        .context("Failed to insert log game")?;

        Ok(result.last_insert_rowid())
    }

    pub async fn save_logs_bulk(&self, logs: &[LogGame]) -> Result<u64> {
        let mut tx = self.pool.begin().await?;
        let mut count = 0u64;

        for log in logs {
            sqlx::query(
                r#"
                INSERT INTO LogGame 
                (GameCode, Periode, LogResult, DateResultInGame, As, Kop, Kepala, Ekor)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                "#,
            )
            .bind(&log.game_code)
            .bind(log.periode)
            .bind(&log.log_result)
            .bind(&log.date_result_in_game)
            .bind(log.as_)
            .bind(log.kop)
            .bind(log.kepala)
            .bind(log.ekor)
            .execute(&mut *tx)
            .await?;
            
            count += 1;
        }

        tx.commit().await?;
        Ok(count)
    }

    pub async fn get_log_by_game_code_and_periode(
        &self,
        game_code: &str,
        periode: i32,
    ) -> Result<Option<LogGame>> {
        let log = sqlx::query_as::<_, LogGame>(
            r#"
            SELECT id, GameCode as game_code, Periode as periode, 
                   LogResult as log_result, DateResultInGame as date_result_in_game,
                   As as as_, Kop as kop, Kepala as kepala, Ekor as ekor,
                   NULL as created_at
            FROM LogGame 
            WHERE GameCode = ?1 AND Periode = ?2
            LIMIT 1
            "#,
        )
        .bind(game_code)
        .bind(periode)
        .fetch_optional(&self.pool)
        .await?;

        Ok(log)
    }

    pub async fn get_logs_by_game_code(&self, game_code: &str) -> Result<Vec<LogGame>> {
        let logs = sqlx::query_as::<_, LogGame>(
            r#"
            SELECT id, GameCode as game_code, Periode as periode, 
                   LogResult as log_result, DateResultInGame as date_result_in_game,
                   As as as_, Kop as kop, Kepala as kepala, Ekor as ekor,
                   NULL as created_at
            FROM LogGame 
            WHERE GameCode = ?1 
            ORDER BY Periode DESC
            "#,
        )
        .bind(game_code)
        .fetch_all(&self.pool)
        .await?;

        Ok(logs)
    }

    pub async fn get_last_month_log_games(&self, game_code: &str) -> Result<Vec<LogGame>> {
        let logs = sqlx::query_as::<_, LogGame>(
            r#"
            SELECT id, GameCode as game_code, Periode as periode, 
                   LogResult as log_result, DateResultInGame as date_result_in_game,
                   As as as_, Kop as kop, Kepala as kepala, Ekor as ekor,
                   NULL as created_at
            FROM LogGame 
            WHERE GameCode = ?1 
            ORDER BY Periode DESC 
            LIMIT 45
            "#,
        )
        .bind(game_code)
        .fetch_all(&self.pool)
        .await?;

        Ok(logs)
    }

    pub async fn get_top_logs(&self, game_code: &str, limit: i32) -> Result<Vec<LogGame>> {
        let logs = sqlx::query_as::<_, LogGame>(
            r#"
            SELECT id, GameCode as game_code, Periode as periode, 
                   LogResult as log_result, DateResultInGame as date_result_in_game,
                   As as as_, Kop as kop, Kepala as kepala, Ekor as ekor,
                   NULL as created_at
            FROM LogGame 
            WHERE GameCode = ?1 
            ORDER BY Periode DESC 
            LIMIT ?2
            "#,
        )
        .bind(game_code)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(logs)
    }

    // ========== MasterGame Methods ==========

    pub async fn get_master_game(&self, game_code: &str) -> Result<Option<MasterGame>> {
        let master = sqlx::query_as::<_, MasterGame>(
            r#"
            SELECT id, GameCode as game_code, 
                   LastPeriodeInRealGame as last_periode_in_real_game,
                   LastResult as last_result, 
                   InputResultDate as input_result_date,
                   DateResult as date_result,
                   GameHour as game_hour, GameMinute as game_minute,
                   StartBetHour as start_bet_hour, StartBetMinute as start_bet_minute
            FROM MasterGame 
            WHERE GameCode = ?1
            LIMIT 1
            "#,
        )
        .bind(game_code)
        .fetch_optional(&self.pool)
        .await?;

        Ok(master)
    }

    pub async fn get_all_master_games(&self) -> Result<Vec<MasterGame>> {
        let masters = sqlx::query_as::<_, MasterGame>(
            r#"
            SELECT id, GameCode as game_code, 
                   LastPeriodeInRealGame as last_periode_in_real_game,
                   LastResult as last_result, 
                   InputResultDate as input_result_date,
                   DateResult as date_result,
                   GameHour as game_hour, GameMinute as game_minute,
                   StartBetHour as start_bet_hour, StartBetMinute as start_bet_minute
            FROM MasterGame
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(masters)
    }

    pub async fn update_master_game(&self, master: &MasterGame) -> Result<u64> {
        let result = sqlx::query(
            r#"
            UPDATE MasterGame 
            SET LastPeriodeInRealGame = ?1, 
                LastResult = ?2, 
                InputResultDate = ?3, 
                DateResult = ?4
            WHERE GameCode = ?5
            "#,
        )
        .bind(master.last_periode_in_real_game)
        .bind(&master.last_result)
        .bind(master.input_result_date)
        .bind(&master.date_result)
        .bind(&master.game_code)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    // ========== SetupLinkGame Methods ==========

    pub async fn get_link_header(&self) -> Result<Option<SetupLinkGame>> {
        let link = sqlx::query_as::<_, SetupLinkGame>(
            r#"
            SELECT Id as id, LinkType as link_type, 
                   LinkGame as link_game, GameCode as game_code
            FROM SetupLinkGame 
            WHERE LinkType = 'CR-HEADER'
            LIMIT 1
            "#,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(link)
    }

    pub async fn get_link_details(&self) -> Result<Vec<SetupLinkGame>> {
        let links = sqlx::query_as::<_, SetupLinkGame>(
            r#"
            SELECT Id as id, LinkType as link_type, 
                   LinkGame as link_game, GameCode as game_code
            FROM SetupLinkGame 
            WHERE LinkType = 'CR-DETAIL'
            ORDER BY Id
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(links)
    }

    pub async fn get_link_header_mq(&self) -> Result<Option<SetupLinkGame>> {
        let link = sqlx::query_as::<_, SetupLinkGame>(
            r#"
            SELECT Id as id, LinkType as link_type, 
                   LinkGame as link_game, GameCode as game_code
            FROM SetupLinkGame 
            WHERE LinkType = 'CR-HEADER-MQ'
            LIMIT 1
            "#,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(link)
    }

    pub async fn get_link_details_mq(&self) -> Result<Vec<SetupLinkGame>> {
        let links = sqlx::query_as::<_, SetupLinkGame>(
            r#"
            SELECT Id as id, LinkType as link_type, 
                   LinkGame as link_game, GameCode as game_code
            FROM SetupLinkGame 
            WHERE LinkType = 'CR-DETAIL-MQ'
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(links)
    }

    // ========== Utility Methods ==========

    pub async fn remove_duplicate_periode(&self, game_code: &str) -> Result<u64> {
        let result = sqlx::query(
            r#"
            DELETE FROM LogGame 
            WHERE Id IN (
                SELECT Id FROM (
                    SELECT Id,
                        ROW_NUMBER() OVER (
                            PARTITION BY Periode      
                            ORDER BY Id ASC           
                        ) AS RowNum
                    FROM LogGame
                    WHERE GameCode = ?1
                )
                WHERE RowNum > 1
            )
            "#,
        )
        .bind(game_code)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }
}

// Clone implementation
impl Clone for GameRepository {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
        }
    }
}
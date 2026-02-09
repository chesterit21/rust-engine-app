use sqlx::{SqlitePool, Error as SqlxError};
use game_models::{  // <-- FIX: import dari game_models
    PatternHitGameQueue,
    PatternHitGameHistory,
    TemplatePatternHitGame,
    CreatePatternHitGameQueue,
    CreatePatternHitGameHistory,
    CreateTemplatePatternHitGame,
};

pub struct PatternHitRepository {
    pool: SqlitePool,
}

impl PatternHitRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    // ==================== PatternHitGameQueue ====================

    pub async fn get_queue_by_game_code(&self, game_code: &str) -> Result<Vec<PatternHitGameQueue>, SqlxError> {
        sqlx::query_as::<_, PatternHitGameQueue>(
            r#"
            SELECT 
                "Id", "GameCode", "PatternFront", "PatternBack"
            FROM "PatternHitGameQueue" 
            WHERE "GameCode" = ?1 
            ORDER BY "Id"
            "#
        )
        .bind(game_code)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn create_queue_item(&self, data: &CreatePatternHitGameQueue) -> Result<PatternHitGameQueue, SqlxError> {  // <-- FIX: tambahin "data: "
        sqlx::query_as::<_, PatternHitGameQueue>(
            r#"
            INSERT INTO "PatternHitGameQueue" 
                ("GameCode", "PatternFront", "PatternBack")
            VALUES (?1, ?2, ?3)
            RETURNING 
                "Id", "GameCode", "PatternFront", "PatternBack"
            "#
        )
        .bind(&data.game_code)
        .bind(&data.pattern_front)
        .bind(&data.pattern_back)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_history_by_game_code(
        &self,
        game_code: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<PatternHitGameHistory>, SqlxError> {
        sqlx::query_as::<_, PatternHitGameHistory>(
            r#"
            SELECT 
                "Id", "GameCode", "PatternFront", "PatternBack", "PlayDate", "IsWin"
            FROM "PatternHitGameHistory" 
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

    pub async fn get_winning_patterns(
        &self,
        game_code: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<PatternHitGameHistory>, SqlxError> {
        sqlx::query_as::<_, PatternHitGameHistory>(
            r#"
            SELECT 
                "Id", "GameCode", "PatternFront", "PatternBack", "PlayDate", "IsWin"
            FROM "PatternHitGameHistory" 
            WHERE "GameCode" = ?1 AND "IsWin" = 1
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

    pub async fn create_history(&self, data: &CreatePatternHitGameHistory) -> Result<PatternHitGameHistory, SqlxError> {  // <-- FIX: tambahin "data: "
        sqlx::query_as::<_, PatternHitGameHistory>(
            r#"
            INSERT INTO "PatternHitGameHistory" 
                ("GameCode", "PatternFront", "PatternBack", "PlayDate", "IsWin")
            VALUES (?1, ?2, ?3, ?4, ?5)
            RETURNING 
                "Id", "GameCode", "PatternFront", "PatternBack", "PlayDate", "IsWin"
            "#
        )
        .bind(&data.game_code)
        .bind(&data.pattern_front)
        .bind(&data.pattern_back)
        .bind(&data.play_date)
        .bind(data.is_win)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_templates_by_game_code(&self, game_code: &str) -> Result<Vec<TemplatePatternHitGame>, SqlxError> {
        sqlx::query_as::<_, TemplatePatternHitGame>(
            r#"
            SELECT 
                "Id", "GameCode", "PatternFront", "PatternBack"
            FROM "TemplatePatternHitGame" 
            WHERE "GameCode" = ?1 
            ORDER BY "Id"
            "#
        )
        .bind(game_code)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn create_template(&self, data: &CreateTemplatePatternHitGame) -> Result<TemplatePatternHitGame, SqlxError> {  // <-- FIX: tambahin "data: "
        sqlx::query_as::<_, TemplatePatternHitGame>(
            r#"
            INSERT INTO "TemplatePatternHitGame" 
                ("GameCode", "PatternFront", "PatternBack")
            VALUES (?1, ?2, ?3)
            RETURNING 
                "Id", "GameCode", "PatternFront", "PatternBack"
            "#
        )
        .bind(&data.game_code)
        .bind(&data.pattern_front)
        .bind(&data.pattern_back)
        .fetch_one(&self.pool)
        .await
    }
}
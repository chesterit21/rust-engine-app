use sqlx::{SqlitePool, Error as SqlxError};
use game_models::{PlayingGameQueue, CreatePlayingGameQueue};

pub struct PlayingGameQueueRepository {
    pool: SqlitePool,
}

impl PlayingGameQueueRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, data: &CreatePlayingGameQueue) -> Result<PlayingGameQueue, SqlxError> {
        sqlx::query_as::<_, PlayingGameQueue>(
            r#"
            INSERT INTO "PlayingGameQueue" 
                ("GameId", "GameCode", "TransCode", "CreatedBy", "CreatedDate", "IsWin")
            VALUES (?1, ?2, ?3, ?4, ?5, 0)
            RETURNING 
                "Id", "GameId", "GameCode", "TransCode", "CreatedBy", "CreatedDate", "IsWin"
            "#
        )
        .bind(data.game_id)
        .bind(&data.game_code)
        .bind(&data.trans_code)
        .bind(&data.created_by)
        .bind(&data.created_date)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn find_by_trans_code(&self, trans_code: &str) -> Result<Option<PlayingGameQueue>, SqlxError> {
        sqlx::query_as::<_, PlayingGameQueue>(
            r#"SELECT "Id", "GameId", "GameCode", "TransCode", "CreatedBy", "CreatedDate", "IsWin" 
               FROM "PlayingGameQueue" WHERE "TransCode" = ?1"#
        )
        .bind(trans_code)
        .fetch_optional(&self.pool)
        .await
    }
}

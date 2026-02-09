use sqlx::{SqlitePool, Error as SqlxError};
use game_models::{DataPlaying, CreateDataPlaying};

pub struct DataPlayingRepository {
    pool: SqlitePool,
}

impl DataPlayingRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, data: &CreateDataPlaying) -> Result<DataPlaying, SqlxError> {
        sqlx::query_as::<_, DataPlaying>(
            r#"
            INSERT INTO "DataPlaying" 
                ("GameCode", "Digit", "Tipe")
            VALUES (?1, ?2, ?3)
            RETURNING 
                "Id", "GameCode", "Digit", "Tipe"
            "#
        )
        .bind(&data.game_code)
        .bind(&data.digit)
        .bind(&data.tipe)
        .fetch_one(&self.pool)
        .await
    }
}

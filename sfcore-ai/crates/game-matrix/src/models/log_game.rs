use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LogGame {
    pub id: i64,
    pub game_code: String,
    pub player_id: String,
    pub bet_amount: i64,
    pub win_amount: i64,
    pub game_result: String,  // bisa JSON string nanti untuk detail result
    pub played_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLogGame {
    pub game_code: String,
    pub player_id: String,
    pub bet_amount: i64,
    pub win_amount: i64,
    pub game_result: String,
}
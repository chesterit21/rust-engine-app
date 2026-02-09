use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Master game configuration - "MasterGame" table
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MasterGame {
    #[sqlx(rename = "Id")]
    pub id: i64,

    #[sqlx(rename = "GameCode")]
    pub game_code: String,

    #[sqlx(rename = "GameName")]
    pub game_name: String,

    #[sqlx(rename = "GameHour")]
    pub game_hour: i64,

    #[sqlx(rename = "GameMinute")]
    pub game_minute: i64,

    #[sqlx(rename = "StartBetHour")]
    pub start_bet_hour: i64,

    #[sqlx(rename = "StartBetMinute")]
    pub start_bet_minute: i64,

    #[sqlx(rename = "LastResult")]
    pub last_result: String,

    #[sqlx(rename = "LastPeriodeInRealGame")]
    pub last_periode_in_real_game: i64,

    #[sqlx(rename = "DateResult")]
    pub date_result: String,

    #[sqlx(rename = "InputResultDate")]
    pub input_result_date: String,

    #[sqlx(rename = "Holiday")]
    pub holiday: String,
}

/// Create MasterGame input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMasterGame {
    pub game_code: String,
    pub game_name: String,
    pub game_hour: i64,
    pub game_minute: i64,
    pub start_bet_hour: i64,
    pub start_bet_minute: i64,
    pub last_result: String,
    pub last_periode_in_real_game: i64,
    pub date_result: String,
    pub input_result_date: String,
    pub holiday: String,
}

/// Update MasterGame input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMasterGame {
    pub id: i64,
    pub game_code: String,
    pub game_name: String,
    pub game_hour: i64,
    pub game_minute: i64,
    pub start_bet_hour: i64,
    pub start_bet_minute: i64,
    pub last_result: String,
    pub last_periode_in_real_game: i64,
    pub date_result: String,
    pub input_result_date: String,
    pub holiday: String,
}
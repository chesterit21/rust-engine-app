use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct PlayingGameQueue {
    #[sqlx(rename = "Id")]
    #[serde(rename = "Id")]
    pub id: i64,
    #[sqlx(rename = "GameId")]
    #[serde(rename = "GameId")]
    pub game_id: i64,
    #[sqlx(rename = "GameCode")]
    #[serde(rename = "GameCode")]
    pub game_code: String,
    #[sqlx(rename = "TransCode")]
    #[serde(rename = "TransCode")]
    pub trans_code: String,
    #[sqlx(rename = "CreatedBy")]
    #[serde(rename = "CreatedBy")]
    pub created_by: String,
    #[sqlx(rename = "CreatedDate")]
    #[serde(rename = "CreatedDate")]
    pub created_date: Option<String>,
    #[sqlx(rename = "IsWin")]
    #[serde(rename = "IsWin")]
    pub is_win: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePlayingGameQueue {
    pub game_id: i64,
    pub game_code: String,
    pub trans_code: String,
    pub created_by: String,
    pub created_date: Option<String>,
}

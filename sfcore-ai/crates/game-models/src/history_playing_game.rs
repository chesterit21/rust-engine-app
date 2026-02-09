use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Player game history - "HistoryPlayingGame" table
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct HistoryPlayingGame {
    #[sqlx(rename = "Id")]
    pub id: i64,

    #[sqlx(rename = "TransCode")]
    pub trans_code: String,

    #[sqlx(rename = "GameId")]
    pub game_id: i64,

    #[sqlx(rename = "GameCode")]
    pub game_code: String,

    #[sqlx(rename = "CreatedBy")]
    pub created_by: String,

    #[sqlx(rename = "CreatedDate")]
    pub created_date: String,

    #[sqlx(rename = "TemplateNumberId")]
    pub template_number_id: i64,

    #[sqlx(rename = "TypePick")]
    pub type_pick: String,

    #[sqlx(rename = "Number")]
    pub number: String,
}

/// Create HistoryPlayingGame input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateHistoryPlayingGame {
    pub trans_code: String,
    pub game_id: i64,
    pub game_code: String,
    pub created_by: String,
    pub created_date: String,
    pub template_number_id: i64,
    pub type_pick: String,
    pub number: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HistoryPlayingGameFilter {
    pub game_code: Option<String>,
    pub trans_code: Option<String>,
    pub created_by: Option<String>,
    pub from_date: Option<String>,
    pub to_date: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

/// Riwayat Ringkasan (Hasil JOIN)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct HistorySummary {
    #[sqlx(rename = "GameCode")]
    #[serde(rename = "GameCode")]
    pub game_code: String,
    
    #[sqlx(rename = "TransCode")]
    #[serde(rename = "TransCode")]
    pub trans_code: String,
    
    #[sqlx(rename = "BUYS")]
    #[serde(rename = "BUYS")]
    pub buys: String,
    
    #[sqlx(rename = "TOTAL_COLLECT")]
    #[serde(rename = "TOTAL_COLLECT")]
    pub total_collect: String,
}
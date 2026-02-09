use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Game log/transaction - "LogGame" table
/// Note: "As" field is a Rust keyword, so we use raw identifier r#"As"#
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LogGame {
    #[sqlx(rename = "Id")]
    pub id: i64,

    #[sqlx(rename = "GameCode")]
    pub game_code: String,

    #[sqlx(rename = "Periode")]
    pub periode: i64,

    #[sqlx(rename = "LogResult")]
    pub log_result: String,

    #[sqlx(rename = "As")]
    pub as_digit: Option<i64>,  // "As" field (Rust keyword)

    #[sqlx(rename = "Kop")]
    pub kop: Option<i64>,

    #[sqlx(rename = "Kepala")]
    pub kepala: Option<i64>,

    #[sqlx(rename = "Ekor")]
    pub ekor: Option<i64>,

    #[sqlx(rename = "CreatedDate")]
    pub created_date: Option<String>,

    #[sqlx(rename = "DateResultInGame")]
    pub date_result_in_game: Option<String>,
}

/// Create LogGame input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLogGame {
    pub game_code: String,
    pub periode: i64,
    pub log_result: String,
    pub as_digit: Option<i64>,
    pub kop: Option<i64>,
    pub kepala: Option<i64>,
    pub ekor: Option<i64>,
    pub created_date: Option<String>,
    pub date_result_in_game: Option<String>,
}

/// Filter untuk query LogGame
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LogGameFilter {
    pub game_code: Option<String>,
    pub periode: Option<i64>,
    pub from_date: Option<String>,
    pub to_date: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}
/// Result for Log Analysis with Trend
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LogAnalysisResult {
    #[sqlx(rename = "Periode")]
    pub periode: i64,
    
    pub formatted_result: Option<String>,
    
    pub trend: Option<String>,
    
    pub prev_formatted: Option<String>,
}
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Pattern hit queue - "PatternHitGameQueue" table
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PatternHitGameQueue {
    #[sqlx(rename = "Id")]
    pub id: i64,

    #[sqlx(rename = "GameCode")]
    pub game_code: String,

    #[sqlx(rename = "PatternFront")]
    pub pattern_front: String,

    #[sqlx(rename = "PatternBack")]
    pub pattern_back: String,
}

/// Pattern hit history - "PatternHitGameHistory" table
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PatternHitGameHistory {
    #[sqlx(rename = "Id")]
    pub id: i64,

    #[sqlx(rename = "GameCode")]
    pub game_code: String,

    #[sqlx(rename = "PatternFront")]
    pub pattern_front: String,

    #[sqlx(rename = "PatternBack")]
    pub pattern_back: String,

    #[sqlx(rename = "PlayDate")]
    pub play_date: Option<String>,

    #[sqlx(rename = "IsWin")]
    pub is_win: Option<i64>,
}

/// Template pattern - "TemplatePatternHitGame" table
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TemplatePatternHitGame {
    #[sqlx(rename = "Id")]
    pub id: i64,

    #[sqlx(rename = "GameCode")]
    pub game_code: String,

    #[sqlx(rename = "PatternFront")]
    pub pattern_front: String,

    #[sqlx(rename = "PatternBack")]
    pub pattern_back: String,
}

/// Create PatternHitGameQueue input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePatternHitGameQueue {
    pub game_code: String,
    pub pattern_front: String,
    pub pattern_back: String,
}

/// Create PatternHitGameHistory input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePatternHitGameHistory {
    pub game_code: String,
    pub pattern_front: String,
    pub pattern_back: String,
    pub play_date: Option<String>,
    pub is_win: Option<i64>,
}

/// Create TemplatePatternHitGame input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTemplatePatternHitGame {
    pub game_code: String,
    pub pattern_front: String,
    pub pattern_back: String,
}
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct DataPlaying {
    #[sqlx(rename = "Id")]
    #[serde(rename = "Id")]
    pub id: i64,
    #[sqlx(rename = "GameCode")]
    #[serde(rename = "GameCode")]
    pub game_code: String,
    #[sqlx(rename = "Digit")]
    #[serde(rename = "Digit")]
    pub digit: String, // "text" in SQL, string in Rust
    #[sqlx(rename = "Tipe")]
    #[serde(rename = "Tipe")]
    pub tipe: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDataPlaying {
    pub game_code: String,
    pub digit: String,
    pub tipe: String,
}

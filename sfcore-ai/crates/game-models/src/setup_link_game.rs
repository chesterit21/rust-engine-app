use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Setup link game - "SetupLinkGame" table
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SetupLinkGame {
    #[sqlx(rename = "Id")]
    pub id: i64,

    #[sqlx(rename = "LinkGame")]
    pub link_game: String,

    #[sqlx(rename = "LinkType")]
    pub link_type: String,

    #[sqlx(rename = "GameCode")]
    pub game_code: Option<String>,
}

/// Create SetupLinkGame input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSetupLinkGame {
    pub link_game: String,
    pub link_type: String,
    pub game_code: Option<String>,
}

/// Update SetupLinkGame input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSetupLinkGame {
    pub id: i64,
    pub link_game: String,
    pub link_type: String,
    pub game_code: Option<String>,
}
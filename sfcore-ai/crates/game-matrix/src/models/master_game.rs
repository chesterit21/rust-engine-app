use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MasterGame {
    pub id: i64,
    pub game_code: String,
    pub game_name: String,
    pub game_type: String,
    pub status: String,
    pub min_bet: i64,
    pub max_bet: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Input validation untuk create/update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMasterGame {
    pub game_code: String,
    pub game_name: String,
    pub game_type: String,
    pub min_bet: i64,
    pub max_bet: i64,
}

impl CreateMasterGame {
    pub fn validate(&self) -> Result<(), String> {
        if self.game_code.trim().is_empty() {
            return Err("game_code cannot be empty".into());
        }
        if self.min_bet < 0 || self.max_bet < 0 {
            return Err("bet amounts must be non-negative".into());
        }
        if self.min_bet > self.max_bet {
            return Err("min_bet cannot be greater than max_bet".into());
        }
        Ok(())
    }
}
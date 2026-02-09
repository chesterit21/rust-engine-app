use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LogGame {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    pub game_code: String,
    pub periode: i32,
    pub log_result: String,
    pub date_result_in_game: String,
    pub as_: i32,
    pub kop: i32,
    pub kepala: i32,
    pub ekor: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MasterGame {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    pub game_code: String,
    pub last_periode_in_real_game: i32,
    pub last_result: String,
    pub input_result_date: DateTime<Utc>,
    pub date_result: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub game_hour: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub game_minute: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_bet_hour: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_bet_minute: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SetupLinkGame {
    pub id: i64,
    pub link_type: String,
    pub link_game: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub game_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AnalisisPatternResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    pub game_code: String,
    pub index_same_result: i32,
}

impl LogGame {
    pub fn new(
        game_code: String,
        periode: i32,
        log_result: String,
        date_result_in_game: String,
    ) -> Self {
        let (as_, kop, kepala, ekor) = Self::parse_result(&log_result);
        
        Self {
            id: None,
            game_code,
            periode,
            log_result,
            date_result_in_game,
            as_,
            kop,
            kepala,
            ekor,
            created_at: Some(Utc::now()),
        }
    }

    /// Parse log result to extract As, Kop, Kepala, Ekor
    /// Example: "1234" -> as=1, kop=2, kepala=3, ekor=4
    fn parse_result(result: &str) -> (i32, i32, i32, i32) {
        let chars: Vec<char> = result.chars().collect();
        
        if chars.len() < 4 {
            return (0, 0, 0, 0);
        }

        let as_ = chars[0].to_digit(10).unwrap_or(0) as i32;
        let kop = chars[1].to_digit(10).unwrap_or(0) as i32;
        let kepala = chars[2].to_digit(10).unwrap_or(0) as i32;
        let ekor = chars[3].to_digit(10).unwrap_or(0) as i32;

        (as_, kop, kepala, ekor)
    }
}

impl MasterGame {
    pub fn new(game_code: String, periode: i32, result: String, date_result: String) -> Self {
        Self {
            id: None,
            game_code,
            last_periode_in_real_game: periode,
            last_result: result,
            input_result_date: Utc::now(),
            date_result,
            game_hour: None,
            game_minute: None,
            start_bet_hour: None,
            start_bet_minute: None,
        }
    }
}
use serde::{Deserialize, Serialize};
use validator::Validate;
use game_models::HistoryPlayingGame;

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct PlaceBetRequest {
    #[validate(length(min = 1, max = 50))]
    pub player_name: String,

    #[validate(length(min = 1, max = 10))]
    pub number: String,

    #[validate(range(min = 1))]
    pub template_number_id: i64,

    #[validate(length(min = 1, max = 20))]
    pub type_pick: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlaceBetResponse {
    pub trans_code: String,
    pub game_code: String,
    pub player_name: String,
    pub number: String,
    pub created_at: String,
}

impl From<HistoryPlayingGame> for PlaceBetResponse {
    fn from(hist: HistoryPlayingGame) -> Self {
        Self {
            trans_code: hist.trans_code,
            game_code: hist.game_code,
            player_name: hist.created_by,
            number: hist.number,
            created_at: hist.created_date,
        }
    }
}
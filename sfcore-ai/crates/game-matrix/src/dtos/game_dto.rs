use serde::{Deserialize, Serialize};
use validator::Validate;
use game_models::{MasterGame, LogGame};

#[derive(Debug, Clone, Serialize)]
pub struct GameWithLogsResponse {
    pub master: MasterGame,
    pub logs: Vec<LogGame>,
    pub pagination: PaginationMeta,
}

#[derive(Debug, Clone, Serialize)]
pub struct PaginationMeta {
    pub page: u32,
    pub page_size: u32,
    pub total_items: i64,
    pub total_pages: u32,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct GameQueryParams {
    #[validate(range(min = 1, max = 100))]
    pub page: Option<u32>,

    #[validate(range(min = 1, max = 100))]
    pub page_size: Option<u32>,
}

impl Default for GameQueryParams {
    fn default() -> Self {
        Self {
            page: Some(1),
            page_size: Some(20),
        }
    }
}
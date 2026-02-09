use std::sync::Arc;
use chrono::Utc;
use rand::Rng;
use game_models::{
    CreateHistoryPlayingGame,
    HistoryPlayingGame,
};
use game_repository::{
    MasterGameRepository,
    LogGameRepository,
    HistoryPlayingGameRepository,
    PatternHitRepository,
};
use crate::errors::GameResult;

pub struct PlayService {
    master_repo: Arc<MasterGameRepository>,
    _log_repo: Arc<LogGameRepository>,
    history_repo: Arc<HistoryPlayingGameRepository>,
    _pattern_repo: Arc<PatternHitRepository>,
}

impl PlayService {
    pub fn new(
        master_repo: MasterGameRepository,
        log_repo: LogGameRepository,
        history_repo: HistoryPlayingGameRepository,
        pattern_repo: PatternHitRepository,
    ) -> Self {
        Self {
            master_repo: Arc::new(master_repo),
            _log_repo: Arc::new(log_repo),
            history_repo: Arc::new(history_repo),
            _pattern_repo: Arc::new(pattern_repo),
        }
    }

    /// Player places a bet → create HistoryPlayingGame record
    pub async fn place_bet(
        &self,
        game_code: &str,
        player_name: &str,
        number: &str,
        template_number_id: i64,
        type_pick: &str,
    ) -> GameResult<HistoryPlayingGame> {
        // 1. Validate game exists
        let master = self.master_repo
            .find_by_game_code(game_code)
            .await?
            .ok_or_else(|| crate::errors::GameError::NotFound(format!("Game '{}' not found", game_code)))?;

        // 2. Generate unique transaction code (simple format: GAMECODE_TIMESTAMP_RANDOM)
        let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
        let random_suffix: u32 = rand::thread_rng().r#gen::<u16>() as u32;
        let trans_code = format!("{}{}_{}", master.game_code, timestamp, random_suffix);

        // 3. Create history record
        let history = self.history_repo
            .create(&CreateHistoryPlayingGame {
                trans_code: trans_code.clone(),
                game_id: master.id,
                game_code: game_code.to_string(),
                created_by: player_name.to_string(),
                created_date: Utc::now().to_rfc3339(),
                template_number_id,
                type_pick: type_pick.to_string(),
                number: number.to_string(),
            })
            .await?;

        Ok(history)
    }

    pub async fn get_history_summary(&self) -> GameResult<Vec<game_models::HistorySummary>> {
        let summary = self.history_repo
            .get_history_summary()
            .await?;
        Ok(summary)
    }

    pub async fn delete_history_by_game_code(&self, game_code: &str) -> GameResult<()> {
        self.history_repo
            .delete_by_game_code(game_code)
            .await?;
        Ok(())
    }

    pub async fn reset_all_data(&self) -> GameResult<()> {
        self.history_repo
            .truncate_all()
            .await?;
        Ok(())
    }

    pub async fn get_missing_numbers(&self, trans_code: &str) -> GameResult<String> {
        let numbers = self.history_repo
            .find_missing_numbers(trans_code)
            .await?;
        Ok(numbers)
    }
}
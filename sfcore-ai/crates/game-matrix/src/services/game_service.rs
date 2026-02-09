use std::sync::Arc;
use game_models::{MasterGame, LogGame, LogAnalysisResult};
use game_repository::{MasterGameRepository, LogGameRepository};
use chrono::{NaiveDate, Local};
use crate::errors::GameResult;

pub struct GameService {
    master_repo: Arc<MasterGameRepository>,
    log_repo: Arc<LogGameRepository>,
}

impl GameService {
    pub fn new(
        master_repo: MasterGameRepository,
        log_repo: LogGameRepository,
    ) -> Self {
        Self {
            master_repo: Arc::new(master_repo),
            log_repo: Arc::new(log_repo),
        }
    }

    /// Get MasterGame by GameCode + latest logs (paginated)
    pub async fn get_game_with_logs(
        &self,
        game_code: &str,
        page: u32,
        page_size: u32,
    ) -> GameResult<(MasterGame, Vec<LogGame>, i64)> {
        // 1. Get master game
        let master = self.master_repo
            .find_by_game_code(game_code)
            .await?
            .ok_or_else(|| crate::errors::GameError::NotFound(format!("Game '{}' not found", game_code)))?;

        // 2. Get logs with pagination
        let logs = self.log_repo
            .find_by_game_code(game_code, page_size, (page - 1) * page_size)
            .await?;

        // 3. Get total count for pagination metadata
        let total_logs = self.log_repo
            .count_by_game_code(game_code)
            .await?;

        Ok((master, logs, total_logs))
    }

    /// Get all active games (simple list)
    pub async fn get_all_games(&self) -> GameResult<Vec<MasterGame>> {
        self.master_repo.find_all().await.map_err(Into::into)
    }

    /// Get latest result for a game
    pub async fn get_latest_result(&self, game_code: &str) -> GameResult<Option<LogGame>> {
        self.log_repo
            .find_latest_by_game_code(game_code)
            .await
            .map_err(Into::into)
    }

    /// Get dashboard data with random trend
    /// Filter: exclude games where DateResult is more than 5 days old
    pub async fn get_dashboard_data(&self) -> GameResult<Vec<game_models::templates::DashboardGameResult>> {
        let all_games = self.master_repo.get_dashboard_active_games().await?;
        
        // Get current date for comparison
        let today = Local::now().date_naive();
        const MAX_DAYS_OLD: i64 = 5;
        
        // Filter games: exclude those with DateResult > 5 days old
        let mut games: Vec<_> = all_games
            .into_iter()
            .filter(|game| {
                // If no date_result, include the game (or exclude - depending on your preference)
                let Some(date_str) = &game.date_result else {
                    return false; // Exclude games without DateResult
                };
                
                // Try parsing common date formats
                // Format 1: "09 Feb 2026" or "9 Feb 2026"
                // Format 2: "2026-02-09"
                // Format 3: "09/02/2026"
                let parsed_date = Self::parse_date_result(date_str);
                
                match parsed_date {
                    Some(game_date) => {
                        let days_diff = (today - game_date).num_days();
                        days_diff <= MAX_DAYS_OLD // Include if within 5 days
                    }
                    None => {
                        // If we can't parse the date, include the game (safer default)
                        tracing::warn!("Cannot parse DateResult: '{}' for game '{}'", date_str, game.game_code);
                        true
                    }
                }
            })
            .collect();
        
        // Add random trend logic
        use rand::Rng;
        let mut rng = rand::thread_rng();

        for game in &mut games {
            let is_up: bool = rng.gen_bool(0.5); // 50/50 chance
            game.trend = Some(if is_up { "UP".to_string() } else { "DOWN".to_string() });
        }

        Ok(games)
    }
    
    /// Parse DateResult string to NaiveDate
    /// Supports multiple formats: "Senin, 09 Feb 2026", "09 Feb 2026", "2026-02-09", "09/02/2026"
    fn parse_date_result(date_str: &str) -> Option<NaiveDate> {
        let trimmed = date_str.trim();
        
        // Handle format: "Senin, 09 Feb 2026" - strip day name prefix
        let date_part = if trimmed.contains(',') {
            trimmed.split(',').nth(1).map(|s| s.trim()).unwrap_or(trimmed)
        } else {
            trimmed
        };
        
        // Try format: "09 Feb 2026" or "9 Feb 2026"
        if let Ok(date) = NaiveDate::parse_from_str(date_part, "%d %b %Y") {
            return Some(date);
        }
        
        // Try format: "09 February 2026"
        if let Ok(date) = NaiveDate::parse_from_str(date_part, "%d %B %Y") {
            return Some(date);
        }
        
        // Try format: "2026-02-09" (ISO format)
        if let Ok(date) = NaiveDate::parse_from_str(date_part, "%Y-%m-%d") {
            return Some(date);
        }
        
        // Try format: "09/02/2026" (DD/MM/YYYY)
        if let Ok(date) = NaiveDate::parse_from_str(date_part, "%d/%m/%Y") {
            return Some(date);
        }
        
        None
    }
    
    pub async fn get_game_analysis(&self, game_code: &str) -> GameResult<Vec<LogAnalysisResult>> {
        self.log_repo.find_analysis_by_game_code(game_code).await.map_err(Into::into)
    }
}
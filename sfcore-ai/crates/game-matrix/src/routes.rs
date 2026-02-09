use axum::Router;
use axum::routing::{get, post};
use crate::handlers::{
    auth::login,
    games::{get_all_games, get_game_by_code, get_latest_result, get_dashboard_games},
    plays::place_bet,
};
use crate::state::AppState;
use std::sync::Arc;

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        // Auth endpoints
        .route("/api/auth/login", post(login))
        
        // Game endpoints
        .route("/api/games", get(get_all_games))
        .route("/api/games/dashboard", get(get_dashboard_games))
        .route("/api/games/{game_code}", get(get_game_by_code))
        .route("/api/games/{game_code}/latest", get(get_latest_result))
        .route("/api/games/{game_code}/analysis", get(crate::handlers::analysis::get_log_analysis))
        .route("/api/games/{game_code}/frequency", get(crate::handlers::analysis::get_frequency_analysis))
        .route("/api/games/{game_code}/history-analysis", get(crate::handlers::analysis::get_history_analysis))

        // Player action endpoints
        .route("/api/games/{game_code}/play", post(place_bet))

        // History summary
        .route("/api/history/summary", get(crate::handlers::plays::get_history_summary))
        .route("/api/history/reset-all", axum::routing::delete(crate::handlers::plays::reset_all_data_handler))
        .route("/api/history/{game_code}", axum::routing::delete(crate::handlers::plays::delete_history_by_game_code))
        .route("/api/history/{trans_code}/missing", get(crate::handlers::plays::get_missing_numbers_handler))
        
        // Setup - Master Game
        .route("/api/setup/master-game", get(crate::handlers::setup::get_master_games))
        .route("/api/setup/master-game", post(crate::handlers::setup::create_master_game))
        .route("/api/setup/master-game", axum::routing::put(crate::handlers::setup::update_master_game))
        .route("/api/setup/master-game/{id}", axum::routing::delete(crate::handlers::setup::delete_master_game))
        
        // Setup - Member Game
        .route("/api/setup/member-game", get(crate::handlers::setup::get_member_games))
        .route("/api/setup/member-game", post(crate::handlers::setup::create_member_game))
        .route("/api/setup/member-game", axum::routing::put(crate::handlers::setup::update_member_game))
        .route("/api/setup/member-game/{id}", axum::routing::delete(crate::handlers::setup::delete_member_game))
        
        // Setup - Site Master
        .route("/api/setup/site-master", get(crate::handlers::setup::get_site_masters))
        .route("/api/setup/site-master", post(crate::handlers::setup::create_site_master))
        .route("/api/setup/site-master", axum::routing::put(crate::handlers::setup::update_site_master))
        .route("/api/setup/site-master/{id}", axum::routing::delete(crate::handlers::setup::delete_site_master))
        
        // Setup - Setup Link Game
        .route("/api/setup/link-game", get(crate::handlers::setup::get_link_games))
        .route("/api/setup/link-game", post(crate::handlers::setup::create_link_game))
        .route("/api/setup/link-game", axum::routing::put(crate::handlers::setup::update_link_game))
        .route("/api/setup/link-game/{id}", axum::routing::delete(crate::handlers::setup::delete_link_game))

        // Pattern Match
        .route("/api/games/save-pattern", post(crate::handlers::save_pattern::save_pattern_handler))

        // Attach state - dereference Arc to get AppState
        .with_state((*state).clone())
}
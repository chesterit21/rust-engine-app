use axum::{
    extract::{Path, Query, State},
    Json,
};
use validator::Validate;
use crate::{
    state::AppState,
    dtos::{GameQueryParams, GameWithLogsResponse, PaginationMeta},
    errors::GameResult,
};

pub async fn get_all_games(
    State(state): State<AppState>,
) -> GameResult<Json<Vec<game_models::MasterGame>>> {
    let games = state.game_service.get_all_games().await?;
    Ok(Json(games))
}

pub async fn get_game_by_code(
    Path(game_code): Path<String>,
    Query(params): Query<GameQueryParams>,
    State(state): State<AppState>,
) -> GameResult<Json<GameWithLogsResponse>> {
    // Validate query params
    params.validate().map_err(|e| crate::errors::GameError::ValidationError(e.to_string()))?;

    let page = params.page.unwrap_or(1);
    let page_size = params.page_size.unwrap_or(20);

    // Get game + logs
    let (master, logs, total_logs) = state
        .game_service
        .get_game_with_logs(&game_code, page, page_size)
        .await?;

    // Build pagination metadata
    let total_pages = ((total_logs as f64) / (page_size as f64)).ceil() as u32;

    let response = GameWithLogsResponse {
        master,
        logs,
        pagination: PaginationMeta {
            page,
            page_size,
            total_items: total_logs,
            total_pages,
        },
    };

    Ok(Json(response))
}

pub async fn get_latest_result(
    Path(game_code): Path<String>,
    State(state): State<AppState>,
) -> GameResult<Json<Option<game_models::LogGame>>> {
    let latest = state.game_service.get_latest_result(&game_code).await?;
    Ok(Json(latest))
}

pub async fn get_dashboard_games(
    State(state): State<AppState>,
) -> GameResult<Json<Vec<game_models::templates::DashboardGameResult>>> {
    let games = state.game_service.get_dashboard_data().await?;
    Ok(Json(games))
}
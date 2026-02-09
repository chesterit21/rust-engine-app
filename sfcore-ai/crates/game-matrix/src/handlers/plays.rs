use axum::{
    extract::{Path, State},
    Json,
};
use validator::Validate;
use crate::{
    state::AppState,
    dtos::{PlaceBetRequest, PlaceBetResponse},
    errors::GameResult,
};

pub async fn place_bet(
    Path(game_code): Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<PlaceBetRequest>,
) -> GameResult<Json<PlaceBetResponse>> {
    // Validate input
    payload.validate().map_err(|e| crate::errors::GameError::ValidationError(e.to_string()))?;

    // Execute business logic
    let history = state.play_service
        .place_bet(
            &game_code,
            &payload.player_name,
            &payload.number,
            payload.template_number_id,
            &payload.type_pick,
        )
        .await?;

    // Map to response DTO
    let response = PlaceBetResponse::from(history);
    Ok(Json(response))
}

pub async fn get_history_summary(
    State(state): State<AppState>,
) -> GameResult<Json<Vec<game_models::HistorySummary>>> {
    let summary = state.play_service.get_history_summary().await?;
    Ok(Json(summary))
}

pub async fn delete_history_by_game_code(
    Path(game_code): Path<String>,
    State(state): State<AppState>,
) -> GameResult<Json<serde_json::Value>> {
    state.play_service.delete_history_by_game_code(&game_code).await?;
    Ok(Json(serde_json::json!({ "status": "success" })))
}

pub async fn reset_all_data_handler(
    State(state): State<AppState>,
) -> GameResult<Json<serde_json::Value>> {
    state.play_service.reset_all_data().await?;
    Ok(Json(serde_json::json!({ "status": "success", "message": "All transaction data reset." })))
}

pub async fn get_missing_numbers_handler(
    Path(trans_code): Path<String>,
    State(state): State<AppState>,
) -> GameResult<Json<serde_json::Value>> {
    let missing_numbers = state.play_service.get_missing_numbers(&trans_code).await?;
    Ok(Json(serde_json::json!({ "trans_code": trans_code, "missing": missing_numbers })))
}
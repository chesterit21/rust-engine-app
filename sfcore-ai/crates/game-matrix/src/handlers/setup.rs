use axum::{
    extract::{Path, State},
    Json,
};
use game_models::*;
use crate::{
    state::AppState,
    errors::GameResult,
};

// --- Master Game ---
pub async fn get_master_games(
    State(state): State<AppState>,
) -> GameResult<Json<Vec<MasterGame>>> {
    let games = state.setup_service.get_all_master_games().await?;
    Ok(Json(games))
}

pub async fn create_master_game(
    State(state): State<AppState>,
    Json(payload): Json<CreateMasterGame>,
) -> GameResult<Json<MasterGame>> {
    let game = state.setup_service.create_master_game(payload).await?;
    Ok(Json(game))
}

pub async fn update_master_game(
    State(state): State<AppState>,
    Json(payload): Json<UpdateMasterGame>,
) -> GameResult<Json<MasterGame>> {
    let game = state.setup_service.update_master_game(payload).await?;
    Ok(Json(game))
}

pub async fn delete_master_game(
    Path(id): Path<i64>,
    State(state): State<AppState>,
) -> GameResult<Json<serde_json::Value>> {
    state.setup_service.delete_master_game(id).await?;
    Ok(Json(serde_json::json!({ "status": "success" })))
}

// --- Member Game ---
pub async fn get_member_games(
    State(state): State<AppState>,
) -> GameResult<Json<Vec<MemberGame>>> {
    let members = state.setup_service.get_all_member_games().await?;
    Ok(Json(members))
}

pub async fn create_member_game(
    State(state): State<AppState>,
    Json(payload): Json<CreateMemberGame>,
) -> GameResult<Json<MemberGame>> {
    let member = state.setup_service.create_member_game(payload).await?;
    Ok(Json(member))
}

pub async fn update_member_game(
    State(state): State<AppState>,
    Json(payload): Json<UpdateMemberGame>,
) -> GameResult<Json<MemberGame>> {
    let member = state.setup_service.update_member_game(payload).await?;
    Ok(Json(member))
}

pub async fn delete_member_game(
    Path(id): Path<i64>,
    State(state): State<AppState>,
) -> GameResult<Json<serde_json::Value>> {
    state.setup_service.delete_member_game(id).await?;
    Ok(Json(serde_json::json!({ "status": "success" })))
}

// --- Site Master ---
pub async fn get_site_masters(
    State(state): State<AppState>,
) -> GameResult<Json<Vec<SiteMaster>>> {
    let sites = state.setup_service.get_all_site_masters().await?;
    Ok(Json(sites))
}

pub async fn create_site_master(
    State(state): State<AppState>,
    Json(payload): Json<CreateSiteMaster>,
) -> GameResult<Json<SiteMaster>> {
    let site = state.setup_service.create_site_master(payload).await?;
    Ok(Json(site))
}

pub async fn update_site_master(
    State(state): State<AppState>,
    Json(payload): Json<UpdateSiteMaster>,
) -> GameResult<Json<SiteMaster>> {
    let site = state.setup_service.update_site_master(payload).await?;
    Ok(Json(site))
}

pub async fn delete_site_master(
    Path(id): Path<i64>,
    State(state): State<AppState>,
) -> GameResult<Json<serde_json::Value>> {
    state.setup_service.delete_site_master(id).await?;
    Ok(Json(serde_json::json!({ "status": "success" })))
}

// --- Setup Link Game ---
pub async fn get_link_games(
    State(state): State<AppState>,
) -> GameResult<Json<Vec<SetupLinkGame>>> {
    let links = state.setup_service.get_all_link_games().await?;
    Ok(Json(links))
}

pub async fn create_link_game(
    State(state): State<AppState>,
    Json(payload): Json<CreateSetupLinkGame>,
) -> GameResult<Json<SetupLinkGame>> {
    let link = state.setup_service.create_link_game(payload).await?;
    Ok(Json(link))
}

pub async fn update_link_game(
    State(state): State<AppState>,
    Json(payload): Json<UpdateSetupLinkGame>,
) -> GameResult<Json<SetupLinkGame>> {
    let link = state.setup_service.update_link_game(payload).await?;
    Ok(Json(link))
}

pub async fn delete_link_game(
    Path(id): Path<i64>,
    State(state): State<AppState>,
) -> GameResult<Json<serde_json::Value>> {
    state.setup_service.delete_link_game(id).await?;
    Ok(Json(serde_json::json!({ "status": "success" })))
}

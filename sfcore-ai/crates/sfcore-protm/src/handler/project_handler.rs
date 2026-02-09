use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, delete},
    Json, Router,
};
use crate::{service::ProjectService, domain::Project, AppState};
use crate::dto::*;
use std::sync::Arc;

pub async fn get_all(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Project>>, (StatusCode, String)> {
    ProjectService::get_all(&state.db)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub async fn create(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateProjectDto>,
) -> Result<Json<Project>, (StatusCode, String)> {
    ProjectService::create(&state.db, payload.name, payload.description)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub async fn update(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateProjectDto>,
) -> Result<Json<Project>, (StatusCode, String)> {
    ProjectService::update(&state.db, id, payload.name, payload.description)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub async fn delete_item(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    ProjectService::delete(&state.db, &id)
        .await
        .map(|count| {
            if count > 0 {
                StatusCode::NO_CONTENT
            } else {
                StatusCode::NOT_FOUND
            }
        })
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(get_all).post(create))
        .route("/{id}", delete(delete_item).put(update))
}

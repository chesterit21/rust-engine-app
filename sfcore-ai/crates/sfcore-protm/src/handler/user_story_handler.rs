use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, delete},
    Json, Router,
};
use crate::{service::UserStoryService, domain::UserStory, AppState};
use std::sync::Arc;

pub async fn get_all(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<UserStory>>, (StatusCode, String)> {
    UserStoryService::get_all(&state.db)
        .await
        .map(Json)
        .map_err(|e: sqlx::Error| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub async fn delete_item(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    UserStoryService::delete(&state.db, &id)
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
        .route("/", get(get_all))
        .route("/{id}", delete(delete_item))
}

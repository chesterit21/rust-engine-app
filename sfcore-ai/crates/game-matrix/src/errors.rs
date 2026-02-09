use axum::{
    response::{IntoResponse, Response},
    http::StatusCode,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GameError {
    #[error("Game not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[allow(dead_code)]
    #[error("Internal server error: {0}")]
    InternalError(String),
}

impl IntoResponse for GameError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            GameError::NotFound(ref msg) => (StatusCode::NOT_FOUND, msg.to_string()),
            GameError::ValidationError(ref msg) => (StatusCode::BAD_REQUEST, msg.to_string()),
            GameError::DatabaseError(ref e) => {
                tracing::error!("Database error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error".into())
            }
            GameError::InternalError(ref msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".into())
            }
        };

        let body = json!({
            "error": {
                "code": status.as_u16(),
                "message": error_message,
                "type": format!("{:?}", self),
            }
        });

        (status, axum::Json(body)).into_response()
    }
}

pub type GameResult<T> = Result<T, GameError>;
//! OpenAI-style error responses

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

#[derive(Debug, Serialize)]
pub struct ErrorDetail {
    pub message: String,
    #[serde(rename = "type")]
    pub error_type: String,
    pub code: String,
}

impl ErrorResponse {
    pub fn new(message: impl Into<String>, error_type: impl Into<String>, code: impl Into<String>) -> Self {
        Self {
            error: ErrorDetail {
                message: message.into(),
                error_type: error_type.into(),
                code: code.into(),
            },
        }
    }
    
    pub fn invalid_api_key() -> Self {
        Self::new(
            "Invalid API key provided",
            "authentication_error",
            "invalid_api_key",
        )
    }
    
    pub fn missing_api_key() -> Self {
        Self::new(
            "Missing authentication",
            "authentication_error",
            "missing_api_key",
        )
    }
    
    pub fn missing_client_app() -> Self {
        Self::new(
            "Missing client identifier",
            "authentication_error",
            "missing_client_app",
        )
    }
    
    pub fn invalid_client_app(client: &str) -> Self {
        Self::new(
            format!("Client '{}' not allowed", client),
            "authentication_error",
            "invalid_client_app",
        )
    }
    
    pub fn rate_limit_exceeded(retry_after: u64) -> Self {
        Self::new(
            format!("Rate limit exceeded. Try again in {} seconds", retry_after),
            "rate_limit_error",
            "rate_limit_exceeded",
        )
    }
    
    pub fn invalid_request(msg: impl Into<String>) -> Self {
        Self::new(
            msg,
            "invalid_request_error",
            "invalid_request",
        )
    }
    
    pub fn server_error(msg: impl Into<String>) -> Self {
        Self::new(
            msg,
            "server_error",
            "internal_error",
        )
    }
}

pub enum AppError {
    Unauthorized(ErrorResponse),
    RateLimited(ErrorResponse),
    BadRequest(ErrorResponse),
    InternalError(ErrorResponse),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_response) = match self {
            AppError::Unauthorized(err) => (StatusCode::UNAUTHORIZED, err),
            AppError::RateLimited(err) => (StatusCode::TOO_MANY_REQUESTS, err),
            AppError::BadRequest(err) => (StatusCode::BAD_REQUEST, err),
            AppError::InternalError(err) => (StatusCode::INTERNAL_SERVER_ERROR, err),
        };
        
        (status, Json(error_response)).into_response()
    }
}


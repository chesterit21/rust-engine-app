use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use crate::auth::jwt::Claims;
use std::sync::Arc;

// TODO: Implement full middleware once AppState is defined
pub struct AuthState {
    // pub jwt_manager: Arc<JwtManager>
}

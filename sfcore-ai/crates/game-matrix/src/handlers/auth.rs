use axum::{
    extract::State,
    Json,
    response::IntoResponse,
    http::StatusCode,
};
use validator::Validate;
use crate::{
    state::AppState,
    dtos::auth::{LoginRequest, LoginResponse},
    errors::GameResult,
};

pub async fn login(
    State(_state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> GameResult<impl IntoResponse> {
    // Validate payload
    if let Err(e) = payload.validate() {
        return Ok((StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e.to_string() }))).into_response());
    }

    // Hardcoded credentials logic
    let (valid, role) = match (payload.email.as_str(), payload.password.as_str()) {
        ("cecep@sfcore", "P@ssw0rd123") => (true, "USER"),
        ("adminapp@sfcore", "Admin#2026") => (true, "ADMIN"),
        ("msalih@sfcore", "Admin#123") => (true, "USER"),
        _ => (false, ""),
    };

    if !valid {
        return Ok((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "error": "Invalid username or password"
            }))
        ).into_response());
    }

    // Generate response (Mock token for now)
    let response = LoginResponse {
        token: format!("mock-jwt-token-for-{}", payload.email),
        username: payload.email,
        role: role.to_string(),
    };

    Ok((StatusCode::OK, Json(response)).into_response())
}

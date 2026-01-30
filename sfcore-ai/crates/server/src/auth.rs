//! Authentication and Rate Limiting middleware

use crate::config::AuthConfig;
use crate::error::{AppError, ErrorResponse};
use axum::{
    extract::{Request, State},
    http::HeaderMap,
    middleware::Next,
    response::Response,
};
use dashmap::DashMap;
use std::sync::Arc;
use std::time::Instant;

/// Shared auth state
#[derive(Clone)]
pub struct AuthState {
    pub api_key: String,
    pub allowed_clients: Vec<String>,
    pub rate_limiters: Arc<DashMap<String, RateLimiter>>,
    pub requests_per_minute: u32,
}

impl AuthState {
    pub fn new(config: &AuthConfig, requests_per_minute: u32) -> Self {
        Self {
            api_key: config.api_key.clone(),
            allowed_clients: config.allowed_clients.clone(),
            rate_limiters: Arc::new(DashMap::new()),
            requests_per_minute,
        }
    }
}

/// Token bucket rate limiter
pub struct RateLimiter {
    tokens: f64,
    last_refill: Instant,
    capacity: f64,
    refill_rate: f64, // tokens per second
}

impl RateLimiter {
    fn new(capacity: u32) -> Self {
        Self {
            tokens: capacity as f64,
            last_refill: Instant::now(),
            capacity: capacity as f64,
            refill_rate: capacity as f64 / 60.0, // per minute -> per second
        }
    }
    
    fn try_consume(&mut self) -> Result<(), u64> {
        // Refill tokens based on time elapsed
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.capacity);
        self.last_refill = now;
        
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            Ok(())
        } else {
            // Calculate retry-after in seconds
            let tokens_needed = 1.0 - self.tokens;
            let retry_after = (tokens_needed / self.refill_rate).ceil() as u64;
            Err(retry_after)
        }
    }
}

/// Auth middleware
pub async fn auth_middleware(
    State(auth_state): State<AuthState>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // 1. Extract and validate Authorization header
    let auth_header = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized(ErrorResponse::missing_api_key()))?;
    
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| AppError::Unauthorized(ErrorResponse::invalid_api_key()))?;
    
    if token != auth_state.api_key {
        return Err(AppError::Unauthorized(ErrorResponse::invalid_api_key()));
    }
    
    // 2. Extract and validate X-Client-App header
    let client_app = headers
        .get("x-client-app")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized(ErrorResponse::missing_client_app()))?;
    
    if !auth_state.allowed_clients.contains(&client_app.to_string()) {
        return Err(AppError::Unauthorized(ErrorResponse::invalid_client_app(client_app)));
    }
    
    // 3. Rate limiting (per API key)
    let mut limiter = auth_state
        .rate_limiters
        .entry(token.to_string())
        .or_insert_with(|| RateLimiter::new(auth_state.requests_per_minute));
    
    if let Err(retry_after) = limiter.try_consume() {
        return Err(AppError::RateLimited(ErrorResponse::rate_limit_exceeded(retry_after)));
    }
    
    Ok(next.run(request).await)
}

use crate::security::{CustomHeaderValidator, IpWhitelist};
use crate::utils::error::ApiError;
use axum::{
    extract::{ConnectInfo, Request},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{debug, warn};

/// Security middleware - check IP whitelist dan custom headers
pub async fn security_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let ip = addr.ip();
    debug!("Incoming request from IP: {}", ip);
    
    // Get shared state dari extensions
    let ip_whitelist = request
        .extensions()
        .get::<Arc<IpWhitelist>>()
        .ok_or_else(|| ApiError::InternalError("IP whitelist not configured".to_string()))?
        .clone();
    
    let header_validator = request
        .extensions()
        .get::<Arc<CustomHeaderValidator>>()
        .ok_or_else(|| ApiError::InternalError("Header validator not configured".to_string()))?
        .clone();
    
    // 1. Check IP whitelist
    if !ip_whitelist.is_allowed(ip).await {
        warn!("Request from non-whitelisted IP: {}", ip);
        return Err(ApiError::Forbidden(format!(
            "Access denied from IP: {}",
            ip
        )));
    }
    
    debug!("IP {} is whitelisted", ip);
    
    // 2. Validate custom headers
    let headers = request.headers();
    let validated = header_validator.validate(headers)?;
    
    debug!(
        "Request validated: app_id={}, timestamp={}",
        validated.app_id, validated.timestamp
    );
    
    // Continue to next middleware/handler
    Ok(next.run(request).await)
}

/// Health check middleware (bypass security)
pub async fn health_check_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Health check tidak perlu security check
    Ok(next.run(request).await)
}

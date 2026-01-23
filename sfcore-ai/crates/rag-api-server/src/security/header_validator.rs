use crate::utils::error::ApiError;
use axum::http::HeaderMap;
use chrono::Utc;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use tracing::{debug, warn};

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone)]
pub struct CustomHeaderValidator {
    pub expected_app_id: String,
    pub expected_api_key: String,
    pub signature_enabled: bool,
    pub timestamp_tolerance: i64, // seconds
}

impl CustomHeaderValidator {
    pub fn new(
        app_id: String,
        api_key: String,
        signature_enabled: bool,
        timestamp_tolerance: i64,
    ) -> Self {
        Self {
            expected_app_id: app_id,
            expected_api_key: api_key,
            signature_enabled,
            timestamp_tolerance,
        }
    }
    
    /// Validate custom headers
    pub fn validate(&self, headers: &HeaderMap) -> Result<ValidatedRequest, ApiError> {
        // 1. Check X-App-ID
        let app_id = headers
            .get("X-App-ID")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| ApiError::Unauthorized("Missing X-App-ID header".to_string()))?;
        
        if app_id != self.expected_app_id {
            warn!("Invalid X-App-ID: expected {}, got {}", self.expected_app_id, app_id);
            return Err(ApiError::Unauthorized("Invalid X-App-ID".to_string()));
        }
        
        // 2. Check X-API-Key
        let api_key = headers
            .get("X-API-Key")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| ApiError::Unauthorized("Missing X-API-Key header".to_string()))?;
        
        if api_key != self.expected_api_key {
            warn!("Invalid X-API-Key");
            return Err(ApiError::Unauthorized("Invalid X-API-Key".to_string()));
        }
        
        // 3. Check X-Request-Timestamp (untuk prevent replay attacks)
        let timestamp = headers
            .get("X-Request-Timestamp")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<i64>().ok())
            .ok_or_else(|| {
                ApiError::Unauthorized("Missing or invalid X-Request-Timestamp header".to_string())
            })?;
        
        // Validate timestamp (not too old or future)
        let now = Utc::now().timestamp();
        let diff = (now - timestamp).abs();
        
        if diff > self.timestamp_tolerance {
            warn!("Timestamp too old/future: {} seconds difference", diff);
            return Err(ApiError::Unauthorized(
                "Request timestamp out of tolerance window".to_string()
            ));
        }
        
        // 4. Optional: Check X-Request-Signature (HMAC)
        if self.signature_enabled {
            let signature = headers
                .get("X-Request-Signature")
                .and_then(|v| v.to_str().ok())
                .ok_or_else(|| {
                    ApiError::Unauthorized("Missing X-Request-Signature header".to_string())
                })?;
            
            // Verify HMAC signature
            // Format: HMAC-SHA256(api_key, app_id + timestamp)
            let message = format!("{}{}", app_id, timestamp);
            
            if !self.verify_signature(&message, signature)? {
                warn!("Invalid request signature");
                return Err(ApiError::Unauthorized("Invalid signature".to_string()));
            }
        }
        
        debug!("Headers validated successfully");
        
        Ok(ValidatedRequest {
            app_id: app_id.to_string(),
            timestamp,
        })
    }
    
    /// Verify HMAC signature
    fn verify_signature(&self, message: &str, signature: &str) -> Result<bool, ApiError> {
        let mut mac = HmacSha256::new_from_slice(self.expected_api_key.as_bytes())
            .map_err(|e| ApiError::InternalError(format!("HMAC error: {}", e)))?;
        
        mac.update(message.as_bytes());
        
        let expected = hex::encode(mac.finalize().into_bytes());
        
        Ok(expected.eq_ignore_ascii_case(signature))
    }
    
    /// Generate signature (untuk testing atau dokumentasi)
    pub fn generate_signature(&self, app_id: &str, timestamp: i64) -> Result<String, ApiError> {
        let message = format!("{}{}", app_id, timestamp);
        
        let mut mac = HmacSha256::new_from_slice(self.expected_api_key.as_bytes())
            .map_err(|e| ApiError::InternalError(format!("HMAC error: {}", e)))?;
        
        mac.update(message.as_bytes());
        
        Ok(hex::encode(mac.finalize().into_bytes()))
    }
}

#[derive(Debug, Clone)]
pub struct ValidatedRequest {
    pub app_id: String,
    pub timestamp: i64,
}

//! JWT token handling

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum JwtError {
    #[error("Token creation failed: {0}")]
    CreationError(String),
    #[error("Token validation failed: {0}")]
    ValidationError(String),
    #[error("Token expired")]
    TokenExpired,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub iat: i64,
    pub exp: i64,
    pub token_type: String,
}

pub struct JwtService {
    secret: String,
    access_token_expiry: i64,
    refresh_token_expiry: i64,
}

impl JwtService {
    pub fn new(secret: String, access_expiry: i64, refresh_expiry: i64) -> Self {
        Self {
            secret,
            access_token_expiry: access_expiry,
            refresh_token_expiry: refresh_expiry,
        }
    }

    pub fn generate_access_token(&self, user_id: &Uuid) -> Result<String, JwtError> {
        self.generate_token(user_id, "access", self.access_token_expiry)
    }

    pub fn generate_refresh_token(&self, user_id: &Uuid) -> Result<String, JwtError> {
        self.generate_token(user_id, "refresh", self.refresh_token_expiry)
    }

    fn generate_token(&self, user_id: &Uuid, token_type: &str, expiry: i64) -> Result<String, JwtError> {
        let now = Utc::now();
        let claims = Claims {
            sub: user_id.to_string(),
            iat: now.timestamp(),
            exp: (now + Duration::seconds(expiry)).timestamp(),
            token_type: token_type.to_string(),
        };
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| JwtError::CreationError(e.to_string()))
    }

    pub fn validate_token(&self, token: &str) -> Result<Claims, JwtError> {
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::default(),
        )
        .map(|data| data.claims)
        .map_err(|e| JwtError::ValidationError(e.to_string()))
    }
}

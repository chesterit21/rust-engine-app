// ============================================================================
// SSO Core - Authentication Service
// File: crates/sso-core/src/services/auth_service.rs
// ============================================================================
//! Authentication service with login, register, and token management

use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;
use tracing::{info, warn, error};

use crate::domain::{MemberUser, MemberStatus};
use crate::error::DomainError;
use crate::repositories::UserRepository;

/// Authentication service for handling user login/register flows
pub struct AuthService<R: UserRepository> {
    user_repo: Arc<R>,
    jwt_secret: String,
    jwt_access_expiry_hours: i64,
    jwt_refresh_expiry_hours: i64,
}

impl<R: UserRepository> AuthService<R> {
    pub fn new(
        user_repo: Arc<R>, 
        jwt_secret: String, 
        jwt_access_expiry_hours: i64,
        jwt_refresh_expiry_hours: i64,
    ) -> Self {
        Self {
            user_repo,
            jwt_secret,
            jwt_access_expiry_hours,
            jwt_refresh_expiry_hours,
        }
    }
    
    /// Login with email and password
    pub async fn login(
        &self,
        email: &str,
        password: &str,
    ) -> Result<LoginResult, DomainError> {
        info!("Login attempt for email: {}", email);
        
        // 1. Find user by email
        let user = self.user_repo.find_by_email(email).await?
            .ok_or_else(|| {
                warn!("Login failed: email not found: {}", email);
                DomainError::InvalidCredentials
            })?;
        
        // 2. Check if user can login
        if !user.can_login() {
            warn!("Login failed: user cannot login (status: {:?})", user.status_member);
            return Err(DomainError::UserNotActive);
        }
        
        // 3. Verify password
        let stored_hash = user.password.as_ref()
            .ok_or(DomainError::InvalidCredentials)?;
        
        let password_valid = sso_security::password::PasswordService::verify(password, stored_hash)
            .map_err(|_e| DomainError::InvalidCredentials)?;
        
        if !password_valid {
            warn!("Login failed: invalid password for: {}", email);
            return Err(DomainError::InvalidCredentials);
        }
        
        // 4. Generate tokens using JwtService
        let jwt_service = sso_security::jwt::JwtService::new(
            self.jwt_secret.clone(),
            self.jwt_access_expiry_hours * 3600, // Convert hours to seconds
            self.jwt_refresh_expiry_hours * 3600,
        );
        
        let access_token = jwt_service.generate_access_token(&user.id)
            .map_err(|e| DomainError::TokenGenerationError(e.to_string()))?;
        
        let refresh_token = jwt_service.generate_refresh_token(&user.id)
            .map_err(|e| DomainError::TokenGenerationError(e.to_string()))?;
        
        // 5. Update last login
        let mut updated_user = user.clone();
        updated_user.record_login();
        
        if let Err(e) = self.user_repo.update(&updated_user).await {
            error!("Failed to update last login: {}", e);
            // Don't fail login for this
        }
        
        info!("Login successful for: {}", email);
        
        Ok(LoginResult {
            user: UserInfo::from(&updated_user),
            access_token,
            refresh_token,
        })
    }
    
    /// Register a new user
    pub async fn register(
        &self,
        display_name: &str,
        email: &str,
        password: &str,
    ) -> Result<RegisterResult, DomainError> {
        info!("Registration attempt for email: {}", email);
        
        // 1. Check if email already exists
        if self.user_repo.find_by_email(email).await?.is_some() {
            warn!("Registration failed: email already exists: {}", email);
            return Err(DomainError::EmailAlreadyExists(email.to_string()));
        }
        
        // 2. Check display name uniqueness
        if self.user_repo.find_by_display_name(display_name).await?.is_some() {
            warn!("Registration failed: display name already exists: {}", display_name);
            return Err(DomainError::DisplayNameAlreadyExists(display_name.to_string()));
        }
        
        // 3. Hash password
        let password_hash = sso_security::password::PasswordService::hash(password)
            .map_err(|e| DomainError::PasswordHashError(e.to_string()))?;
        
        // 4. Create user entity (password is Option<String>)
        let user = MemberUser::new(
            display_name.to_string(), 
            email.to_string(), 
            Some(password_hash)
        ).map_err(|e| DomainError::ValidationError(e.to_string()))?;
        
        // 5. Save to database
        let created_user = self.user_repo.create(&user).await?;
        
        info!("Registration successful for: {}", email);
        
        Ok(RegisterResult {
            user: UserInfo::from(&created_user),
            requires_email_verification: true,
        })
    }
    
    /// Verify email with token
    pub async fn verify_email(&self, user_id: &Uuid) -> Result<(), DomainError> {
        let user = self.user_repo.find_by_id(user_id).await?
            .ok_or(DomainError::UserNotFound)?;
        
        let mut updated_user = user;
        updated_user.email_verified = true;
        updated_user.status_member = MemberStatus::Active;
        updated_user.is_active = true;
        updated_user.modified_at = Some(Utc::now());
        
        self.user_repo.update(&updated_user).await?;
        
        info!("Email verified for user: {}", user_id);
        Ok(())
    }
}

/// Result of successful login
#[derive(Debug, Clone)]
pub struct LoginResult {
    pub user: UserInfo,
    pub access_token: String,
    pub refresh_token: String,
}

/// Result of successful registration
#[derive(Debug, Clone)]
pub struct RegisterResult {
    pub user: UserInfo,
    pub requires_email_verification: bool,
}

/// User info returned in auth responses
#[derive(Debug, Clone)]
pub struct UserInfo {
    pub id: Uuid,
    pub display_name: String,
    pub email: String,
    pub email_verified: bool,
    pub status: String,
    pub profile_image: Option<String>,
}

impl From<&MemberUser> for UserInfo {
    fn from(user: &MemberUser) -> Self {
        Self {
            id: user.id,
            display_name: user.display_name.clone(),
            email: user.email.clone(),
            email_verified: user.email_verified,
            status: user.status_member.as_str().to_string(),
            profile_image: user.link_profile_image.clone(),
        }
    }
}

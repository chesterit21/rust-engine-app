//! Domain errors

use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum DomainError {
    #[error("User not found")]
    UserNotFound,
    
    #[error("User not found: {0}")]
    UserNotFoundById(String),
    
    #[error("User not active")]
    UserNotActive,
    
    #[error("Invalid credentials")]
    InvalidCredentials,
    
    #[error("Email already exists: {0}")]
    EmailAlreadyExists(String),
    
    #[error("Display name already exists: {0}")]
    DisplayNameAlreadyExists(String),
    
    #[error("Tenant not found")]
    TenantNotFound,
    
    #[error("Tenant name already exists: {0}")]
    TenantNameAlreadyExists(String),
    
    #[error("Tenant slug already exists: {0}")]
    TenantSlugAlreadyExists(String),
    
    #[error("Tenant not active")]
    TenantNotActive,
    
    #[error("Tenant max users reached")]
    TenantMaxUsersReached,
    
    #[error("User already in tenant")]
    UserAlreadyInTenant,
    
    #[error("Owner level {0} already exists")]
    OwnerLevelAlreadyExists(i32),
    
    #[error("Client app name already exists: {0}")]
    ClientAppNameAlreadyExists(String),
    
    #[error("Client app unique name already exists: {0}")]
    ClientAppUniqueNameAlreadyExists(String),
    
    #[error("Group name already exists in client app {client_app_id}: {name}")]
    GroupNameAlreadyExists { client_app_id: Uuid, name: String },
    
    #[error("Menu name already exists in client app {client_app_id}: {name}")]
    MenuNameAlreadyExists { client_app_id: Uuid, name: String },
    
    #[error("Password too short")]
    PasswordTooShort,
    
    #[error("Password too long")]
    PasswordTooLong,
    
    #[error("Password too weak")]
    PasswordTooWeak,
    
    #[error("Password hash error: {0}")]
    PasswordHashError(String),
    
    #[error("Token generation error: {0}")]
    TokenGenerationError(String),
    
    #[error("Unable to generate unique name")]
    UnableToGenerateUniqueName,
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}


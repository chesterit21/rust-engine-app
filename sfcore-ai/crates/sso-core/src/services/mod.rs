//! Domain services (business logic)

pub mod auth_service;
pub mod user_service;
pub mod tenant_service;

pub use auth_service::{AuthService, LoginResult, RegisterResult, UserInfo};

//! # SSO Core
//! 
//! Domain entities, services, and repository traits for SSO application.

pub mod domain;
pub mod services;
pub mod repositories;
pub mod error;

// Re-export domain entities
pub use domain::*;
pub use error::DomainError;

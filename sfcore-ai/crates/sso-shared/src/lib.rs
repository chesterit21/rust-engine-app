//! # SSO Shared
//! 
//! Shared utilities, types, and telemetry for the SSO application.

pub mod constants;
pub mod types;
pub mod utils;
pub mod telemetry;
pub mod config;
pub mod error;

pub use types::*;
pub use error::AppError;

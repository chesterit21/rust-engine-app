//! # SSO Infrastructure
//! 
//! Database and cache implementations (adapters).

pub mod database;
pub mod cache;

pub use database::{create_pool, PgUserRepository, PgTenantRepository, PgClientAppRepository};
pub use cache::RedisCache;

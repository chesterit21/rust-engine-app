//! Database module (PostgreSQL adapters)

pub mod connection;
pub mod postgres;

pub use connection::create_pool;
pub use postgres::{PgUserRepository, PgTenantRepository, PgClientAppRepository};

//! Repository traits (ports)

pub mod user_repository;
pub mod tenant_repository;

pub use user_repository::UserRepository;
pub use tenant_repository::TenantRepository;

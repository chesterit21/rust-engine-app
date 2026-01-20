//! Tenant repository trait (port)

use async_trait::async_trait;
use uuid::Uuid;
use crate::domain::TenantApp;
use crate::error::DomainError;

#[async_trait]
pub trait TenantRepository: Send + Sync {
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<TenantApp>, DomainError>;
    async fn find_by_name(&self, name: &str) -> Result<Option<TenantApp>, DomainError>;
    async fn find_by_slug(&self, slug: &str) -> Result<Option<TenantApp>, DomainError>;
    async fn create(&self, tenant: &TenantApp) -> Result<TenantApp, DomainError>;
    async fn update(&self, tenant: &TenantApp) -> Result<TenantApp, DomainError>;
}

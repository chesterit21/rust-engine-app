//! User repository trait (port)

use async_trait::async_trait;
use uuid::Uuid;
use crate::domain::MemberUser;
use crate::error::DomainError;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<MemberUser>, DomainError>;
    async fn find_by_email(&self, email: &str) -> Result<Option<MemberUser>, DomainError>;
    async fn find_by_display_name(&self, display_name: &str) -> Result<Option<MemberUser>, DomainError>;
    async fn create(&self, user: &MemberUser) -> Result<MemberUser, DomainError>;
    async fn update(&self, user: &MemberUser) -> Result<MemberUser, DomainError>;
    async fn delete(&self, id: &Uuid) -> Result<(), DomainError>;
}

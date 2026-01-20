// ============================================================================
// SSO Core - Member User Tenant App Entity
// File: crates/sso-core/src/domain/member_user_tenant_app.rs
// Description: User-Tenant relationship with roles
// ============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Tenant role enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TenantRole {
    Owner,
    Admin,
    Member,
    Guest,
}

impl TenantRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            TenantRole::Owner => "owner",
            TenantRole::Admin => "admin",
            TenantRole::Member => "member",
            TenantRole::Guest => "guest",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "owner" => Some(TenantRole::Owner),
            "admin" => Some(TenantRole::Admin),
            "member" => Some(TenantRole::Member),
            "guest" => Some(TenantRole::Guest),
            _ => None,
        }
    }
}

impl Default for TenantRole {
    fn default() -> Self {
        TenantRole::Member
    }
}

/// Member User Tenant App entity (User-Tenant Relationship)
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct MemberUserTenantApp {
    pub id: Uuid,
    pub tenant_app_id: Uuid,
    pub member_user_id: Uuid,
    pub is_owner: bool,
    
    #[validate(range(min = 1, message = "Level owner must be at least 1"))]
    pub level_owner: Option<i32>,
    
    pub role_in_tenant: TenantRole,
    pub joined_at: DateTime<Utc>,
    
    // Audit fields
    pub created_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub modified_at: Option<DateTime<Utc>>,
    pub modified_by: Option<Uuid>,
    pub removed_at: Option<DateTime<Utc>>,
    pub removed_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub approved_by: Option<Uuid>,
}

impl MemberUserTenantApp {
    pub fn new(
        tenant_app_id: Uuid,
        member_user_id: Uuid,
        role_in_tenant: TenantRole,
        created_by: Option<Uuid>,
    ) -> Result<Self, validator::ValidationErrors> {
        let relation = Self {
            id: Uuid::new_v4(),
            tenant_app_id,
            member_user_id,
            is_owner: false,
            level_owner: None,
            role_in_tenant,
            joined_at: Utc::now(),
            created_at: Utc::now(),
            created_by,
            modified_at: None,
            modified_by: None,
            removed_at: None,
            removed_by: None,
            approved_at: None,
            approved_by: None,
        };

        relation.validate()?;
        Ok(relation)
    }

    pub fn new_owner(
        tenant_app_id: Uuid,
        member_user_id: Uuid,
        level_owner: i32,
        created_by: Option<Uuid>,
    ) -> Result<Self, validator::ValidationErrors> {
        let relation = Self {
            id: Uuid::new_v4(),
            tenant_app_id,
            member_user_id,
            is_owner: true,
            level_owner: Some(level_owner),
            role_in_tenant: TenantRole::Owner,
            joined_at: Utc::now(),
            created_at: Utc::now(),
            created_by,
            modified_at: None,
            modified_by: None,
            removed_at: None,
            removed_by: None,
            approved_at: None,
            approved_by: None,
        };

        relation.validate()?;
        Ok(relation)
    }

    pub fn promote_to_owner(&mut self, level_owner: i32, promoted_by: Uuid) {
        self.is_owner = true;
        self.level_owner = Some(level_owner);
        self.role_in_tenant = TenantRole::Owner;
        self.modified_at = Some(Utc::now());
        self.modified_by = Some(promoted_by);
    }

    pub fn demote_from_owner(&mut self, new_role: TenantRole, demoted_by: Uuid) {
        self.is_owner = false;
        self.level_owner = None;
        self.role_in_tenant = new_role;
        self.modified_at = Some(Utc::now());
        self.modified_by = Some(demoted_by);
    }

    pub fn soft_delete(&mut self, deleted_by: Uuid) {
        self.removed_at = Some(Utc::now());
        self.removed_by = Some(deleted_by);
    }

    pub fn is_deleted(&self) -> bool {
        self.removed_at.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_member_relation() {
        let rel = MemberUserTenantApp::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            TenantRole::Member,
            None,
        );
        assert!(rel.is_ok());
        assert!(!rel.unwrap().is_owner);
    }

    #[test]
    fn test_create_owner_relation() {
        let rel = MemberUserTenantApp::new_owner(
            Uuid::new_v4(),
            Uuid::new_v4(),
            1,
            None,
        );
        assert!(rel.is_ok());
        let rel = rel.unwrap();
        assert!(rel.is_owner);
        assert_eq!(rel.level_owner, Some(1));
    }

    #[test]
    fn test_promote_demote() {
        let mut rel = MemberUserTenantApp::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            TenantRole::Member,
            None,
        ).unwrap();
        
        rel.promote_to_owner(1, Uuid::new_v4());
        assert!(rel.is_owner);
        
        rel.demote_from_owner(TenantRole::Admin, Uuid::new_v4());
        assert!(!rel.is_owner);
        assert_eq!(rel.role_in_tenant, TenantRole::Admin);
    }
}

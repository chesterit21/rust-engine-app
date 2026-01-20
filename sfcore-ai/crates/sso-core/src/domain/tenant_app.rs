// ============================================================================
// SSO Core - Tenant App Entity
// File: crates/sso-core/src/domain/tenant_app.rs
// Description: Tenant entity with subscription management
// ============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Subscription plan enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionPlan {
    Free,
    Basic,
    Premium,
    Enterprise,
}

impl SubscriptionPlan {
    pub fn as_str(&self) -> &'static str {
        match self {
            SubscriptionPlan::Free => "free",
            SubscriptionPlan::Basic => "basic",
            SubscriptionPlan::Premium => "premium",
            SubscriptionPlan::Enterprise => "enterprise",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "free" => Some(SubscriptionPlan::Free),
            "basic" => Some(SubscriptionPlan::Basic),
            "premium" => Some(SubscriptionPlan::Premium),
            "enterprise" => Some(SubscriptionPlan::Enterprise),
            _ => None,
        }
    }
}

impl Default for SubscriptionPlan {
    fn default() -> Self {
        SubscriptionPlan::Free
    }
}

/// Tenant App entity
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct TenantApp {
    pub id: Uuid,
    
    #[validate(length(min = 2, max = 100, message = "Tenant name must be between 2 and 100 characters"))]
    pub name: String,
    
    #[validate(length(max = 1000, message = "Description too long"))]
    pub description: Option<String>,
    
    #[validate(length(min = 2, max = 100, message = "Slug must be between 2 and 100 characters"))]
    pub slug: String,
    
    pub is_active: bool,
    
    #[validate(range(min = 1, max = 10000, message = "Max users must be between 1 and 10000"))]
    pub max_users: i32,
    
    pub subscription_plan: SubscriptionPlan,
    pub subscription_expires_at: Option<DateTime<Utc>>,
    
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

impl TenantApp {
    pub fn new(
        name: String,
        description: Option<String>,
        slug: String,
        max_users: i32,
        subscription_plan: SubscriptionPlan,
        created_by: Option<Uuid>,
    ) -> Result<Self, validator::ValidationErrors> {
        let tenant = Self {
            id: Uuid::new_v4(),
            name: name.trim().to_string(),
            description: description.map(|d| d.trim().to_string()),
            slug: slug.trim().to_lowercase(),
            is_active: true,
            max_users,
            subscription_plan,
            subscription_expires_at: None,
            created_at: Utc::now(),
            created_by,
            modified_at: None,
            modified_by: None,
            removed_at: None,
            removed_by: None,
            approved_at: None,
            approved_by: None,
        };

        tenant.validate()?;
        Ok(tenant)
    }

    pub fn is_subscription_active(&self) -> bool {
        if let Some(expires_at) = self.subscription_expires_at {
            expires_at > Utc::now()
        } else {
            true // No expiration means active
        }
    }

    pub fn upgrade_subscription(
        &mut self,
        plan: SubscriptionPlan,
        max_users: i32,
        expires_at: Option<DateTime<Utc>>,
        modified_by: Uuid,
    ) {
        self.subscription_plan = plan;
        self.max_users = max_users;
        self.subscription_expires_at = expires_at;
        self.modified_at = Some(Utc::now());
        self.modified_by = Some(modified_by);
    }

    pub fn soft_delete(&mut self, deleted_by: Uuid) {
        self.removed_at = Some(Utc::now());
        self.removed_by = Some(deleted_by);
        self.is_active = false;
    }

    pub fn is_deleted(&self) -> bool {
        self.removed_at.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_tenant() {
        let tenant = TenantApp::new(
            "Test Tenant".to_string(),
            Some("Description".to_string()),
            "test-tenant".to_string(),
            100,
            SubscriptionPlan::Basic,
            None,
        );
        assert!(tenant.is_ok());
    }

    #[test]
    fn test_subscription_active() {
        let tenant = TenantApp::new(
            "Test".to_string(),
            None,
            "test".to_string(),
            10,
            SubscriptionPlan::Free,
            None,
        ).unwrap();
        
        assert!(tenant.is_subscription_active());
    }
}

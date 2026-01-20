// ============================================================================
// SSO Core - Client App Entity
// File: crates/sso-core/src/domain/client_app.rs
// Description: Client application entity
// ============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Application type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AppType {
    Web,
    Mobile,
    Desktop,
    Api,
    Console,
}

impl AppType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AppType::Web => "web",
            AppType::Mobile => "mobile",
            AppType::Desktop => "desktop",
            AppType::Api => "api",
            AppType::Console => "console",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "web" => Some(AppType::Web),
            "mobile" => Some(AppType::Mobile),
            "desktop" => Some(AppType::Desktop),
            "api" => Some(AppType::Api),
            "console" => Some(AppType::Console),
            _ => None,
        }
    }
}

impl Default for AppType {
    fn default() -> Self {
        AppType::Web
    }
}

/// Client Application entity
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ClientApp {
    pub id: Uuid,
    
    #[validate(length(min = 3, max = 100, message = "Name must be between 3 and 100 characters"))]
    pub name: String,
    
    #[validate(length(max = 1000, message = "Description too long"))]
    pub description: Option<String>,
    
    #[validate(length(min = 3, max = 100, message = "Unique name must be between 3 and 100 characters"))]
    pub unique_name: String,
    
    pub type_app: AppType,
    
    #[validate(url(message = "Invalid application URL"))]
    #[validate(length(max = 2048, message = "URL too long"))]
    pub url_app: String,
    
    /// Hashed client secret
    pub client_secret: String,
    
    pub redirect_uris: Vec<String>,
    
    pub is_active: bool,
    
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

impl ClientApp {
    /// Create new client application
    pub fn new(
        name: String,
        description: Option<String>,
        unique_name: String,
        type_app: AppType,
        url_app: String,
        client_secret: String,
        redirect_uris: Vec<String>,
        created_by: Option<Uuid>,
    ) -> Result<Self, validator::ValidationErrors> {
        let app = Self {
            id: Uuid::new_v4(),
            name: name.trim().to_string(),
            description: description.map(|d| d.trim().to_string()),
            unique_name: unique_name.trim().to_lowercase(),
            type_app,
            url_app: url_app.trim().to_string(),
            client_secret,
            redirect_uris,
            is_active: true,
            created_at: Utc::now(),
            created_by,
            modified_at: None,
            modified_by: None,
            removed_at: None,
            removed_by: None,
            approved_at: None,
            approved_by: None,
        };

        app.validate()?;
        Ok(app)
    }

    /// Activate application
    pub fn activate(&mut self, activated_by: Uuid) {
        self.is_active = true;
        self.approved_at = Some(Utc::now());
        self.approved_by = Some(activated_by);
        self.modified_at = Some(Utc::now());
        self.modified_by = Some(activated_by);
    }

    /// Deactivate application
    pub fn deactivate(&mut self, deactivated_by: Uuid) {
        self.is_active = false;
        self.modified_at = Some(Utc::now());
        self.modified_by = Some(deactivated_by);
    }

    /// Soft delete application
    pub fn soft_delete(&mut self, deleted_by: Uuid) {
        self.removed_at = Some(Utc::now());
        self.removed_by = Some(deleted_by);
        self.is_active = false;
    }

    /// Check if application is deleted
    pub fn is_deleted(&self) -> bool {
        self.removed_at.is_some()
    }

    /// Verify redirect URI is allowed
    pub fn is_redirect_uri_allowed(&self, uri: &str) -> bool {
        self.redirect_uris.iter().any(|allowed| {
            // Exact match or wildcard subdomain match
            allowed == uri || 
            (allowed.contains('*') && uri.ends_with(&allowed.replace("*.", "")))
        })
    }

    /// Rotate client secret
    pub fn rotate_secret(&mut self, new_secret: String, rotated_by: Uuid) {
        self.client_secret = new_secret;
        self.modified_at = Some(Utc::now());
        self.modified_by = Some(rotated_by);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_valid_client_app() {
        let app = ClientApp::new(
            "Test App".to_string(),
            Some("Test description".to_string()),
            "test-app".to_string(),
            AppType::Web,
            "https://example.com".to_string(),
            "secret_hash".to_string(),
            vec!["https://example.com/callback".to_string()],
            None,
        );
        assert!(app.is_ok());
    }

    #[test]
    fn test_redirect_uri_validation() {
        let app = ClientApp::new(
            "Test App".to_string(),
            None,
            "test-app".to_string(),
            AppType::Web,
            "https://example.com".to_string(),
            "secret_hash".to_string(),
            vec![
                "https://example.com/callback".to_string(),
                "https://*.example.com/callback".to_string(),
            ],
            None,
        ).unwrap();

        assert!(app.is_redirect_uri_allowed("https://example.com/callback"));
        assert!(!app.is_redirect_uri_allowed("https://evil.com/callback"));
    }
}

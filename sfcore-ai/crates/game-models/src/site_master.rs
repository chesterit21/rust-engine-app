use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Site master - "SiteMaster" table
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SiteMaster {
    #[sqlx(rename = "Id")]
    pub id: i64,

    #[sqlx(rename = "GroupName")]
    pub group_name: String,

    #[sqlx(rename = "ProviderName")]
    pub provider_name: String,

    #[sqlx(rename = "LinkSite")]
    pub link_site: String,
}

/// Create SiteMaster input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSiteMaster {
    pub group_name: String,
    pub provider_name: String,
    pub link_site: String,
}

/// Update SiteMaster input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSiteMaster {
    pub id: i64,
    pub group_name: String,
    pub provider_name: String,
    pub link_site: String,
}
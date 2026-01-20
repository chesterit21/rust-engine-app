//! # SSO Core - Domain Module
//! 
//! Domain entities for the SSO application.

pub mod member_user;
pub mod client_app;
pub mod tenant_app;
pub mod menu_app;
pub mod group_app;
pub mod group_menu_app;
pub mod user_group_app;
pub mod member_user_tenant_app;

// Re-export all entities and enums
pub use member_user::{MemberUser, MemberStatus};
pub use client_app::{ClientApp, AppType};
pub use tenant_app::{TenantApp, SubscriptionPlan};
pub use menu_app::MenuApp;
pub use group_app::GroupApp;
pub use group_menu_app::{GroupMenuApp, MenuPermissions};
pub use user_group_app::UserGroupApp;
pub use member_user_tenant_app::{MemberUserTenantApp, TenantRole};

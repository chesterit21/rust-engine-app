// ============================================================================
// SSO Infrastructure - PostgreSQL Tenant Repository
// File: crates/sso-infrastructure/src/database/postgres/tenant_repo_impl.rs
// ============================================================================

use async_trait::async_trait;
use sqlx::{PgPool, FromRow};
use uuid::Uuid;
use tracing::{info, error};
use chrono::{DateTime, Utc};

use sso_core::domain::{TenantApp, SubscriptionPlan};
use sso_core::error::DomainError;
use sso_core::repositories::TenantRepository;

pub struct PgTenantRepository {
    pool: PgPool,
}

impl PgTenantRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

// Internal row type for SQLx mapping
#[derive(Debug, FromRow)]
struct TenantAppRow {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub slug: String,
    pub is_active: bool,
    pub max_users: i32,
    pub subscription_plan: String,
    pub subscription_expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub modified_at: Option<DateTime<Utc>>,
    pub modified_by: Option<Uuid>,
    pub removed_at: Option<DateTime<Utc>>,
    pub removed_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub approved_by: Option<Uuid>,
}

impl From<TenantAppRow> for TenantApp {
    fn from(row: TenantAppRow) -> Self {
        TenantApp {
            id: row.id,
            name: row.name,
            description: row.description,
            slug: row.slug,
            is_active: row.is_active,
            max_users: row.max_users,
            subscription_plan: SubscriptionPlan::from_str(&row.subscription_plan).unwrap_or_default(),
            subscription_expires_at: row.subscription_expires_at,
            created_at: row.created_at,
            created_by: row.created_by,
            modified_at: row.modified_at,
            modified_by: row.modified_by,
            removed_at: row.removed_at,
            removed_by: row.removed_by,
            approved_at: row.approved_at,
            approved_by: row.approved_by,
        }
    }
}

#[async_trait]
impl TenantRepository for PgTenantRepository {
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<TenantApp>, DomainError> {
        let row: Option<TenantAppRow> = sqlx::query_as(
            r#"
            SELECT 
                id, name, description, slug,
                is_active, max_users, subscription_plan, subscription_expires_at,
                created_at, created_by, modified_at, modified_by,
                removed_at, removed_by, approved_at, approved_by
            FROM tenant_apps
            WHERE id = $1 AND removed_at IS NULL
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e: sqlx::Error| {
            error!("Database error finding tenant by id: {}", e);
            DomainError::DatabaseError(e.to_string())
        })?;

        Ok(row.map(|r| r.into()))
    }

    async fn find_by_name(&self, name: &str) -> Result<Option<TenantApp>, DomainError> {
        let row: Option<TenantAppRow> = sqlx::query_as(
            r#"
            SELECT 
                id, name, description, slug,
                is_active, max_users, subscription_plan, subscription_expires_at,
                created_at, created_by, modified_at, modified_by,
                removed_at, removed_by, approved_at, approved_by
            FROM tenant_apps
            WHERE LOWER(name) = LOWER($1) AND removed_at IS NULL
            "#
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e: sqlx::Error| {
            error!("Database error finding tenant by name: {}", e);
            DomainError::DatabaseError(e.to_string())
        })?;

        Ok(row.map(|r| r.into()))
    }

    async fn find_by_slug(&self, slug: &str) -> Result<Option<TenantApp>, DomainError> {
        let row: Option<TenantAppRow> = sqlx::query_as(
            r#"
            SELECT 
                id, name, description, slug,
                is_active, max_users, subscription_plan, subscription_expires_at,
                created_at, created_by, modified_at, modified_by,
                removed_at, removed_by, approved_at, approved_by
            FROM tenant_apps
            WHERE LOWER(slug) = LOWER($1) AND removed_at IS NULL
            "#
        )
        .bind(slug)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e: sqlx::Error| {
            error!("Database error finding tenant by slug: {}", e);
            DomainError::DatabaseError(e.to_string())
        })?;

        Ok(row.map(|r| r.into()))
    }

    async fn create(&self, tenant: &TenantApp) -> Result<TenantApp, DomainError> {
        info!("Creating tenant: {}", tenant.name);
        
        let row: TenantAppRow = sqlx::query_as(
            r#"
            INSERT INTO tenant_apps (
                id, name, description, slug,
                is_active, max_users, subscription_plan, subscription_expires_at,
                created_at, created_by, modified_at, modified_by,
                removed_at, removed_by, approved_at, approved_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            RETURNING 
                id, name, description, slug,
                is_active, max_users, subscription_plan, subscription_expires_at,
                created_at, created_by, modified_at, modified_by,
                removed_at, removed_by, approved_at, approved_by
            "#
        )
        .bind(tenant.id)
        .bind(&tenant.name)
        .bind(&tenant.description)
        .bind(&tenant.slug)
        .bind(tenant.is_active)
        .bind(tenant.max_users)
        .bind(tenant.subscription_plan.as_str())
        .bind(tenant.subscription_expires_at)
        .bind(tenant.created_at)
        .bind(tenant.created_by)
        .bind(tenant.modified_at)
        .bind(tenant.modified_by)
        .bind(tenant.removed_at)
        .bind(tenant.removed_by)
        .bind(tenant.approved_at)
        .bind(tenant.approved_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e: sqlx::Error| {
            error!("Database error creating tenant: {}", e);
            let msg = e.to_string();
            if msg.contains("unique") || msg.contains("duplicate") {
                if msg.contains("slug") {
                    DomainError::TenantSlugAlreadyExists(tenant.slug.clone())
                } else {
                    DomainError::TenantNameAlreadyExists(tenant.name.clone())
                }
            } else {
                DomainError::DatabaseError(msg)
            }
        })?;

        info!("Tenant created successfully: {}", row.id);
        Ok(row.into())
    }

    async fn update(&self, tenant: &TenantApp) -> Result<TenantApp, DomainError> {
        let row: TenantAppRow = sqlx::query_as(
            r#"
            UPDATE tenant_apps
            SET 
                name = $2,
                description = $3,
                slug = $4,
                is_active = $5,
                max_users = $6,
                subscription_plan = $7,
                subscription_expires_at = $8,
                modified_at = $9,
                modified_by = $10,
                removed_at = $11,
                removed_by = $12,
                approved_at = $13,
                approved_by = $14
            WHERE id = $1
            RETURNING 
                id, name, description, slug,
                is_active, max_users, subscription_plan, subscription_expires_at,
                created_at, created_by, modified_at, modified_by,
                removed_at, removed_by, approved_at, approved_by
            "#
        )
        .bind(tenant.id)
        .bind(&tenant.name)
        .bind(&tenant.description)
        .bind(&tenant.slug)
        .bind(tenant.is_active)
        .bind(tenant.max_users)
        .bind(tenant.subscription_plan.as_str())
        .bind(tenant.subscription_expires_at)
        .bind(tenant.modified_at)
        .bind(tenant.modified_by)
        .bind(tenant.removed_at)
        .bind(tenant.removed_by)
        .bind(tenant.approved_at)
        .bind(tenant.approved_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e: sqlx::Error| {
            error!("Database error updating tenant: {}", e);
            DomainError::DatabaseError(e.to_string())
        })?;

        Ok(row.into())
    }
}

// ============================================================================
// SSO Infrastructure - PostgreSQL ClientApp Repository
// File: crates/sso-infrastructure/src/database/postgres/client_app_repo_impl.rs
// ============================================================================

use sqlx::{PgPool, FromRow};
use uuid::Uuid;
use tracing::{info, error};
use chrono::{DateTime, Utc};

use sso_core::domain::{ClientApp, AppType};
use sso_core::error::DomainError;

pub struct PgClientAppRepository {
    pool: PgPool,
}

impl PgClientAppRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(&self, id: &Uuid) -> Result<Option<ClientApp>, DomainError> {
        let row: Option<ClientAppRow> = sqlx::query_as(
            r#"
            SELECT 
                id, name, description, unique_name, type_app,
                url_app, client_secret, redirect_uris, is_active,
                created_at, created_by, modified_at, modified_by,
                removed_at, removed_by, approved_at, approved_by
            FROM client_apps
            WHERE id = $1 AND removed_at IS NULL
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e: sqlx::Error| {
            error!("Database error finding client app by id: {}", e);
            DomainError::DatabaseError(e.to_string())
        })?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn find_by_unique_name(&self, unique_name: &str) -> Result<Option<ClientApp>, DomainError> {
        let row: Option<ClientAppRow> = sqlx::query_as(
            r#"
            SELECT 
                id, name, description, unique_name, type_app,
                url_app, client_secret, redirect_uris, is_active,
                created_at, created_by, modified_at, modified_by,
                removed_at, removed_by, approved_at, approved_by
            FROM client_apps
            WHERE LOWER(unique_name) = LOWER($1) AND removed_at IS NULL
            "#
        )
        .bind(unique_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e: sqlx::Error| {
            error!("Database error finding client app by unique_name: {}", e);
            DomainError::DatabaseError(e.to_string())
        })?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn create(&self, app: &ClientApp) -> Result<ClientApp, DomainError> {
        info!("Creating client app: {}", app.name);
        
        let row: ClientAppRow = sqlx::query_as(
            r#"
            INSERT INTO client_apps (
                id, name, description, unique_name, type_app,
                url_app, client_secret, redirect_uris, is_active,
                created_at, created_by, modified_at, modified_by,
                removed_at, removed_by, approved_at, approved_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
            RETURNING 
                id, name, description, unique_name, type_app,
                url_app, client_secret, redirect_uris, is_active,
                created_at, created_by, modified_at, modified_by,
                removed_at, removed_by, approved_at, approved_by
            "#
        )
        .bind(app.id)
        .bind(&app.name)
        .bind(&app.description)
        .bind(&app.unique_name)
        .bind(app.type_app.as_str())
        .bind(&app.url_app)
        .bind(&app.client_secret)
        .bind(&app.redirect_uris)
        .bind(app.is_active)
        .bind(app.created_at)
        .bind(app.created_by)
        .bind(app.modified_at)
        .bind(app.modified_by)
        .bind(app.removed_at)
        .bind(app.removed_by)
        .bind(app.approved_at)
        .bind(app.approved_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e: sqlx::Error| {
            error!("Database error creating client app: {}", e);
            let msg = e.to_string();
            if msg.contains("unique") || msg.contains("duplicate") {
                DomainError::ClientAppUniqueNameAlreadyExists(app.unique_name.clone())
            } else {
                DomainError::DatabaseError(msg)
            }
        })?;

        info!("Client app created successfully: {}", row.id);
        Ok(row.into())
    }
}

// Internal row type for SQLx mapping
#[derive(Debug, FromRow)]
struct ClientAppRow {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub unique_name: String,
    pub type_app: String,
    pub url_app: String,
    pub client_secret: String,
    pub redirect_uris: Vec<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub modified_at: Option<DateTime<Utc>>,
    pub modified_by: Option<Uuid>,
    pub removed_at: Option<DateTime<Utc>>,
    pub removed_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub approved_by: Option<Uuid>,
}

impl From<ClientAppRow> for ClientApp {
    fn from(row: ClientAppRow) -> Self {
        ClientApp {
            id: row.id,
            name: row.name,
            description: row.description,
            unique_name: row.unique_name,
            type_app: AppType::from_str(&row.type_app).unwrap_or_default(),
            url_app: row.url_app,
            client_secret: row.client_secret,
            redirect_uris: row.redirect_uris,
            is_active: row.is_active,
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

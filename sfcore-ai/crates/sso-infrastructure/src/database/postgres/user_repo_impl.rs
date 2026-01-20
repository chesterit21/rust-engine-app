// ============================================================================
// SSO Infrastructure - PostgreSQL User Repository
// File: crates/sso-infrastructure/src/database/postgres/user_repo_impl.rs
// ============================================================================

use async_trait::async_trait;
use sqlx::{PgPool, Row, FromRow};
use uuid::Uuid;
use tracing::{info, error};
use chrono::{DateTime, Utc};

use sso_core::domain::{MemberUser, MemberStatus};
use sso_core::error::DomainError;
use sso_core::repositories::UserRepository;

pub struct PgUserRepository {
    pool: PgPool,
}

impl PgUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

// Internal row type for SQLx mapping
#[derive(Debug, FromRow)]
struct MemberUserRow {
    pub id: Uuid,
    pub display_name: String,
    pub email: String,
    pub password: Option<String>,
    pub is_active: bool,
    pub is_login: bool,
    pub last_login: Option<DateTime<Utc>>,
    pub status_member: String,
    pub link_profile_image: Option<String>,
    pub email_verified: bool,
    pub phone_number: Option<String>,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub modified_at: Option<DateTime<Utc>>,
    pub modified_by: Option<Uuid>,
    pub removed_at: Option<DateTime<Utc>>,
    pub removed_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub approved_by: Option<Uuid>,
}

impl From<MemberUserRow> for MemberUser {
    fn from(row: MemberUserRow) -> Self {
        MemberUser {
            id: row.id,
            display_name: row.display_name,
            email: row.email,
            password: row.password,
            is_active: row.is_active,
            is_login: row.is_login,
            last_login: row.last_login,
            status_member: MemberStatus::from_str(&row.status_member).unwrap_or_default(),
            link_profile_image: row.link_profile_image,
            email_verified: row.email_verified,
            phone_number: row.phone_number,
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
impl UserRepository for PgUserRepository {
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<MemberUser>, DomainError> {
        let row: Option<MemberUserRow> = sqlx::query_as(
            r#"
            SELECT 
                id, display_name, email, password, 
                is_active, is_login, last_login, status_member,
                link_profile_image, email_verified, phone_number,
                created_at, created_by, modified_at, modified_by,
                removed_at, removed_by, approved_at, approved_by
            FROM member_users
            WHERE id = $1 AND removed_at IS NULL
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e: sqlx::Error| {
            error!("Database error finding user by id: {}", e);
            DomainError::DatabaseError(e.to_string())
        })?;

        Ok(row.map(|r| r.into()))
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<MemberUser>, DomainError> {
        let row: Option<MemberUserRow> = sqlx::query_as(
            r#"
            SELECT 
                id, display_name, email, password, 
                is_active, is_login, last_login, status_member,
                link_profile_image, email_verified, phone_number,
                created_at, created_by, modified_at, modified_by,
                removed_at, removed_by, approved_at, approved_by
            FROM member_users
            WHERE LOWER(email) = LOWER($1) AND removed_at IS NULL
            "#
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e: sqlx::Error| {
            error!("Database error finding user by email: {}", e);
            DomainError::DatabaseError(e.to_string())
        })?;

        Ok(row.map(|r| r.into()))
    }

    async fn find_by_display_name(&self, display_name: &str) -> Result<Option<MemberUser>, DomainError> {
        let row: Option<MemberUserRow> = sqlx::query_as(
            r#"
            SELECT 
                id, display_name, email, password, 
                is_active, is_login, last_login, status_member,
                link_profile_image, email_verified, phone_number,
                created_at, created_by, modified_at, modified_by,
                removed_at, removed_by, approved_at, approved_by
            FROM member_users
            WHERE LOWER(display_name) = LOWER($1) AND removed_at IS NULL
            "#
        )
        .bind(display_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e: sqlx::Error| {
            error!("Database error finding user by display_name: {}", e);
            DomainError::DatabaseError(e.to_string())
        })?;

        Ok(row.map(|r| r.into()))
    }

    async fn create(&self, user: &MemberUser) -> Result<MemberUser, DomainError> {
        info!("Creating user with email: {}", user.email);
        
        let row: MemberUserRow = sqlx::query_as(
            r#"
            INSERT INTO member_users (
                id, display_name, email, password,
                is_active, is_login, last_login, status_member,
                link_profile_image, email_verified, phone_number,
                created_at, created_by, modified_at, modified_by,
                removed_at, removed_by, approved_at, approved_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)
            RETURNING 
                id, display_name, email, password, 
                is_active, is_login, last_login, status_member,
                link_profile_image, email_verified, phone_number,
                created_at, created_by, modified_at, modified_by,
                removed_at, removed_by, approved_at, approved_by
            "#
        )
        .bind(user.id)
        .bind(&user.display_name)
        .bind(&user.email)
        .bind(&user.password)
        .bind(user.is_active)
        .bind(user.is_login)
        .bind(user.last_login)
        .bind(user.status_member.as_str())
        .bind(&user.link_profile_image)
        .bind(user.email_verified)
        .bind(&user.phone_number)
        .bind(user.created_at)
        .bind(user.created_by)
        .bind(user.modified_at)
        .bind(user.modified_by)
        .bind(user.removed_at)
        .bind(user.removed_by)
        .bind(user.approved_at)
        .bind(user.approved_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e: sqlx::Error| {
            error!("Database error creating user: {}", e);
            let msg = e.to_string();
            if msg.contains("unique") || msg.contains("duplicate") {
                if msg.contains("email") {
                    DomainError::EmailAlreadyExists(user.email.clone())
                } else {
                    DomainError::DisplayNameAlreadyExists(user.display_name.clone())
                }
            } else {
                DomainError::DatabaseError(msg)
            }
        })?;

        info!("User created successfully: {}", row.id);
        Ok(row.into())
    }

    async fn update(&self, user: &MemberUser) -> Result<MemberUser, DomainError> {
        let row: MemberUserRow = sqlx::query_as(
            r#"
            UPDATE member_users
            SET 
                display_name = $2,
                email = $3,
                password = $4,
                is_active = $5,
                is_login = $6,
                last_login = $7,
                status_member = $8,
                link_profile_image = $9,
                email_verified = $10,
                phone_number = $11,
                modified_at = $12,
                modified_by = $13,
                removed_at = $14,
                removed_by = $15,
                approved_at = $16,
                approved_by = $17
            WHERE id = $1
            RETURNING 
                id, display_name, email, password, 
                is_active, is_login, last_login, status_member,
                link_profile_image, email_verified, phone_number,
                created_at, created_by, modified_at, modified_by,
                removed_at, removed_by, approved_at, approved_by
            "#
        )
        .bind(user.id)
        .bind(&user.display_name)
        .bind(&user.email)
        .bind(&user.password)
        .bind(user.is_active)
        .bind(user.is_login)
        .bind(user.last_login)
        .bind(user.status_member.as_str())
        .bind(&user.link_profile_image)
        .bind(user.email_verified)
        .bind(&user.phone_number)
        .bind(user.modified_at)
        .bind(user.modified_by)
        .bind(user.removed_at)
        .bind(user.removed_by)
        .bind(user.approved_at)
        .bind(user.approved_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e: sqlx::Error| {
            error!("Database error updating user: {}", e);
            DomainError::DatabaseError(e.to_string())
        })?;

        Ok(row.into())
    }

    async fn delete(&self, id: &Uuid) -> Result<(), DomainError> {
        sqlx::query(
            r#"
            UPDATE member_users
            SET removed_at = NOW(), is_active = false
            WHERE id = $1
            "#
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e: sqlx::Error| {
            error!("Database error deleting user: {}", e);
            DomainError::DatabaseError(e.to_string())
        })?;

        Ok(())
    }
}

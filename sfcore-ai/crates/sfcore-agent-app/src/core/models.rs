//! User Model

use serde::{Deserialize, Serialize};

/// User entity dari database
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub is_active: bool,
}

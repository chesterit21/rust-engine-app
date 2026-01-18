//! Authentication Service
//!
//! Login verification dengan support untuk Bcrypt DAN Argon2.

use crate::core::models::User;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use sqlx::PgPool;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("User not found: {0}")]
    UserNotFound(String),
    #[error("Invalid password")]
    InvalidPassword,
    #[error("User is inactive")]
    UserInactive,
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Password verification failed: {0}")]
    VerificationError(String),
}

/// Hash password dengan Argon2 (untuk registrasi baru)
#[allow(dead_code)]
pub fn hash_password(password: &str) -> Result<String, AuthError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| AuthError::VerificationError(e.to_string()))
}

/// Verify password - supports BCRYPT and ARGON2
fn verify_password_hash(password: &str, stored_hash: &str) -> bool {
    eprintln!("[DEBUG] Verifying password against hash: {}...", &stored_hash.chars().take(20).collect::<String>());

    // Try Bcrypt first (most common for existing DBs)
    if stored_hash.starts_with("$2") {
        // Bcrypt hash starts with $2a$, $2b$, or $2y$
        eprintln!("[DEBUG] Detected BCRYPT hash format");
        match bcrypt::verify(password, stored_hash) {
            Ok(valid) => {
                eprintln!("[DEBUG] Bcrypt verify result: {}", valid);
                return valid;
            }
            Err(e) => {
                eprintln!("[DEBUG] Bcrypt verify error: {}", e);
                return false;
            }
        }
    }

    // Try Argon2 ($argon2id$, $argon2i$, $argon2d$)
    if stored_hash.starts_with("$argon2") {
        eprintln!("[DEBUG] Detected ARGON2 hash format");
        if let Ok(parsed_hash) = PasswordHash::new(stored_hash) {
            let result = Argon2::default()
                .verify_password(password.as_bytes(), &parsed_hash)
                .is_ok();
            eprintln!("[DEBUG] Argon2 verify result: {}", result);
            return result;
        }
    }

    eprintln!("[DEBUG] Unknown hash format, trying as plain text comparison (NOT RECOMMENDED!)");
    // Last resort: plain text comparison (untuk legacy systems)
    password == stored_hash
}

/// Verify login credentials - supports login by username OR email
pub async fn verify_login(pool: &PgPool, username_or_email: &str, password: &str) -> Result<User, AuthError> {
    eprintln!("[DEBUG] Attempting login for: '{}'", username_or_email);

    // Query by username OR email
    let result = sqlx::query_as::<_, User>(
        "SELECT id, username, password, email, phone, is_active FROM users WHERE username = $1 OR email = $1",
    )
    .bind(username_or_email)
    .fetch_optional(pool)
    .await;

    match &result {
        Ok(Some(u)) => eprintln!("[DEBUG] Found user: id={}, username='{}', email={:?}", u.id, u.username, u.email),
        Ok(None) => eprintln!("[DEBUG] No user found with: '{}'", username_or_email),
        Err(e) => eprintln!("[DEBUG] Database error: {}", e),
    }

    let user = result?.ok_or_else(|| AuthError::UserNotFound(username_or_email.to_string()))?;

    if !user.is_active {
        eprintln!("[DEBUG] User is inactive");
        return Err(AuthError::UserInactive);
    }

    // Verify password (blocking task for CPU-intensive hash verification)
    let stored_hash = user.password.clone();
    let password_owned = password.to_string();

    let is_valid = tokio::task::spawn_blocking(move || {
        verify_password_hash(&password_owned, &stored_hash)
    })
    .await
    .map_err(|e| AuthError::VerificationError(e.to_string()))?;

    if !is_valid {
        return Err(AuthError::InvalidPassword);
    }

    eprintln!("[DEBUG] Login successful for: {}", user.username);
    Ok(user)
}

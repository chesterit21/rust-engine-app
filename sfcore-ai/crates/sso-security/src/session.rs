//! Session management (placeholder)

use uuid::Uuid;

pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub expires_at: i64,
}

impl Session {
    pub fn new(user_id: Uuid, ttl_seconds: i64) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            expires_at: chrono::Utc::now().timestamp() + ttl_seconds,
        }
    }
}

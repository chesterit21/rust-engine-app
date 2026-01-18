//! Application Events
//!
//! Event enum untuk komunikasi async → UI thread.

use crate::core::models::User;

/// Events yang dikirim dari background tasks ke UI
#[derive(Debug)]
pub enum AppEvent {
    /// Login berhasil dengan data user
    LoginSuccess(User),
    /// Login gagal dengan error message
    LoginFailed(String),
    /// Update loading state
    SetLoading(bool),
    /// Response from AI Engine
    EngineResponse(String),
    /// Server status changed (Starting, WarmingUp, Running, Error)
    ServerStatusChange(String),
    /// Streaming finished (reset loading state)
    StreamEnd,
}

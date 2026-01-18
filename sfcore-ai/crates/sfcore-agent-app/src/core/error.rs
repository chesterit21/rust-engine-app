use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("Socket connection failed: {0}")]
    ConnectionFailed(#[from] std::io::Error),
    #[error("Server timeout after {0:?}")]
    Timeout(Duration),
    #[error("Model loading failed: {0}")]
    ModelLoadFailed(String),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("Server error: {0}")]
    ServerError(String),
    #[error("Process error: {0}")]
    ProcessError(String),
}

pub type Result<T> = std::result::Result<T, EngineError>;

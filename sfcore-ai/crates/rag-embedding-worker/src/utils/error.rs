use thiserror::Error;

#[derive(Error, Debug)]
pub enum WorkerError {
    #[error("Document not found: {0}")]
    DocumentNotFound(i32),
    
    #[error("File not found: {0}")]
    FileNotFound(String),
    
    #[error("Unsupported file type: {0}")]
    UnsupportedFileType(String),
    
    #[error("File too large: {0} MB (max: {1} MB)")]
    FileTooLarge(u64, u64),
    
    #[error("Parsing error: {0}")]
    ParsingError(String),
    
    #[error("Chunking error: {0}")]
    ChunkingError(String),
    
    #[error("Embedding error: {0}")]
    EmbeddingError(String),
    
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
    
    #[error("Llama-server not running")]
    LlamaServerNotRunning,
    
    #[error("Llama-server failed to start: {0}")]
    LlamaServerStartFailed(String),
    
    #[error("Insufficient memory: {0} MB available (required: {1} MB)")]
    InsufficientMemory(u64, u64),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<anyhow::Error> for WorkerError {
    fn from(err: anyhow::Error) -> Self {
        WorkerError::Unknown(err.to_string())
    }
}

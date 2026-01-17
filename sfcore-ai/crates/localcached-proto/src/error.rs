use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProtoError {
    #[error("invalid key format; expected svc:table:pk")]
    InvalidKeyFormat,

    #[error("invalid utf8")]
    InvalidUtf8,

    #[error("bad payload")]
    BadPayload,

    #[error("unsupported format")]
    UnsupportedFormat,

    #[error("not found")]
    NotFound,

    #[error("server error: {0}")]
    ServerError(String),

    #[error("invalid frame: {0}")]
    InvalidFrame(String),

    #[error("connection closed")]
    ConnectionClosed,

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

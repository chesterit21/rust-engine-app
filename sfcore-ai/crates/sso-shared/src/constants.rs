//! Application-wide constants

pub const DEFAULT_PAGE_SIZE: u32 = 20;
pub const MAX_PAGE_SIZE: u32 = 100;
pub const TOKEN_TYPE_ACCESS: &str = "access";
pub const TOKEN_TYPE_REFRESH: &str = "refresh";
pub const DEFAULT_ACCESS_TOKEN_EXPIRY: i64 = 900;
pub const DEFAULT_REFRESH_TOKEN_EXPIRY: i64 = 604800;
pub const MIN_PASSWORD_LENGTH: usize = 8;
pub const MAX_PASSWORD_LENGTH: usize = 128;

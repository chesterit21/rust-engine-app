pub mod game_dto;
pub mod play_dto;
pub mod auth;

pub use game_dto::*;
pub use play_dto::*;
#[allow(unused_imports)]
pub use auth::*;

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ApiResponse<T> {
    pub status: String,
    pub message: String,
    pub data: Option<T>,
}
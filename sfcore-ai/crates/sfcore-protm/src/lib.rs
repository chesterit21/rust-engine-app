
pub mod domain;
pub mod repository;
pub mod service;
pub mod handler;
pub mod config;
pub mod dto;

pub struct AppState {
    pub db: sqlx::SqlitePool,
}

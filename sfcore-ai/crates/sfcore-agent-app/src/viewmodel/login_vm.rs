//! Login ViewModel
//!
//! State dan logic untuk login form dengan async handling.

use crate::core::auth::verify_login;
use crate::events::AppEvent;
use eframe::egui;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::mpsc;

pub struct LoginViewModel {
    pub username: String,
    pub password: String,
    pub error: Option<String>,
    pub is_loading: bool,
    pub is_logged_in: bool,
    pub show_password: bool, // Toggle password visibility
    db_pool: Arc<PgPool>,
    event_tx: mpsc::UnboundedSender<AppEvent>,
}

impl LoginViewModel {
    pub fn new(db_pool: Arc<PgPool>, event_tx: mpsc::UnboundedSender<AppEvent>) -> Self {
        Self {
            username: String::with_capacity(100),
            password: String::with_capacity(100),
            error: None,
            is_loading: false,
            is_logged_in: false,
            show_password: false,
            db_pool,
            event_tx,
        }
    }

    /// Attempt login - non-blocking async
    pub fn login(&mut self, ctx: egui::Context) {
        if self.is_loading {
            return;
        }

        if self.username.trim().is_empty() || self.password.is_empty() {
            self.error = Some("Email dan password wajib diisi".to_string());
            return;
        }

        self.error = None;
        self.is_loading = true;

        let username = self.username.clone();
        let password = self.password.clone();
        let pool = Arc::clone(&self.db_pool);
        let tx = self.event_tx.clone();

        tokio::spawn(async move {
            match verify_login(&pool, &username, &password).await {
                Ok(user) => {
                    let _ = tx.send(AppEvent::LoginSuccess(user));
                }
                Err(e) => {
                    let _ = tx.send(AppEvent::LoginFailed(e.to_string()));
                }
            }
            ctx.request_repaint();
        });
    }

    pub fn on_login_success(&mut self) {
        self.is_loading = false;
        self.is_logged_in = true;
        self.password.clear();
        self.password.shrink_to_fit();
    }

    pub fn on_login_failed(&mut self, error: String) {
        self.is_loading = false;
        self.error = Some(error);
    }
}

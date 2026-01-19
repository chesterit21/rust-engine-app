//! Root Application
//!
//! Dock-style UI with image icons.
//! 
//! ## Window Layout Reference:
//! - Login window: 280x380px (compact form)
//! - Dock only: DOCK_WIDTH x DOCK_HEIGHT (100x800 default)
//! - Dock + Panel: (DOCK_WIDTH + GAP + PANEL_WIDTH) x DOCK_HEIGHT

use crate::assets::TextureCache;
use crate::core::db;
use crate::events::AppEvent;
use crate::view::{
    dock_view::render_dock, login_view::render_login, panel_view::render_panel,
    theme_view::render_theme,
};
use crate::viewmodel::{EngineViewModel, LoginViewModel, MenuViewModel, ThemeViewModel};
use eframe::egui;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::mpsc;

// ============================================================================
// WINDOW SIZE & POSITION CONSTANTS - EDIT THESE TO ADJUST LAYOUT
// ============================================================================

/// Width of the dock bar (vertical menu on the right side)
/// Increase this if icons appear too cramped
pub const DOCK_WIDTH: f32 = 60.0;

/// Height of both dock and panel windows
/// Increased by 50px for better visibility
pub const DOCK_HEIGHT: f32 = 430.0;

/// Width of the slide-out panel (appears when menu clicked)
/// Increased by 100px for more space
pub const PANEL_WIDTH: f32 = 700.0;

/// Gap/margin between dock and panel (transparent space between them)
/// Increase for more visible separation
pub const GAP: f32 = 80.0;

// ============================================================================

pub enum AppScreen {
    Login,
    Dock,
}

pub struct MyApp {
    pub screen: AppScreen,
    pub login_vm: LoginViewModel,
    pub menu_vm: MenuViewModel,
    pub theme_vm: ThemeViewModel,
    pub engine_vm: EngineViewModel,
    pub textures: TextureCache,
    event_rx: mpsc::UnboundedReceiver<AppEvent>,
    _db_pool: Arc<PgPool>,
    last_panel_visible: bool,
    last_theme_visible: bool,
    /// Posisi dock ASLI sebelum panel dibuka (untuk restore saat close)
    saved_dock_position: Option<egui::Pos2>,
}

impl MyApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        let db_pool = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                match db::create_pool().await {
                    Ok(pool) => Arc::new(pool),
                    Err(e) => {
                        eprintln!("Failed to connect to database: {}", e);
                        panic!("Database connection required: {}", e);
                    }
                }
            })
        });

        // ====================================================================
        // LOGIN WINDOW SIZE - Initial window size for login form
        // Format: (width, height) in pixels
        // ====================================================================
        cc.egui_ctx
            .send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(280.0, 380.0)));
        cc.egui_ctx
            .send_viewport_cmd(egui::ViewportCommand::Decorations(false)); // No title bar

        Self {
            screen: AppScreen::Login,
            login_vm: LoginViewModel::new(Arc::clone(&db_pool), event_tx.clone()),
            menu_vm: MenuViewModel::new(),
            theme_vm: ThemeViewModel::new(),
            engine_vm: EngineViewModel::new(event_tx),
            textures: TextureCache::new(),
            event_rx,
            _db_pool: db_pool,
            last_panel_visible: false,
            last_theme_visible: false,
            saved_dock_position: None,
        }
    }

    fn process_events(&mut self, ctx: &egui::Context) {
        while let Ok(event) = self.event_rx.try_recv() {
            match event {
                AppEvent::LoginSuccess(user) => {
                    self.login_vm.error = None;
                    self.login_vm.is_loading = false;
                    println!("[DEBUG] Login successful for: {}", user.username);
                    
                    // Switch to Dock screen
                    self.screen = AppScreen::Dock;

                    // Resize to dock dimensions
                    ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(
                        DOCK_WIDTH,
                        DOCK_HEIGHT,
                    )));
                }
                AppEvent::LoginFailed(msg) => {
                    self.login_vm.error = Some(msg);
                    self.login_vm.is_loading = false;
                }
                AppEvent::SetLoading(loading) => {
                    self.login_vm.is_loading = loading;
                }
                AppEvent::EngineResponse(response) => {
                    self.engine_vm.append_response(&response);
                    // DON'T set is_loading = false here! Wait for StreamEnd
                }
                AppEvent::ServerStatusChange(status) => {
                    use crate::viewmodel::engine_vm::ServerStatus;
                    self.engine_vm.status = match status.as_str() {
                        "WarmingUp" => ServerStatus::WarmingUp,
                        "Running" => ServerStatus::Running,
                        "Error" => ServerStatus::Error("Timeout".to_string()),
                        _ => ServerStatus::Stopped,
                    };
                }
                AppEvent::StreamEnd => {
                    self.engine_vm.is_loading = false;
                }
            }
            ctx.request_repaint();
        }
    }

    fn update_window_size(&mut self, ctx: &egui::Context) {
        let panel_visible = self.menu_vm.menu_visible && self.menu_vm.active_menu.is_some();
        let theme_visible = self.theme_vm.window_visible;

        // Check if panel OR theme changed visibility
        let panel_changed = panel_visible != self.last_panel_visible;
        let theme_changed = theme_visible != self.last_theme_visible;

        if panel_changed || theme_changed {
            let was_open = self.last_panel_visible || self.last_theme_visible;
            let now_open = panel_visible || theme_visible;

            self.last_panel_visible = panel_visible;
            self.last_theme_visible = theme_visible;

            if now_open && !was_open {
                // ============================================================
                // OPENING - Save dock position BEFORE expand
                // ============================================================
                let current_pos = ctx.input(|i| i.viewport().outer_rect)
                    .map(|r| r.min)
                    .unwrap_or(egui::pos2(500.0, 300.0));
                
                // Save posisi dock asli untuk di-restore nanti
                self.saved_dock_position = Some(current_pos);

                let content_width = if panel_visible {
                    PANEL_WIDTH
                } else {
                    250.0 // Theme popup width
                };

                let total_width = DOCK_WIDTH + GAP + content_width;
                let new_x = current_pos.x - content_width - GAP;

                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(
                    total_width,
                    DOCK_HEIGHT,
                )));
                ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(
                    egui::pos2(new_x.max(0.0), current_pos.y).into(),
                ));
            } else if !now_open && was_open {
                // ============================================================
                // CLOSING - Restore dock ke posisi ASLI
                // ============================================================
                if let Some(saved_pos) = self.saved_dock_position {
                    ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(
                        DOCK_WIDTH,
                        DOCK_HEIGHT,
                    )));
                    ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(
                        saved_pos.into(),
                    ));
                }
                self.saved_dock_position = None;
            } else if now_open {
                // ============================================================
                // SWITCHING between panels (e.g. Dashboard -> Settings)
                // Keep window position, just resize if needed
                // ============================================================
                let content_width = if panel_visible {
                    PANEL_WIDTH
                } else {
                    250.0
                };

                let total_width = DOCK_WIDTH + GAP + content_width;
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(
                    total_width,
                    DOCK_HEIGHT,
                )));
            }
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.process_events(ctx);
        self.theme_vm.theme.apply(ctx);

        match self.screen {
            AppScreen::Login => {
                render_login(ctx, &mut self.login_vm);
            }
            AppScreen::Dock => {
                self.update_window_size(ctx);
                render_panel(ctx, &mut self.menu_vm, &mut self.engine_vm, &mut self.textures);
                render_dock(ctx, &mut self.menu_vm, &mut self.theme_vm, &mut self.textures);
                render_theme(ctx, &mut self.theme_vm, &mut self.textures);
            }
        }

        // GRADIENT BORDER - Draw at window edge after all views
        let screen_rect = ctx.screen_rect().shrink(1.0);
        let layer_id = egui::LayerId::new(egui::Order::Foreground, egui::Id::new("window_border"));
        let painter = ctx.layer_painter(layer_id);
        crate::ui_helpers::draw_gradient_border(&painter, screen_rect, 1.0);
    }
}

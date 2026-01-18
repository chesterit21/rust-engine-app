//! SFCore Agent App - Desktop Application
//!
//! Aplikasi desktop berbasis eGUI + PostgreSQL dengan arsitektur MVVM.
//! UI: Dock-style vertical bar dengan slide-out panels.

mod app;
mod assets;
mod core;
mod events;
mod theme;
mod ui_helpers;
mod view;
mod viewmodel;

use app::MyApp;

#[tokio::main]
async fn main() -> eframe::Result<()> {
    dotenvy::dotenv().ok();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            // ================================================================
            // INITIAL WINDOW SIZE - Login form dimensions (width x height)
            // This is the first window that appears before login
            // ================================================================
            .with_inner_size([280.0, 380.0]) // ← Login window size
            .with_transparent(true)          // Allow semi-transparent background
            .with_decorations(false)         // No title bar
            .with_always_on_top()            // Stay above other windows
            .with_resizable(false),          // Fixed size
        centered: false,
        ..Default::default()
    };

    eframe::run_native(
        "SFCore Agent",
        native_options,
        Box::new(|cc| {
            // ================================================================
            // INITIAL WINDOW POSITION - Where login window appears
            // Format: pos2(X from left edge, Y from top edge)
            // Adjust these values for your screen resolution
            // ================================================================
            cc.egui_ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(
                egui::pos2(800.0, 800.0).into(), // ← Login window position
            ));
            Ok(Box::new(MyApp::new(cc)))
        }),
    )
}

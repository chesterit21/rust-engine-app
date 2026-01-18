//! View Module
//!
//! UI layer - hanya rendering, tidak ada business logic.

pub mod dock_view;
pub mod login_view;
pub mod menu_view;
pub mod panel_view;
pub mod theme_view;

pub use dock_view::render_dock;
pub use login_view::render_login;
pub use panel_view::render_panel;
pub use theme_view::render_theme;

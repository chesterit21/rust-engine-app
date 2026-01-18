//! Theme Handler
//!
//! Light/Dark theme dengan proper eGUI visuals.
//! Window background transparent supaya rounded corners terlihat cantik.

use eframe::egui;

/// Theme variants
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Theme {
    #[default]
    Light,
    Dark,
}

impl Theme {
    /// Apply theme ke egui context
    /// Window background dibuat FULLY TRANSPARENT supaya:
    /// - Rounded corners pada dock/panel terlihat smooth
    /// - Tidak ada "kotak" di belakang UI elements
    pub fn apply(&self, ctx: &egui::Context) {
        let mut visuals = match self {
            Theme::Light => egui::Visuals::light(),
            Theme::Dark => egui::Visuals::dark(),
        };

        // FULLY TRANSPARENT window background
        // Ini bikin rounded corners pada Frame terlihat cantik
        visuals.window_fill = egui::Color32::TRANSPARENT;
        visuals.panel_fill = egui::Color32::TRANSPARENT;

        // Extreme dark background untuk widgets (tetap ada contrast)
        visuals.extreme_bg_color = egui::Color32::from_rgba_unmultiplied(20, 20, 25, 200);

        ctx.set_visuals(visuals);
    }

    /// Toggle between themes
    pub fn toggle(&mut self) {
        *self = match self {
            Theme::Light => Theme::Dark,
            Theme::Dark => Theme::Light,
        };
    }
}

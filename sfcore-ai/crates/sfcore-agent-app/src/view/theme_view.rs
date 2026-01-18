//! ============================================================================
//! THEME VIEW - Theme Picker
//! ============================================================================
//!
//! Border drawn at window level in app.rs.

use crate::assets::TextureCache;
use crate::theme::Theme;
use crate::viewmodel::ThemeViewModel;
use eframe::egui::{self, Color32, CursorIcon, Image, Pos2, RichText, Vec2};

// ============================================================================
// THEME POPUP SIZE
// ============================================================================

const POPUP_WIDTH: f32 = 250.0;
const POPUP_HEIGHT: f32 = 150.0;

// ============================================================================

pub fn render_theme(ctx: &egui::Context, vm: &mut ThemeViewModel, textures: &mut TextureCache) {
    vm.animate(ctx);

    if !vm.window_visible {
        return;
    }

    let popup_x = 0.0 - vm.offset;
    let popup_y = 0.0;

    let area_response = egui::Area::new(egui::Id::new("theme_picker"))
        .fixed_pos(Pos2::new(popup_x, popup_y))
        .show(ctx, |ui| {
            egui::Frame::none()
                .fill(Color32::from_rgba_unmultiplied(35, 35, 45, 250))
                .inner_margin(20.0)
                .show(ui, |ui| {
                    ui.set_min_size(Vec2::new(POPUP_WIDTH, POPUP_HEIGHT));

                    // HEADER - Close button + Title
                    ui.horizontal(|ui| {
                        let close_tex = textures.close(ctx);
                        let close_btn = ui.add(
                            egui::ImageButton::new(
                                Image::new(&close_tex).fit_to_exact_size(Vec2::splat(24.0)),
                            )
                            .frame(false),
                        );
                        if close_btn.hovered() {
                            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
                        }
                        if close_btn.clicked() {
                            vm.hide_window();
                        }
                        close_btn.on_hover_text("Close");

                        ui.add_space(15.0);
                        ui.heading(
                            RichText::new("Select Theme")
                                .size(20.0)
                                .color(Color32::WHITE),
                        );
                    });

                    ui.add_space(5.0);
                    ui.separator();
                    ui.add_space(15.0);

                    // THEME BUTTONS
                    ui.horizontal(|ui| {
                        // Light theme
                        let light_btn = ui.add(
                            egui::Button::new(
                                RichText::new("☀️ Light").size(16.0).color(Color32::WHITE),
                            )
                            .fill(if vm.theme == Theme::Light {
                                Color32::from_rgb(70, 100, 150)
                            } else {
                                Color32::from_rgb(60, 60, 70)
                            })
                            .rounding(8.0)
                            .min_size(Vec2::new(100.0, 45.0)),
                        );
                        if light_btn.hovered() {
                            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
                        }
                        if light_btn.clicked() {
                            vm.set_theme(Theme::Light);
                        }

                        ui.add_space(15.0);

                        // Dark theme
                        let dark_btn = ui.add(
                            egui::Button::new(
                                RichText::new("🌙 Dark").size(16.0).color(Color32::WHITE),
                            )
                            .fill(if vm.theme == Theme::Dark {
                                Color32::from_rgb(70, 100, 150)
                            } else {
                                Color32::from_rgb(60, 60, 70)
                            })
                            .rounding(8.0)
                            .min_size(Vec2::new(100.0, 45.0)),
                        );
                        if dark_btn.hovered() {
                            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
                        }
                        if dark_btn.clicked() {
                            vm.set_theme(Theme::Dark);
                        }
                    });
                });
        });
}

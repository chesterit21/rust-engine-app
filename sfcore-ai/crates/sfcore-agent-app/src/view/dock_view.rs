//! ============================================================================
//! DOCK VIEW - Vertical Dock Bar
//! ============================================================================
//!
//! Border drawn at window level in app.rs.

use crate::assets::TextureCache;
use crate::viewmodel::{menu_vm::MenuItem, MenuViewModel, ThemeViewModel};
use eframe::egui::{self, Color32, CursorIcon, Image, RichText, Sense, Vec2};

// ============================================================================
// DOCK SIZE CONSTANTS - Should match app.rs values
// ============================================================================

/// Width of dock bar - adjust if icons are cramped
const DOCK_WIDTH: f32 = 60.0;

/// Height of dock bar - synced with DOCK_HEIGHT in app.rs
const DOCK_HEIGHT: f32 = 380.0;

/// Size of menu icons (width = height, square icons)
const ICON_SIZE: f32 = 24.0;

// ============================================================================

pub fn render_dock(
    ctx: &egui::Context,
    menu_vm: &mut MenuViewModel,
    theme_vm: &mut ThemeViewModel,
    textures: &mut TextureCache,
) {
    let window_width = ctx.screen_rect().width();
    let dock_x = window_width - DOCK_WIDTH;

    let area_response = egui::Area::new(egui::Id::new("dock_bar"))
        .fixed_pos(egui::pos2(dock_x, 0.0))
        .sense(Sense::drag())
        .show(ctx, |ui| {
            // DRAG TO MOVE WINDOW
            let response = ui.interact(
                ui.max_rect(),
                egui::Id::new("dock_drag_area"),
                Sense::drag(),
            );

            if response.dragged() {
                ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
                let delta = response.drag_delta();
                if delta != Vec2::ZERO {
                    ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }
            } else if response.hovered() {
                ui.ctx().set_cursor_icon(CursorIcon::Grab);
            }

            egui::Frame::none()
                .fill(Color32::from_rgba_unmultiplied(30, 30, 40, 245))
                .inner_margin(8.0)
                .show(ui, |ui| {
                    ui.set_min_size(Vec2::new(DOCK_WIDTH - 16.0, DOCK_HEIGHT));

                    ui.vertical_centered(|ui| {
                        ui.add_space(10.0);

                        // Logo from favicon.ico
                        let favicon = textures.favicon(ctx);
                        let logo = ui.add(
                            Image::new(&favicon)
                                .fit_to_exact_size(Vec2::splat(40.0))
                                .sense(Sense::click()),
                        );
                        if logo.hovered() {
                            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
                        }

                        ui.add_space(3.0);
                        ui.label(RichText::new("SFCore").size(10.0).color(Color32::GRAY));
                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(10.0);

                        // Menu items with image icons
                        let dashboard_tex = textures.dashboard(ctx);
                        dock_menu_item(
                            ui,
                            &dashboard_tex,
                            "Dashboard",
                            MenuItem::Dashboard,
                            menu_vm,
                        );
                        ui.add_space(8.0);

                        let speedometer_tex = textures.speedometer(ctx);
                        dock_menu_item(
                            ui,
                            &speedometer_tex,
                            "Settings",
                            MenuItem::Settings,
                            menu_vm,
                        );
                        ui.add_space(8.0);

                        let workflow_tex = textures.workflow(ctx);
                        dock_menu_item(ui, &workflow_tex, "Reports", MenuItem::Reports, menu_vm);

                        // Flexible spacer
                        let remaining = ui.available_height() - 100.0;
                        if remaining > 0.0 {
                            ui.add_space(remaining);
                        } else {
                            ui.add_space(5.0);
                        }

                        ui.separator();
                        ui.add_space(8.0);

                        // Theme button
                        let theme_btn = ui.add(
                            egui::Button::new(RichText::new("🎨").size(20.0))
                                .frame(false)
                                .sense(Sense::click()),
                        );
                        if theme_btn.hovered() {
                            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
                        }
                        if theme_btn.clicked() {
                            if theme_vm.window_visible {
                                theme_vm.hide_window();
                            } else {
                                theme_vm.show_window();
                            }
                        }
                        theme_btn.on_hover_text("Change Theme");

                        ui.add_space(8.0);

                        // Close app button
                        let close_app_tex = textures.close_app(ctx);
                        let close_btn = ui.add(
                            egui::ImageButton::new(
                                Image::new(&close_app_tex).fit_to_exact_size(Vec2::splat(24.0)),
                            )
                            .frame(false),
                        );
                        if close_btn.hovered() {
                            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
                        }
                        if close_btn.clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        close_btn.on_hover_text("Close Application");
                    });
                });
        });
}

fn dock_menu_item(
    ui: &mut egui::Ui,
    icon: &egui::TextureHandle,
    tooltip: &str,
    item: MenuItem,
    menu_vm: &mut MenuViewModel,
) {
    let is_active = menu_vm.active_menu.as_ref() == Some(&item);
    let bg_color = if is_active {
        Color32::from_rgba_unmultiplied(80, 100, 140, 200)
    } else {
        Color32::TRANSPARENT
    };

    let btn = ui.add(
        egui::ImageButton::new(Image::new(icon).fit_to_exact_size(Vec2::splat(ICON_SIZE)))
            .frame(true)
            .rounding(8.0)
            .tint(if is_active {
                Color32::WHITE
            } else {
                Color32::LIGHT_GRAY
            }),
    );

    // Cursor pointer on hover
    if btn.hovered() {
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
    }

    // Draw background if active
    if is_active {
        let rect = btn.rect.expand(4.0);
        ui.painter().rect_filled(rect, 8.0, bg_color);
    }

    if btn.clicked() {
        if is_active {
            menu_vm.hide_menu();
        } else {
            menu_vm.select_menu(item);
            menu_vm.show_menu();
        }
    }

    btn.on_hover_text(tooltip);
}

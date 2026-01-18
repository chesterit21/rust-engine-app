//! Login View
//!
//! Compact login form. Border drawn at window level.

use crate::viewmodel::LoginViewModel;
use eframe::egui::{self, Button, Color32, RichText, TextEdit, Vec2};

pub fn render_login(ctx: &egui::Context, vm: &mut LoginViewModel) {
    let response = egui::CentralPanel::default()
        .frame(
            egui::Frame::none()
                .fill(Color32::from_rgba_unmultiplied(25, 25, 35, 245))
                .inner_margin(20.0),
        )
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(15.0);

                // Logo / Title
                ui.label(RichText::new("🔐").size(40.0));
                ui.add_space(5.0);
                ui.label(
                    RichText::new("SFCore Agent")
                        .size(18.0)
                        .color(Color32::WHITE),
                );
                ui.add_space(20.0);

                // Username input
                ui.label(
                    RichText::new("Email / Username")
                        .size(11.5)
                        .color(Color32::WHITE),
                );
                ui.add_space(3.0);
                let username_edit = TextEdit::singleline(&mut vm.username)
                    .font(egui::TextStyle::Heading)
                    .text_color(Color32::WHITE)
                    .desired_width(220.0)
                    .margin(Vec2::new(10.0, 8.0));
                ui.add_enabled(!vm.is_loading, username_edit);
                ui.add_space(12.0);

                // Password input
                ui.label(RichText::new("Password").size(12.0).color(Color32::WHITE));
                ui.add_space(3.0);
                ui.horizontal(|ui| {
                    let password_edit = TextEdit::singleline(&mut vm.password)
                        .font(egui::TextStyle::Heading)
                        .text_color(Color32::WHITE)
                        .password(!vm.show_password)
                        .desired_width(180.0)
                        .margin(Vec2::new(10.0, 8.0));
                    ui.add_enabled(!vm.is_loading, password_edit);

                    let eye_icon = if vm.show_password {
                        "👁"
                    } else {
                        "👁‍🗨"
                    };
                    if ui.add(Button::new(eye_icon).frame(false)).clicked() {
                        vm.show_password = !vm.show_password;
                    }
                });
                ui.add_space(20.0);

                // Login button or spinner
                if vm.is_loading {
                    ui.spinner();
                    ui.label(
                        RichText::new("Logging in...")
                            .size(12.0)
                            .color(Color32::GRAY),
                    );
                } else {
                    let login_btn = ui.add_sized(
                        [220.0, 40.0],
                        Button::new(RichText::new("Login").size(16.0).color(Color32::WHITE))
                            .fill(Color32::from_rgb(60, 100, 180))
                            .rounding(8.0),
                    );
                    if login_btn.clicked() {
                        vm.login(ctx.clone());
                    }
                }

                // Error message
                if let Some(err) = &vm.error {
                    ui.add_space(10.0);
                    ui.label(
                        RichText::new(err)
                            .size(11.0)
                            .color(Color32::from_rgb(255, 100, 100)),
                    );
                }

                ui.add_space(10.0);
            });
        });
}

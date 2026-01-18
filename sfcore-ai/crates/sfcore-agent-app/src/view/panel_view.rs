//! Slide-out Panel View
//!
//! Panel appears on the LEFT side when a menu item is clicked.
//! Settings renamed to "SFCore AI Engine" with server control and chat.

use crate::assets::TextureCache;
use crate::ui_helpers::draw_gradient_border;
use crate::viewmodel::{menu_vm::MenuItem, EngineViewModel, MenuViewModel};
use eframe::egui::{self, Color32, CursorIcon, Image, Pos2, RichText, ScrollArea, TextEdit, Vec2};

// ============================================================================
// PANEL SIZE CONSTANTS
// ============================================================================

const PANEL_WIDTH: f32 = 600.0;
const PANEL_HEIGHT: f32 = 380.0;

// ============================================================================

pub fn render_panel(
    ctx: &egui::Context,
    menu_vm: &mut MenuViewModel,
    engine_vm: &mut EngineViewModel,
    textures: &mut TextureCache,
) {
    menu_vm.animate(ctx);

    if !menu_vm.menu_visible || menu_vm.active_menu.is_none() {
        return;
    }

    let window_width = ctx.screen_rect().width();
    let min_width = PANEL_WIDTH + 50.0;
    if window_width < min_width {
        return;
    }

    let menu = menu_vm.active_menu.clone().unwrap();
    let panel_x = 0.0;

    let bg_color = match menu {
        MenuItem::Settings => Color32::from_rgba_unmultiplied(20, 20, 25, 245),
        _ => Color32::from_rgba_unmultiplied(35, 35, 45, 250),
    };

    let _area_response = egui::Area::new(egui::Id::new("slide_panel"))
        .fixed_pos(Pos2::new(panel_x, 0.0))
        .show(ctx, |ui| {
            egui::Frame::none()
                .fill(bg_color)
                .inner_margin(20.0)
                .show(ui, |ui| {
                    ui.set_min_size(Vec2::new(PANEL_WIDTH, PANEL_HEIGHT));
                    ui.set_max_size(Vec2::new(PANEL_WIDTH, PANEL_HEIGHT));

                    // HEADER
                    ui.horizontal(|ui| {
                        // Close button
                        let close_tex = textures.close(ctx);
                        let close_btn = ui.add(
                            egui::ImageButton::new(
                                Image::new(&close_tex).fit_to_exact_size(Vec2::splat(24.0)),
                            )
                            .frame(false),
                        );
                        if close_btn.clicked() {
                            menu_vm.hide_menu();
                        }
                        close_btn.on_hover_text("Close Panel");

                        ui.add_space(15.0);

                        // Title
                        match menu {
                            MenuItem::Dashboard => {
                                ui.heading(
                                    RichText::new("Dashboard").size(22.0).color(Color32::WHITE),
                                );
                            }
                            MenuItem::Settings => {
                                ui.heading(
                                    RichText::new("SFCore AI Engine")
                                        .size(22.0)
                                        .color(Color32::WHITE),
                                );

                                // Play/Stop button at RIGHT side
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        let icon = if engine_vm.is_running() {
                                            textures.stop(ctx)
                                        } else {
                                            textures.start(ctx)
                                        };

                                        let btn = ui.add(
                                            egui::ImageButton::new(
                                                Image::new(&icon)
                                                    .fit_to_exact_size(Vec2::splat(24.0)),
                                            )
                                            .frame(false),
                                        );

                                        // Add border only for START button (when not running)
                                        if !engine_vm.is_running() {
                                            crate::ui_helpers::draw_gradient_border(
                                                ui.painter(),
                                                btn.rect.expand(2.0),
                                                1.0,
                                            );
                                        }

                                        if btn.hovered() {
                                            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
                                        }

                                        if btn.clicked() {
                                            engine_vm.toggle_server();
                                        }

                                        let tooltip = if engine_vm.is_running() {
                                            "Stop Server"
                                        } else {
                                            "Start Server"
                                        };
                                        btn.on_hover_text(tooltip);

                                        ui.add_space(5.0);

                                        // Clear Chat Button
                                        let clear_btn = ui.add(
                                            egui::Button::new(RichText::new("🗑").size(16.0))
                                                .fill(Color32::from_rgb(80, 60, 60))
                                                .min_size(Vec2::new(28.0, 28.0)),
                                        );

                                        if clear_btn.hovered() {
                                            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
                                        }

                                        if clear_btn.clicked() {
                                            engine_vm.clear_chat();
                                        }
                                        clear_btn.on_hover_text("Clear Chat");

                                        ui.add_space(10.0);

                                        // Status Indicator (Left of button)
                                        use crate::viewmodel::engine_vm::ServerStatus;
                                        let (status_text, status_color) = match &engine_vm.status {
                                            ServerStatus::Stopped => {
                                                ("Server Stopped", Color32::GRAY)
                                            }
                                            ServerStatus::Starting => {
                                                ("Starting...", Color32::YELLOW)
                                            }
                                            ServerStatus::WarmingUp => (
                                                "Warming up model...",
                                                Color32::from_rgb(255, 165, 0),
                                            ),
                                            ServerStatus::Running => {
                                                ("Server Ready", Color32::GREEN)
                                            }
                                            ServerStatus::Error(_) => ("Error", Color32::RED),
                                        };

                                        ui.label(
                                            RichText::new(status_text)
                                                .size(12.0)
                                                .color(status_color),
                                        );

                                        // Signal icon only when running
                                        if engine_vm.is_ready() {
                                            let signal_tex = textures.signal(ctx);
                                            ui.add(
                                                Image::new(&signal_tex)
                                                    .fit_to_exact_size(Vec2::splat(16.0)),
                                            );

                                            // Token Speed Metrics
                                            if let Some(metrics) = &engine_vm.current_metrics {
                                                if metrics.tokens_generated > 0 {
                                                    ui.add_space(5.0);
                                                    ui.label(
                                                        RichText::new(format!(
                                                            "⚡ {} tok/s",
                                                            metrics.tokens_generated
                                                        ))
                                                        .size(10.0)
                                                        .color(Color32::from_rgb(100, 200, 255)),
                                                    );
                                                }
                                            }
                                        }
                                    },
                                );
                            }
                            MenuItem::Reports => {
                                ui.heading(
                                    RichText::new("Reports").size(22.0).color(Color32::WHITE),
                                );
                            }
                        }
                    });

                    ui.add_space(5.0);
                    ui.separator();
                    ui.add_space(10.0);

                    // CONTENT
                    match menu {
                        MenuItem::Dashboard => render_dashboard(ui),
                        MenuItem::Settings => render_engine(ui, ctx, engine_vm, textures),
                        MenuItem::Reports => render_reports(ui),
                    }
                });
        });
}

// ============================================================================
// PANEL CONTENT FUNCTIONS
// ============================================================================

fn render_dashboard(ui: &mut egui::Ui) {
    ui.label(RichText::new("Welcome to SFCore Agent Dashboard").color(Color32::LIGHT_GRAY));
    ui.add_space(20.0);

    ui.group(|ui| {
        ui.label(
            RichText::new("Quick Stats")
                .size(16.0)
                .color(Color32::WHITE),
        );
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            stat_card(ui, "Users", "128");
            ui.add_space(10.0);
            stat_card(ui, "Active", "42");
        });
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            stat_card(ui, "Tasks", "15");
            ui.add_space(10.0);
            stat_card(ui, "Pending", "7");
        });
    });
}

/// SFCore AI Engine view with server control and chat
fn render_engine(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    vm: &mut EngineViewModel,
    textures: &mut TextureCache,
) {
    // Log Panel (Small area for logs)
    egui::Frame::none()
        .fill(Color32::from_rgb(10, 10, 15))
        .rounding(4.0)
        .inner_margin(5.0)
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ScrollArea::vertical()
                .max_height(60.0) // Reduced from 80 to raise textarea
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    if vm.logs.is_empty() {
                        ui.label(
                            RichText::new("System logs will appear here...")
                                .size(10.0)
                                .color(Color32::DARK_GRAY),
                        );
                    } else {
                        for log in &vm.logs {
                            ui.label(
                                RichText::new(log)
                                    .size(10.0)
                                    .monospace()
                                    .color(Color32::LIGHT_GRAY),
                            );
                        }
                    }
                });
        });

    ui.add_space(5.0); // Reduced from 10

    // Response area (scrollable, takes most of the space)
    // Layout: Response at TOP, Input at BOTTOM (fixed height)
    ui.vertical(|ui| {
        // Calculate dynamic heights
        let input_height = 90.0; // Reduced from 100 to fit better
        let response_height = ui.available_height() - input_height;

        // 1. Response ScrollArea (Top)
        egui::Frame::none()
            .fill(Color32::from_rgba_unmultiplied(15, 15, 20, 200))
            .rounding(4.0)
            .inner_margin(6.0) // Reduced from 10
            .show(ui, |ui| {
                ScrollArea::vertical()
                    .max_height(response_height)
                    .min_scrolled_height(response_height)
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        ui.set_min_height(response_height);

                        if vm.messages.is_empty() {
                            ui.vertical_centered(|ui| {
                                ui.add_space(response_height / 3.0);
                                ui.label(
                                    RichText::new("SFCore AI Engine Ready")
                                        .size(16.0)
                                        .color(Color32::LIGHT_GRAY),
                                );
                                ui.label(
                                    RichText::new("Start by typing a message below...")
                                        .color(Color32::DARK_GRAY),
                                );
                            });
                        } else {
                            for msg in &vm.messages {
                                ui.add_space(5.0);
                                match msg.role {
                                    crate::viewmodel::engine_vm::MessageRole::User => {
                                        let max_bubble_width = ui.available_width() * 0.75;
                                        ui.horizontal(|ui| {
                                            ui.with_layout(
                                                egui::Layout::right_to_left(egui::Align::Min),
                                                |ui| {
                                                    ui.set_max_width(max_bubble_width);
                                                    egui::Frame::none()
                                                        .fill(Color32::from_rgb(60, 100, 160))
                                                        .rounding(8.0)
                                                        .inner_margin(8.0)
                                                        .show(ui, |ui| {
                                                            ui.set_max_width(
                                                                max_bubble_width - 16.0,
                                                            );
                                                            ui.add(
                                                                egui::Label::new(
                                                                    RichText::new(&msg.content)
                                                                        .color(Color32::WHITE),
                                                                )
                                                                .wrap(),
                                                            );
                                                        });
                                                },
                                            );
                                        });
                                    }
                                    crate::viewmodel::engine_vm::MessageRole::Assistant => {
                                        let max_bubble_width = ui.available_width() * 0.85;
                                        ui.horizontal(|ui| {
                                            ui.set_max_width(max_bubble_width);
                                            egui::Frame::none()
                                                .fill(Color32::from_gray(35))
                                                .rounding(8.0)
                                                .inner_margin(8.0)
                                                .show(ui, |ui| {
                                                    ui.set_max_width(max_bubble_width - 16.0);
                                                    // Parse thinking tags
                                                    render_message_with_thinking(ui, &msg.content);
                                                });
                                        });
                                    }
                                    crate::viewmodel::engine_vm::MessageRole::System => {
                                        ui.vertical_centered(|ui| {
                                            ui.label(
                                                RichText::new(&msg.content)
                                                    .size(11.0)
                                                    .color(Color32::GRAY),
                                            );
                                        });
                                    }
                                }
                            }

                            // Typing Indicator (if loading)
                            if vm.is_loading {
                                ui.add_space(5.0);
                                ui.horizontal(|ui| {
                                    egui::Frame::none()
                                        .fill(Color32::from_gray(35))
                                        .rounding(8.0)
                                        .inner_margin(8.0)
                                        .show(ui, |ui| {
                                            let time = ui.input(|i| i.time);
                                            let alpha = (time * 3.0).sin().abs() as f32;
                                            ui.label(
                                                RichText::new("▋").color(
                                                    Color32::LIGHT_GRAY.gamma_multiply(alpha),
                                                ),
                                            );
                                        });
                                });
                            }
                        }
                    });
            });

        ui.add_space(5.0); // Reduced from 10.0 to raise textarea

        // 2. Chat Input Area (Bottom)
        ui.horizontal(|ui| {
            let input_width = ui.available_width() - 50.0;

            // Textarea with scroll (Taller: 4 rows)
            let text_edit = TextEdit::multiline(&mut vm.chat_input)
                .desired_width(input_width)
                .desired_rows(4) // Increased from 2
                .text_color(Color32::WHITE)
                .hint_text("Type your message...");

            let response = ui.add(text_edit);

            // Draw gradient border around textarea
            let border_rect = response.rect.expand(2.0);
            draw_gradient_border(ui.painter(), border_rect, 1.0);

            // Send button with Icon
            let send_tex = textures.send(ctx);
            let send_btn = ui.add(
                egui::Button::image(Image::new(&send_tex).fit_to_exact_size(Vec2::splat(20.0)))
                    .fill(Color32::from_rgb(60, 100, 160))
                    .min_size(Vec2::new(40.0, 75.0)), // Match taller input
            );

            if send_btn.hovered() {
                ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
            }

            // Handle Send on Click OR Enter (only if server ready)
            let can_send = vm.is_ready() && !vm.chat_input.trim().is_empty();
            if can_send
                && (send_btn.clicked()
                    || (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))))
            {
                vm.send_message();
            }
        });
    });
}

fn render_reports(ui: &mut egui::Ui) {
    ui.label(RichText::new("Generate and view reports").color(Color32::LIGHT_GRAY));
    ui.add_space(20.0);

    if ui
        .add(
            egui::Button::new(RichText::new("📄 Generate Report").color(Color32::WHITE))
                .fill(Color32::from_rgb(60, 100, 160))
                .min_size(Vec2::new(150.0, 40.0)),
        )
        .clicked()
    {
        // TODO: Implement report generation
    }
}

fn stat_card(ui: &mut egui::Ui, label: &str, value: &str) {
    egui::Frame::none()
        .fill(Color32::from_rgba_unmultiplied(60, 60, 80, 200))
        .rounding(8.0)
        .inner_margin(15.0)
        .show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(RichText::new(value).size(32.0).color(Color32::WHITE));
                ui.label(RichText::new(label).size(12.0).color(Color32::GRAY));
            });
        });
}

/// Renders message content with collapsible thinking sections
fn render_message_with_thinking(ui: &mut egui::Ui, content: &str) {
    // Check if content has <think>...</think> tags
    let think_start = "<think>";
    let think_end = "</think>";

    if let Some(start_idx) = content.find(think_start) {
        if let Some(end_idx) = content.find(think_end) {
            // Extract parts
            let before_think = &content[..start_idx];
            let thinking_content = &content[start_idx + think_start.len()..end_idx];
            let after_think = &content[end_idx + think_end.len()..];

            // Render text before thinking (with wrap)
            if !before_think.trim().is_empty() {
                ui.add(
                    egui::Label::new(RichText::new(before_think.trim()).color(Color32::LIGHT_GRAY))
                        .wrap(),
                );
            }

            // Render collapsible thinking section
            egui::CollapsingHeader::new(
                RichText::new("💭 Thinking...")
                    .size(12.0)
                    .color(Color32::from_rgb(150, 150, 180))
                    .italics(),
            )
            .default_open(false)
            .show(ui, |ui| {
                egui::Frame::none()
                    .fill(Color32::from_rgba_unmultiplied(40, 40, 50, 150))
                    .rounding(4.0)
                    .inner_margin(6.0)
                    .show(ui, |ui| {
                        ui.add(
                            egui::Label::new(
                                RichText::new(thinking_content.trim())
                                    .size(11.0)
                                    .color(Color32::GRAY)
                                    .italics(),
                            )
                            .wrap(),
                        );
                    });
            });

            // Render text after thinking (with wrap)
            if !after_think.trim().is_empty() {
                ui.add_space(4.0);
                ui.add(
                    egui::Label::new(RichText::new(after_think.trim()).color(Color32::LIGHT_GRAY))
                        .wrap(),
                );
            }
        } else {
            // No end tag, render normally with wrap
            ui.add(egui::Label::new(RichText::new(content).color(Color32::LIGHT_GRAY)).wrap());
        }
    } else {
        // No thinking tags, render normally with wrap
        ui.add(egui::Label::new(RichText::new(content).color(Color32::LIGHT_GRAY)).wrap());
    }
}

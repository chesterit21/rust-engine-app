//! ============================================================================
//! MENU VIEW (LEGACY - Tidak aktif digunakan)
//! ============================================================================
//!
//! File ini adalah versi lama dari menu view.
//! Sekarang digantikan oleh dock_view.rs + panel_view.rs
//!
//! ## Status: DEPRECATED (tidak digunakan)
//! Warnings "function `render_menu` is never used" adalah normal.
//!
//! ## Notes:
//! - Code ini disimpan sebagai referensi
//! - Pattern animasi yang sama digunakan di panel_view.rs

use crate::viewmodel::{menu_vm::MenuItem, MenuViewModel};
use eframe::egui::{self, Color32, Pos2, RichText, Vec2};

pub fn render_menu(ctx: &egui::Context, vm: &mut MenuViewModel) {
    // Jalankan animasi setiap frame
    // animate() mengupdate vm.offset berdasarkan vm.target_offset
    vm.animate(ctx);

    // ========================================================================
    // VISIBILITY CHECK
    // ========================================================================
    // Early exit jika menu tidak visible DAN animasi sudah selesai
    // offset >= 799.0 berarti panel sudah keluar layar sepenuhnya
    // Kondisi ini menghemat resource karena tidak perlu render apapun
    if !vm.menu_visible && vm.offset >= 799.0 {
        return;
    }

    // Area untuk menu overlay
    // egui::Area = floating container yang bisa di-posisi-kan di mana saja
    egui::Area::new(egui::Id::new("menu_area"))
        // fixed_pos: Posisi absolut di layar
        // screen_width - 250 + offset = slide dari kanan ke kiri
        .fixed_pos(Pos2::new(
            ctx.screen_rect().width() - 250.0 + vm.offset,
            50.0, // Y position dari atas
        ))
        .show(ctx, |ui| {
            egui::Frame::none()
                // ============================================================
                // FRAME STYLING
                // ============================================================
                // fill: Warna background dengan format RGBA (Red, Green, Blue, Alpha)
                // - (40, 40, 40, 240): Dark gray, semi-transparan (240/255 = ~94%)
                // Semakin tinggi Alpha, semakin solid warnanya
                .fill(Color32::from_rgba_unmultiplied(40, 40, 40, 240))
                // rounding: Border radius dalam pixel (sudut melengkung)
                // Nilai 8.0 = sudut sedikit rounded
                .rounding(8.0)
                // inner_margin: Padding di dalam frame (jarak konten dari tepi)
                // Nilai 15.0 = 15 pixel padding di semua sisi
                .inner_margin(15.0)
                .show(ui, |ui| {
                    // ========================================================
                    // CONTAINER SIZE
                    // ========================================================
                    // set_min_size: Ukuran minimum container
                    // Vec2::new(width, height) dalam pixel
                    // Container bisa lebih besar dari ini, tapi tidak lebih kecil
                    ui.set_min_size(Vec2::new(220.0, 400.0));

                    ui.heading(RichText::new("Menu").color(Color32::WHITE));
                    // separator: Garis horizontal pemisah
                    ui.separator();
                    // add_space: Menambah jarak vertikal dalam pixel
                    ui.add_space(10.0);

                    // Menu buttons menggunakan selectable_label
                    // Parameter pertama: bool apakah item ini selected
                    // Parameter kedua: RichText untuk label
                    if ui
                        .selectable_label(
                            vm.active_menu == Some(MenuItem::Dashboard),
                            RichText::new("📊 Dashboard")
                                .size(16.0)
                                .color(Color32::WHITE),
                        )
                        .clicked()
                    {
                        vm.select_menu(MenuItem::Dashboard);
                    }

                    ui.add_space(5.0); // Jarak 5px antar menu item

                    if ui
                        .selectable_label(
                            vm.active_menu == Some(MenuItem::Settings),
                            RichText::new("⚙️ Settings")
                                .size(16.0)
                                .color(Color32::WHITE),
                        )
                        .clicked()
                    {
                        vm.select_menu(MenuItem::Settings);
                    }

                    ui.add_space(5.0);

                    if ui
                        .selectable_label(
                            vm.active_menu == Some(MenuItem::Reports),
                            RichText::new("📈 Reports").size(16.0).color(Color32::WHITE),
                        )
                        .clicked()
                    {
                        vm.select_menu(MenuItem::Reports);
                    }

                    ui.add_space(20.0); // Jarak lebih besar sebelum close button

                    if ui.button("Close Menu").clicked() {
                        vm.hide_menu();
                    }
                });
        });

    // ========================================================================
    // SECONDARY PANEL - Konten detail menu
    // ========================================================================
    // Panel kedua yang muncul di kiri menu utama
    // Menampilkan konten berdasarkan menu yang dipilih
    if let Some(menu) = &vm.active_menu {
        egui::Area::new(egui::Id::new("menu_content"))
            // fixed_pos: Posisi absolut di layar (dalam pixel)
            // - screen_width - 520: Posisi X dari kiri (520px dari kanan)
            // - secondary_offset: Animasi slide (800 → 0 saat muncul)
            .fixed_pos(Pos2::new(
                ctx.screen_rect().width() - 520.0 + vm.secondary_offset,
                // - 50.0: Posisi Y dari atas layar
                50.0,
            ))
            .show(ctx, |ui| {
                egui::Frame::none()
                    // Warna berbeda untuk Settings (lebih gelap)
                    .fill(match menu {
                        MenuItem::Settings => Color32::from_rgba_unmultiplied(20, 20, 20, 220),
                        _ => Color32::from_rgba_unmultiplied(50, 50, 60, 240),
                    })
                    .rounding(8.0)
                    // inner_margin: Padding di dalam frame (15px semua sisi)
                    .inner_margin(15.0)
                    .show(ui, |ui| {
                        // set_min_size: Ukuran minimum panel content
                        // Vec2::new(lebar, tinggi) dalam pixel
                        ui.set_min_size(Vec2::new(250.0, 400.0));

                        match menu {
                            MenuItem::Dashboard => {
                                ui.heading(RichText::new("📊 Dashboard").color(Color32::WHITE));
                                ui.separator();
                                ui.label(
                                    RichText::new("Dashboard content here...")
                                        .color(Color32::LIGHT_GRAY),
                                );
                            }
                            MenuItem::Settings => {
                                ui.heading(RichText::new("⚙️ Settings").color(Color32::WHITE));
                                ui.label(
                                    RichText::new("(Semi-transparent dark mode)")
                                        .color(Color32::GRAY),
                                );
                                ui.separator();
                                ui.label(
                                    RichText::new("Settings content here...")
                                        .color(Color32::LIGHT_GRAY),
                                );
                            }
                            MenuItem::Reports => {
                                ui.heading(RichText::new("📈 Reports").color(Color32::WHITE));
                                ui.separator();
                                ui.label(
                                    RichText::new("Reports content here...")
                                        .color(Color32::LIGHT_GRAY),
                                );
                            }
                        }
                    });
            });
    }
}

/// Render "Show Menu" button (tidak digunakan dalam dock UI)
pub fn render_menu_button(ui: &mut egui::Ui, vm: &mut MenuViewModel) {
    if ui.button("☰ Menu").clicked() {
        if vm.menu_visible {
            vm.hide_menu();
        } else {
            vm.show_menu();
        }
    }
}

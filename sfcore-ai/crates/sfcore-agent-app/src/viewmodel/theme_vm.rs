//! ============================================================================
//! THEME VIEW MODEL
//! ============================================================================
//!
//! ViewModel untuk mengatur state dan animasi pada Theme Picker Window.
//!
//! ## Dependencies (File yang bergantung pada module ini):
//! - `src/app.rs` → Menyimpan instance ThemeViewModel
//! - `src/view/theme_view.rs` → Menggunakan property offset untuk animasi slide
//! - `src/view/dock_view.rs` → Menggunakan window_visible untuk toggle
//!
//! ## Impact (Dampak perubahan):
//! - Mengubah `offset` → Mempengaruhi posisi X panel theme
//! - Mengubah `animation_speed` → Mempengaruhi kecepatan animasi slide
//! - Mengubah `target_offset` → Menentukan posisi akhir animasi

use crate::theme::Theme;
use eframe::egui;

pub struct ThemeViewModel {
    /// Theme aktif saat ini (Light/Dark)
    /// Impact: Digunakan di theme_view.rs untuk highlight button aktif
    pub theme: Theme,

    /// Apakah window theme picker sedang visible
    /// Impact: Jika false, window tidak di-render sama sekali (hemat resource)
    pub window_visible: bool,

    // ========================================================================
    // ANIMATION PROPERTIES - Mengatur animasi slide-in/slide-out
    // ========================================================================
    /// Posisi offset X saat ini (dalam pixel)
    /// - Nilai 0 = Panel sudah sepenuhnya muncul (visible)
    /// - Nilai 150 = Panel tersembunyi di luar layar (hidden)
    /// Impact: Digunakan di theme_view.rs untuk menghitung posisi fixed_pos()
    /// Dependency: theme_view.rs line ~26 → popup_x calculation
    pub offset: f32,

    /// Target posisi offset yang ingin dicapai
    /// - Set ke 0 saat show_window() → panel akan slide masuk
    /// - Set ke 150 saat hide_window() → panel akan slide keluar
    /// Impact: Menentukan arah dan tujuan animasi
    pub target_offset: f32,

    /// Kecepatan animasi (semakin besar = semakin cepat)
    /// - Default: 10.0 (smooth animation)
    /// - Nilai < 5.0: animasi lambat
    /// - Nilai > 15.0: animasi sangat cepat, hampir instant
    /// Impact: Mempengaruhi durasi total animasi slide
    animation_speed: f32,

    /// Timestamp terakhir kali animate() dipanggil
    /// Digunakan untuk menghitung delta time (frame-rate independent)
    last_update: Option<f64>,
}

impl Default for ThemeViewModel {
    fn default() -> Self {
        Self::new()
    }
}

impl ThemeViewModel {
    pub fn new() -> Self {
        Self {
            theme: Theme::Light,

            // Window hidden by default
            window_visible: false,

            // ================================================================
            // INITIAL ANIMATION STATE
            // ================================================================
            // offset = 150.0 → Panel dimulai di posisi tersembunyi
            // target_offset = 150.0 → Target awal juga tersembunyi
            // Saat show_window() dipanggil, target_offset berubah ke 0
            // ================================================================
            offset: 150.0,
            target_offset: 150.0,

            // animation_speed = 10.0 → Kecepatan animasi default
            // Rumus: offset += (target - current) * speed * deltaTime
            animation_speed: 10.0,

            last_update: None,
        }
    }

    /// ========================================================================
    /// SHOW WINDOW - Membuka theme picker window dengan animasi slide-in
    /// ========================================================================
    /// Dipanggil dari: dock_view.rs saat user klik tombol theme 🎨
    ///
    /// Cara kerja:
    /// 1. Set window_visible = true → window mulai di-render
    /// 2. offset = 150.0 → Reset posisi ke hidden (untuk animasi fresh)
    /// 3. target_offset = 0.0 → Animasi akan bergerak dari 150 ke 0
    ///
    /// Impact: Panel theme akan slide dari kanan ke kiri
    pub fn show_window(&mut self) {
        self.window_visible = true;
        self.offset = 150.0; // Mulai dari posisi hidden
        self.target_offset = 0.0; // Animasi menuju posisi visible
    }

    /// ========================================================================
    /// HIDE WINDOW - Menutup theme picker window dengan animasi slide-out
    /// ========================================================================
    /// Dipanggil dari: theme_view.rs saat user klik close button atau pilih theme
    ///
    /// Cara kerja:
    /// 1. target_offset = 150.0 → Animasi akan bergerak dari current ke 150
    /// 2. Setelah animasi selesai, animate() akan set window_visible = false
    ///
    /// Impact: Panel theme akan slide dari kiri ke kanan lalu hilang
    pub fn hide_window(&mut self) {
        self.target_offset = 150.0; // Animasi menuju posisi hidden
    }

    /// Set theme dan otomatis tutup window
    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
        self.hide_window();
    }

    /// ========================================================================
    /// ANIMATE - Frame-rate independent animation loop
    /// ========================================================================
    /// Dipanggil setiap frame dari: app.rs → render_theme() → animate()
    ///
    /// Cara kerja:
    /// 1. Hitung delta time (waktu sejak frame terakhir)
    /// 2. Interpolasi offset menuju target_offset
    /// 3. Jika animasi belum selesai, request repaint untuk frame berikutnya
    /// 4. Jika animasi selesai dan target = hidden, set window_visible = false
    ///
    /// Rumus animasi: offset += (target - offset) * speed * deltaTime
    /// Ini menghasilkan "ease-out" effect (cepat di awal, lambat di akhir)
    pub fn animate(&mut self, ctx: &egui::Context) {
        if !self.window_visible {
            return;
        }

        // Hitung delta time untuk frame-rate independence
        let current_time = ctx.input(|i| i.time);
        let delta = self
            .last_update
            .map(|last| (current_time - last) as f32)
            .unwrap_or(0.016); // Default ~60fps

        self.last_update = Some(current_time);

        // Interpolasi menuju target
        let diff = self.target_offset - self.offset;
        if diff.abs() > 0.5 {
            // Masih animating
            self.offset += diff * self.animation_speed * delta;
            ctx.request_repaint(); // Request frame berikutnya
        } else {
            // Animasi selesai
            self.offset = self.target_offset;

            // Auto-hide jika target adalah posisi hidden (150+)
            if self.target_offset >= 149.0 {
                self.window_visible = false;
            }
        }
    }
}

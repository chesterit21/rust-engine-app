//! ============================================================================
//! MENU VIEW MODEL
//! ============================================================================
//!
//! ViewModel untuk mengatur state dan animasi pada Menu Panel (Dashboard, Settings, Reports).
//!
//! ## Dependencies (File yang bergantung pada module ini):
//! - `src/app.rs` → Menyimpan instance MenuViewModel
//! - `src/view/panel_view.rs` → Menggunakan offset untuk animasi slide panel
//! - `src/view/dock_view.rs` → Menggunakan active_menu untuk highlight icon aktif
//!
//! ## Impact (Dampak perubahan):
//! - Mengubah `offset` → Mempengaruhi posisi X slide panel
//! - Mengubah `active_menu` → Menentukan panel mana yang ditampilkan
//! - Mengubah `animation_speed` → Mempengaruhi kecepatan animasi

use eframe::egui;

/// ============================================================================
/// MENU ITEM ENUM - Daftar menu yang tersedia
/// ============================================================================
/// Digunakan untuk identify panel mana yang aktif/dipilih
/// Impact: Menentukan konten apa yang di-render di panel_view.rs
#[derive(Debug, Clone, PartialEq)]
pub enum MenuItem {
    Dashboard, // Panel dashboard → render_dashboard()
    Settings,  // Panel settings → render_settings()
    Reports,   // Panel reports → render_reports()
}

pub struct MenuViewModel {
    /// Apakah menu panel sedang visible
    /// Impact: Jika false, panel tidak di-render (hemat resource)
    /// Diubah oleh: show_menu(), hide_menu(), animate()
    pub menu_visible: bool,

    /// Menu item yang sedang aktif/dipilih
    /// - Some(MenuItem::Dashboard) → Panel dashboard ditampilkan
    /// - None → Tidak ada panel yang ditampilkan
    /// Impact: Digunakan di dock_view untuk highlight icon, panel_view untuk konten
    pub active_menu: Option<MenuItem>,

    // ========================================================================
    // PRIMARY ANIMATION - Animasi panel utama (slide-in/out)
    // ========================================================================
    /// Posisi offset X saat ini untuk panel utama (dalam pixel)
    /// - Nilai 0 = Panel sudah sepenuhnya muncul (visible)
    /// - Nilai 800 = Panel tersembunyi di luar layar ke kanan (hidden)
    /// Impact: Digunakan di panel_view.rs → panel_x = 0.0 - offset
    /// Dependency: panel_view.rs line ~22 → let panel_x = 0.0 - menu_vm.offset
    pub offset: f32,

    /// Target posisi offset yang ingin dicapai untuk panel utama
    /// - Set ke 0 saat show_menu() → panel akan slide masuk dari kanan
    /// - Set ke 800 saat hide_menu() → panel akan slide keluar ke kanan
    pub target_offset: f32,

    // ========================================================================
    // SECONDARY ANIMATION - Animasi transisi antar panel (tidak digunakan saat ini)
    // ========================================================================
    /// Offset untuk animasi secondary (reserved untuk future use)
    pub secondary_offset: f32,

    /// Target secondary offset
    pub secondary_target: f32,

    // ========================================================================
    // ANIMATION CONFIG
    // ========================================================================
    /// Kecepatan animasi (semakin besar = semakin cepat)
    /// - Default: 8.0 (smooth animation)
    /// - Nilai < 5.0: animasi sangat lambat
    /// - Nilai > 12.0: animasi cepat
    /// Impact: Mempengaruhi durasi total animasi slide
    /// Rumus: offset += (target - current) * speed * deltaTime
    animation_speed: f32,

    /// Timestamp terakhir kali animate() dipanggil
    /// Digunakan untuk menghitung delta time (frame-rate independent)
    /// Tanpa ini, animasi akan berbeda kecepatannya di monitor 60Hz vs 144Hz
    last_update: Option<f64>,
}

impl Default for MenuViewModel {
    fn default() -> Self {
        Self::new()
    }
}

impl MenuViewModel {
    pub fn new() -> Self {
        Self {
            menu_visible: false,
            active_menu: None,

            // ================================================================
            // INITIAL ANIMATION STATE
            // ================================================================
            // offset = 800.0 → Panel dimulai di posisi tersembunyi (di luar layar)
            // 800 dipilih karena mendekati lebar panel (PANEL_WIDTH di app.rs)
            // ================================================================
            offset: 680.0,
            target_offset: 700.0,

            // Secondary animation (reserved)
            secondary_offset: 680.0,
            secondary_target: 700.0,

            // animation_speed = 8.0 → Sedikit lebih lambat dari theme (10.0)
            // Karena panel lebih besar, animasi lebih lambat terasa lebih smooth
            animation_speed: 8.0,

            last_update: None,
        }
    }

    /// ========================================================================
    /// SHOW MENU - Membuka panel menu dengan animasi slide-in
    /// ========================================================================
    /// Dipanggil dari: dock_view.rs saat user klik menu icon
    ///
    /// Cara kerja:
    /// 1. Set menu_visible = true → panel mulai di-render
    /// 2. target_offset = 0.0 → Animasi bergerak dari current ke 0
    ///
    /// Impact:
    /// - Panel akan slide dari kanan ke kiri
    /// - Window size akan expand (ditangani di app.rs update_window_size)
    pub fn show_menu(&mut self) {
        self.menu_visible = true;
        // PENTING: Reset offset ke posisi hidden dulu, baru animasi ke 0
        // Tanpa ini, panel bisa langsung muncul tanpa animasi/gap
        self.offset = 800.0;
        self.target_offset = 0.0; // Animasi menuju posisi visible
    }

    /// ========================================================================
    /// HIDE MENU - Menutup panel menu dengan animasi slide-out
    /// ========================================================================
    /// Dipanggil dari: panel_view.rs saat user klik close button
    ///
    /// Cara kerja:
    /// 1. target_offset = 800.0 → Animasi bergerak ke posisi hidden
    /// 2. active_menu = None → Reset pilihan menu
    /// 3. Setelah animasi selesai, animate() set menu_visible = false
    ///
    /// Impact:
    /// - Panel akan slide dari kiri ke kanan lalu hilang
    /// - Window size akan shrink kembali ke dock-only
    pub fn hide_menu(&mut self) {
        self.target_offset = 800.0; // Animasi menuju posisi hidden
        self.secondary_target = 800.0; // Reset secondary juga
        self.active_menu = None; // Clear selected menu
    }

    /// ========================================================================
    /// SELECT MENU - Memilih menu item dan menampilkan panel content-nya
    /// ========================================================================
    /// Dipanggil dari: dock_view.rs saat user klik menu icon
    ///
    /// Cara kerja:
    /// 1. Set active_menu = item → Menentukan panel mana yang ditampilkan
    /// 2. Reset secondary animation (untuk transisi antar panel)
    ///
    /// Impact: panel_view.rs akan render konten sesuai menu yang dipilih
    pub fn select_menu(&mut self, item: MenuItem) {
        self.active_menu = Some(item);
        self.secondary_offset = 800.0;
        self.secondary_target = 0.0;
    }

    /// ========================================================================
    /// ANIMATE - Frame-rate independent animation loop
    /// ========================================================================
    /// Dipanggil setiap frame dari: app.rs → render_panel() → animate()
    ///
    /// Frame-rate Independence:
    /// - Animasi berjalan dengan kecepatan sama di 60Hz, 144Hz, atau Hz apapun
    /// - Caranya: Menggunakan delta time (waktu sejak frame terakhir)
    ///
    /// Rumus: offset += (target - offset) * speed * deltaTime
    /// - Ini menghasilkan "ease-out" effect (cepat di awal, lambat di akhir)
    /// - Semakin dekat ke target, semakin lambat pergerakannya
    ///
    /// Impact:
    /// - Jika animasi ongoing → request_repaint() untuk render frame berikutnya
    /// - Jika offset >= 799 → set menu_visible = false (panel sudah hidden)
    pub fn animate(&mut self, ctx: &egui::Context) {
        // Hitung delta time untuk frame-rate independence
        let current_time = ctx.input(|i| i.time);
        let delta = self
            .last_update
            .map(|last| (current_time - last) as f32)
            .unwrap_or(0.016); // Default ~60fps jika tidak ada data

        self.last_update = Some(current_time);

        let mut needs_repaint = false;

        // ====================================================================
        // ANIMATE PRIMARY OFFSET - Panel slide animasi
        // ====================================================================
        let diff = self.target_offset - self.offset;
        if diff.abs() > 0.5 {
            // Masih animating - interpolasi menuju target
            self.offset += diff * self.animation_speed * delta;
            needs_repaint = true;
        } else {
            // Animasi selesai - snap ke target
            self.offset = self.target_offset;
        }

        // ====================================================================
        // ANIMATE SECONDARY OFFSET - Untuk transisi antar panel (future use)
        // ====================================================================
        let sec_diff = self.secondary_target - self.secondary_offset;
        if sec_diff.abs() > 0.5 {
            self.secondary_offset += sec_diff * self.animation_speed * delta;
            needs_repaint = true;
        } else {
            self.secondary_offset = self.secondary_target;
        }

        // ====================================================================
        // AUTO-HIDE - Jika panel sudah sepenuhnya keluar layar
        // ====================================================================
        if self.offset >= 799.0 {
            self.menu_visible = false;
        }

        // Request repaint hanya jika animasi masih berjalan
        // Ini menghemat resource ketika tidak ada animasi
        if needs_repaint {
            ctx.request_repaint();
        }
    }
}

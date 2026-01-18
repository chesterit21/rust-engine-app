//! UI Helpers
//!
//! Helper functions untuk UI styling dengan smooth gradient border.
//! Warna gradient: Crimson → Green → Blue → Yellow → Crimson (loop)

use eframe::egui::{self, Color32, Pos2, Rect, Stroke};

/// Border colors untuk gradient effect
const CRIMSON: Color32 = Color32::from_rgb(220, 20, 60);
const GREEN: Color32 = Color32::from_rgb(50, 205, 50);
const BLUE: Color32 = Color32::from_rgb(30, 144, 255);
const YELLOW: Color32 = Color32::from_rgb(255, 215, 0);

/// Interpolate between two colors based on t (0.0 to 1.0)
fn lerp_color(c1: Color32, c2: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    Color32::from_rgb(
        ((c1.r() as f32) * (1.0 - t) + (c2.r() as f32) * t) as u8,
        ((c1.g() as f32) * (1.0 - t) + (c2.g() as f32) * t) as u8,
        ((c1.b() as f32) * (1.0 - t) + (c2.b() as f32) * t) as u8,
    )
}

/// Get gradient color based on position around the rectangle (0.0 to 1.0)
fn get_gradient_color(t: f32) -> Color32 {
    let t = t % 1.0;

    if t < 0.25 {
        lerp_color(CRIMSON, GREEN, t * 4.0)
    } else if t < 0.5 {
        lerp_color(GREEN, BLUE, (t - 0.25) * 4.0)
    } else if t < 0.75 {
        lerp_color(BLUE, YELLOW, (t - 0.5) * 4.0)
    } else {
        lerp_color(YELLOW, CRIMSON, (t - 0.75) * 4.0)
    }
}

/// Get a point on the rectangle perimeter based on t (0.0 to 1.0)
fn get_point_on_rect(rect: Rect, t: f32) -> Pos2 {
    let perimeter = 2.0 * (rect.width() + rect.height());
    let dist = t * perimeter;

    if dist < rect.width() {
        Pos2::new(rect.left() + dist, rect.top())
    } else if dist < rect.width() + rect.height() {
        let d = dist - rect.width();
        Pos2::new(rect.right(), rect.top() + d)
    } else if dist < 2.0 * rect.width() + rect.height() {
        let d = dist - rect.width() - rect.height();
        Pos2::new(rect.right() - d, rect.bottom())
    } else {
        let d = dist - 2.0 * rect.width() - rect.height();
        Pos2::new(rect.left(), rect.bottom() - d)
    }
}

/// Draw smooth gradient border (NO glow/shadow - clean border only)
/// Colors blend smoothly around the perimeter: Crimson → Green → Blue → Yellow → loop
pub fn draw_gradient_border(painter: &egui::Painter, rect: Rect, width: f32) {
    let perimeter = 2.0 * (rect.width() + rect.height());
    let step = 2.0;
    let segments = (perimeter / step) as usize;

    // Draw main border only (no glow layers)
    for i in 0..segments {
        let t1 = i as f32 / segments as f32;
        let t2 = (i + 1) as f32 / segments as f32;

        let p1 = get_point_on_rect(rect, t1);
        let p2 = get_point_on_rect(rect, t2);
        let color = get_gradient_color(t1);

        painter.line_segment([p1, p2], Stroke::new(width, color));
    }
}

//! Assets Module
//!
//! Load and cache image textures from assets folder.

use eframe::egui::{self, ColorImage, TextureHandle, TextureOptions};
use std::collections::HashMap;

/// Embedded image bytes
pub struct EmbeddedImages {
    pub favicon: &'static [u8],
    pub close: &'static [u8],
    pub close_app: &'static [u8],
    pub dashboard: &'static [u8],
    pub speedometer: &'static [u8],
    pub workflow: &'static [u8],
    pub start: &'static [u8],
    pub stop: &'static [u8],
    pub send: &'static [u8],
    pub signal: &'static [u8],
}

impl Default for EmbeddedImages {
    fn default() -> Self {
        Self {
            favicon: include_bytes!("../assets/favicon.ico"),
            close: include_bytes!("../assets/close.png"),
            close_app: include_bytes!("../assets/close_app.png"),
            dashboard: include_bytes!("../assets/icon_dashboard.png"),
            speedometer: include_bytes!("../assets/ai-icon.png"),
            workflow: include_bytes!("../assets/workflow.png"),
            start: include_bytes!("../assets/start.png"),
            stop: include_bytes!("../assets/stop_exit.png"),
            send: include_bytes!("../assets/send.png"),
            signal: include_bytes!("../assets/signal-on.png"),
        }
    }
}

/// Texture cache for loaded images
pub struct TextureCache {
    textures: HashMap<String, TextureHandle>,
    images: EmbeddedImages,
}

impl TextureCache {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
            images: EmbeddedImages::default(),
        }
    }

    /// Load image from bytes and cache it
    fn load_texture(&mut self, ctx: &egui::Context, name: &str, bytes: &[u8]) -> TextureHandle {
        if let Some(tex) = self.textures.get(name) {
            return tex.clone();
        }

        let image = match image::load_from_memory(bytes) {
            Ok(img) => img.to_rgba8(),
            Err(e) => {
                eprintln!("Failed to load image {}: {}", name, e);
                let fallback = ColorImage::new([1, 1], egui::Color32::TRANSPARENT);
                return ctx.load_texture(name, fallback, TextureOptions::default());
            }
        };

        let size = [image.width() as usize, image.height() as usize];
        let pixels = image.into_raw();
        let color_image = ColorImage::from_rgba_unmultiplied(size, &pixels);

        let texture = ctx.load_texture(name, color_image, TextureOptions::default());
        self.textures.insert(name.to_string(), texture.clone());
        texture
    }

    pub fn favicon(&mut self, ctx: &egui::Context) -> TextureHandle {
        let bytes = self.images.favicon;
        self.load_texture(ctx, "favicon", bytes)
    }

    pub fn close(&mut self, ctx: &egui::Context) -> TextureHandle {
        let bytes = self.images.close;
        self.load_texture(ctx, "close", bytes)
    }

    pub fn close_app(&mut self, ctx: &egui::Context) -> TextureHandle {
        let bytes = self.images.close_app;
        self.load_texture(ctx, "close_app", bytes)
    }

    pub fn dashboard(&mut self, ctx: &egui::Context) -> TextureHandle {
        let bytes = self.images.dashboard;
        self.load_texture(ctx, "dashboard", bytes)
    }

    pub fn speedometer(&mut self, ctx: &egui::Context) -> TextureHandle {
        let bytes = self.images.speedometer;
        self.load_texture(ctx, "speedometer", bytes)
    }

    pub fn workflow(&mut self, ctx: &egui::Context) -> TextureHandle {
        let bytes = self.images.workflow;
        self.load_texture(ctx, "workflow", bytes)
    }

    pub fn start(&mut self, ctx: &egui::Context) -> TextureHandle {
        let bytes = self.images.start;
        self.load_texture(ctx, "start", bytes)
    }

    pub fn stop(&mut self, ctx: &egui::Context) -> TextureHandle {
        let bytes = self.images.stop;
        self.load_texture(ctx, "stop", bytes)
    }

    pub fn send(&mut self, ctx: &egui::Context) -> TextureHandle {
        let bytes = self.images.send;
        self.load_texture(ctx, "send", bytes)
    }

    pub fn signal(&mut self, ctx: &egui::Context) -> TextureHandle {
        let bytes = self.images.signal;
        self.load_texture(ctx, "signal", bytes)
    }
}

impl Default for TextureCache {
    fn default() -> Self {
        Self::new()
    }
}

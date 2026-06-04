mod app;
mod atomcode_io;
mod config_io;
mod models;

use app::AtomcodeSwitchApp;
use eframe::egui;
use std::sync::Arc;

fn load_icon() -> Option<egui::IconData> {
    let icon_path = std::path::Path::new("icon.png");
    if !icon_path.exists() {
        return None;
    }

    let img = image::open(icon_path).ok()?;
    let img = img.resize(256, 256, image::imageops::FilterType::Lanczos3);
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();

    Some(egui::IconData {
        rgba: rgba.into_raw(),
        width,
        height,
    })
}

fn main() -> eframe::Result<()> {
    let icon_data = load_icon();

    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([780.0, 560.0])
        .with_min_inner_size([600.0, 400.0]);

    if let Some(icon) = icon_data {
        viewport = viewport.with_icon(Arc::new(icon));
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "AtomCode Switch",
        options,
        Box::new(|cc| Box::new(AtomcodeSwitchApp::new(cc))),
    )
}

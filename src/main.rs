mod app;
mod atomcode_io;
mod config_io;
mod models;

use app::AtomcodeSwitchApp;
use eframe::egui;

fn main() -> eframe::Result<()> {
    let viewport = egui::ViewportBuilder::default()
        .with_inner_size([780.0, 560.0])
        .with_min_inner_size([600.0, 400.0]);

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
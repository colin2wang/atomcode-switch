mod app;
mod atomcode_io;
mod config_io;
mod models;

use app::AtomcodeSwitchApp;
use eframe::egui;
use std::sync::Arc;

fn main() -> eframe::Result<()> {
    // 加载窗口图标（使用轻量 ico crate）
    let icon = {
        let bytes = include_bytes!("../atomcode.ico");
        let icon_dir = ico::IconDir::read(std::io::Cursor::new(bytes))
            .expect("无法解析 atomcode.ico");
        // 选取分辨率最高的图标
        let entry = icon_dir.entries()
            .iter()
            .max_by_key(|e| e.width() * e.height())
            .unwrap();
        let img = entry.decode().expect("无法解码 atomcode.ico");
        egui::IconData {
            rgba: img.rgba_data().to_vec(),
            width: img.width(),
            height: img.height(),
        }
    };

    let viewport = egui::ViewportBuilder::default()
        .with_inner_size([780.0, 560.0])
        .with_min_inner_size([600.0, 400.0])
        .with_icon(Arc::new(icon));

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        &format!("AtomCode Switch v{}", env!("CARGO_PKG_VERSION")),
        options,
        Box::new(|cc| Box::new(AtomcodeSwitchApp::new(cc))),
    )?;
    Ok(())
}
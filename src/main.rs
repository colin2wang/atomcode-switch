#![windows_subsystem = "windows"]

use atomcode_switch::AtomcodeSwitchApp;
use eframe::egui;
use std::sync::Arc;

/// 从内嵌 ICO 中解析窗口图标；解析失败时返回 None，降级为无图标启动，
/// 避免 release(panic=abort) 下因损坏的图标资源直接 abort。
fn load_window_icon() -> Option<egui::IconData> {
    let bytes = include_bytes!("../atomcode.ico");
    let icon_dir = ico::IconDir::read(std::io::Cursor::new(bytes)).ok()?;
    let entry = icon_dir
        .entries()
        .iter()
        .max_by_key(|e| e.width() * e.height())?;
    let img = entry.decode().ok()?;
    Some(egui::IconData {
        rgba: img.rgba_data().to_vec(),
        width: img.width(),
        height: img.height(),
    })
}

fn main() -> eframe::Result<()> {
    let icon = load_window_icon().map(Arc::new);

    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([780.0, 560.0])
        .with_min_inner_size([600.0, 400.0]);
    if let Some(icon) = &icon {
        viewport = viewport.with_icon(icon.clone());
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        &format!("AtomCode Switch v{}", env!("CARGO_PKG_VERSION")),
        options,
        Box::new(move |cc| Box::new(AtomcodeSwitchApp::new(cc, icon.clone()))),
    )?;
    Ok(())
}

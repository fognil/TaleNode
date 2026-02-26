mod actions;
mod app;
mod collab;
mod export;
mod import;
mod integrations;
mod model;
mod scripting;
mod ui;
mod validation;

use app::TaleNodeApp;

fn load_icon() -> Option<egui::IconData> {
    let png_bytes = include_bytes!("../talenode.png");
    let img = image::load_from_memory(png_bytes).ok()?.into_rgba8();
    Some(egui::IconData {
        width: img.width(),
        height: img.height(),
        rgba: img.into_raw(),
    })
}

fn main() -> eframe::Result {
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([1280.0, 720.0])
        .with_min_inner_size([800.0, 500.0]);
    if let Some(icon) = load_icon() {
        viewport = viewport.with_icon(std::sync::Arc::new(icon));
    }
    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "TaleNode — Dialogue Editor",
        options,
        Box::new(|cc| Ok(Box::new(TaleNodeApp::new(cc)))),
    )
}

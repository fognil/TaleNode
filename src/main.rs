mod actions;
mod app;
mod export;
mod import;
mod model;
mod scripting;
mod ui;
mod validation;

use app::TaleNodeApp;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_min_inner_size([800.0, 500.0]),
        ..Default::default()
    };

    eframe::run_native(
        "TaleNode — Dialogue Editor",
        options,
        Box::new(|cc| Ok(Box::new(TaleNodeApp::new(cc)))),
    )
}

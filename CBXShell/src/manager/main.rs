#![windows_subsystem = "windows"]

///! CBXManager - Modern configuration utility for CBXShell
///!
///! Built with egui for a clean, modern interface

mod state;
mod registry_ops;
mod ui;
mod utils;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([360.0, 370.0])
            .with_resizable(false)
            .with_title("CBXShell Manager"),
        ..Default::default()
    };

    eframe::run_native(
        "CBXShell Manager",
        options,
        Box::new(|cc| Ok(Box::new(ui::CBXManagerApp::new(cc)))),
    )
}

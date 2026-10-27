#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

#[cfg(windows)]
mod gui;

#[cfg(windows)]
fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([860.0, 790.0])
            .with_min_inner_size([660.0, 520.0]),
        ..Default::default()
    };
    eframe::run_native(
        "CLON//CONTROL - Instant A704F",
        options,
        Box::new(|cc| Ok(Box::new(gui::Configurator::new(cc)))),
    )
}

#[cfg(not(windows))]
fn main() {
    eprintln!("CLON//CONTROL es una aplicacion para Windows 10/11.");
}

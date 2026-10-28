#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

#[cfg(windows)]
mod gui;

#[cfg(windows)]
fn app_icon() -> eframe::egui::IconData {
    // Produced from the same pixel renderer as the .ico embedded in the PE.
    let rgba = include_bytes!(concat!(env!("OUT_DIR"), "/cloncontrol.rgba"));
    eframe::egui::IconData { rgba: rgba.to_vec(), width: 64, height: 64 }
}

#[cfg(windows)]
fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([900.0, 740.0])
            .with_min_inner_size([700.0, 560.0])
            .with_icon(app_icon()),
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

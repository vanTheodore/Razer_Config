#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

#[cfg(windows)]
mod gui;

/// A crisp 64px mouse glyph drawn as RGBA pixels, independent of OS fonts.
/// Used by the window/taskbar rather than the default eframe icon.
#[cfg(windows)]
fn app_icon() -> eframe::egui::IconData {
    const SIDE: u32 = 64;
    let mut rgba = Vec::with_capacity((SIDE * SIDE * 4) as usize);
    for y in 0..SIDE {
        for x in 0..SIDE {
            let xf = x as f32;
            let yf = y as f32;
            let body = ((xf - 32.0) / 21.0).powi(2) + ((yf - 33.0) / 27.0).powi(2) <= 1.0;
            let wheel = (28.0..=36.0).contains(&xf) && (15.0..=27.0).contains(&yf);
            let seam = (31.0..=33.0).contains(&xf) && (11.0..=35.0).contains(&yf);
            let pixel = if wheel { [15, 24, 32, 255] }
                else if seam { [88, 174, 162, 255] }
                else if body { [100, 224, 196, 255] }
                else { [0, 0, 0, 0] };
            rgba.extend_from_slice(&pixel);
        }
    }
    eframe::egui::IconData { rgba, width: SIDE, height: SIDE }
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

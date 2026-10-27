#[cfg(windows)]
fn main() {
    use cloncontrol_a704f::{device, protocol};
    use std::{fmt::Write as _, fs, path::PathBuf};
    let mut out = String::new();
    writeln!(out, "CLON//CONTROL — diagnóstico HID de solo lectura").unwrap();
    writeln!(out, "VID:PID {:04X}:{:04X}", protocol::VID, protocol::PID).unwrap();
    match device::scan() {
        Ok(devices) if devices.is_empty() => {
            writeln!(out, "No se encontraron interfaces USB coincidentes.").unwrap();
        }
        Ok(devices) => {
            for (i, d) in devices.iter().enumerate() {
                writeln!(out,
                    "\n#{i} {} | Fabricante: {} | MI {} | Usage {:04X}:{:04X} | Candidata: {}",
                    d.product, d.manufacturer, d.interface_number, d.usage_page,
                    d.usage, if d.eligible { "sí" } else { "no" }).unwrap();
                if d.eligible {
                    match device::read_report_descriptor(&d.path) {
                        Ok(bytes) => {
                            let ids = protocol::feature_report_ids(&bytes);
                            writeln!(out, "  Descriptor HID: {} bytes; Feature Report IDs: {:?}",
                                bytes.len(), ids).unwrap();
                            if !ids.contains(&7) && !ids.contains(&0x21) {
                                writeln!(out, "  ATENCIÓN: el descriptor no anuncia los IDs 07 ni 21. No confirma soporte.").unwrap();
                            }
                            if bytes.is_empty() {
                                writeln!(out, "  Descriptor vacío; no es una prueba de compatibilidad.").unwrap();
                            }
                        }
                        Err(e) => { writeln!(out, "  Descriptor no accesible: {e}").unwrap(); }
                    }
                }
            }
        }
        Err(e) => { writeln!(out, "Error de enumeración: {e}").unwrap(); }
    }
    let rgb = protocol::rgb(protocol::Light::Off);
    writeln!(out, "\nTrama HID 8 bytes (no probada): {}", protocol::hex(&rgb)).unwrap();
    writeln!(out, "Trama Linux 16 bytes (no probada en Windows): {:02X?}", 
        protocol::Transport::LinuxReference16.encode(&rgb).unwrap()).unwrap();
    writeln!(out, "No se ha enviado ningún reporte al mouse.").unwrap();

    println!("{out}");
    let directory = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."))
        .join("ClonControl");
    if fs::create_dir_all(&directory).is_ok() {
        let path = directory.join("diagnostico.txt");
        if fs::write(&path, out).is_ok() {
            println!("Informe guardado en: {}", path.display());
            println!("No incluye número de serie ni ruta USB.");
        }
    }
    if !std::env::args().any(|x| x == "--no-pause") {
        println!("Pulsa Enter para cerrar.");
        let mut input = String::new();
        let _ = std::io::stdin().read_line(&mut input);
    }
}
#[cfg(not(windows))]
fn main() { println!("Diagnóstico disponible en Windows."); }

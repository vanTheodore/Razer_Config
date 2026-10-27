#[cfg(windows)]
fn main() {
    use cloncontrol_a704f::{device, protocol};
    use std::{fmt::Write as _, fs, path::PathBuf};
    let mut out = String::new();
    writeln!(out, "CLON//CONTROL — diagnóstico HID de solo lectura (todas las colecciones)").unwrap();
    writeln!(out, "VID:PID {:04X}:{:04X}", protocol::VID, protocol::PID).unwrap();
    writeln!(out, "Esta herramienta NO envía reportes, no cambia DPI, iluminación ni firmware.").unwrap();
    match device::scan() {
        Ok(devices) if devices.is_empty() => {
            writeln!(out, "No se encontraron interfaces USB coincidentes.").unwrap();
        }
        Ok(devices) => {
            writeln!(out, "Colecciones detectadas: {}", devices.len()).unwrap();
            for (i, d) in devices.iter().enumerate() {
                writeln!(out, "\n#{i} {} | Fabricante: {} | MI {} | Usage {:04X}:{:04X}",
                    d.product, d.manufacturer, d.interface_number, d.usage_page, d.usage).unwrap();
                writeln!(out, "  Selección actual del configurador: {}",
                    if d.eligible { "mouse (no confirma escritura)" } else { "excluida (puede ser interfaz HID adicional)" }).unwrap();
                match device::read_report_descriptor(&d.path) {
                    Ok(bytes) => {
                        let inv = protocol::report_inventory(&bytes);
                        writeln!(out, "  Descriptor: {} bytes", bytes.len()).unwrap();
                        writeln!(out, "  Input Report IDs: {:?}", inv.input).unwrap();
                        writeln!(out, "  Output Report IDs: {:?}", inv.output).unwrap();
                        writeln!(out, "  Feature Report IDs: {:?}", inv.feature).unwrap();
                        if inv.feature.is_empty() {
                            writeln!(out, "  Sin Feature Reports declarados en ESTA colección.").unwrap();
                        }
                        if d.usage_page >= 0xFF00 {
                            writeln!(out, "  Colección definida por el fabricante; requiere análisis antes de utilizarla.").unwrap();
                        }
                        writeln!(out, "  Descriptor hexadecimal (16 bytes por línea):").unwrap();
                        for (idx, chunk) in bytes.chunks(16).enumerate() {
                            let hex = chunk.iter().map(|b| format!("{b:02X}")).collect::<Vec<_>>().join(" ");
                            writeln!(out, "    {:04X}: {hex}", idx * 16).unwrap();
                        }
                    }
                    Err(e) => {
                        writeln!(out, "  No se pudo consultar el descriptor en Windows: {e}").unwrap();
                    }
                }
            }
        }
        Err(e) => { writeln!(out, "Error de enumeración: {e}").unwrap(); }
    }
    writeln!(out, "\nLos ID 07/21 de la investigación Linux no prueban soporte HID de Windows.").unwrap();
    writeln!(out, "No se enviaron comandos a ninguna interfaz.").unwrap();
    println!("{out}");

    let directory = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."))
        .join("ClonControl");
    if fs::create_dir_all(&directory).is_ok() {
        let path = directory.join("diagnostico.txt");
        if fs::write(&path, out).is_ok() {
            println!("Informe guardado en: {}", path.display());
            println!("No contiene rutas USB ni números de serie.");
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

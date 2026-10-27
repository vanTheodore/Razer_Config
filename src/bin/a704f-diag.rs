#[cfg(windows)]
fn main() {
    use cloncontrol_a704f::{device, protocol};
    println!("CLON//CONTROL | Diagnostico HID en solo lectura");
    println!("VID:PID {:04X}:{:04X}", protocol::VID, protocol::PID);
    match device::scan() {
        Ok(devices) if devices.is_empty() => println!("No se encontraron interfaces HID compatibles."),
        Ok(devices) => {
            for (i, d) in devices.iter().enumerate() {
                println!(
                    "\n#{i}: {}\n Fabricante: {}\n Interfaz: {}\n Usage: {:04X}:{:04X}\n Elegible: {}\n Ruta: {}",
                    d.product, d.manufacturer, d.interface_number, d.usage_page, d.usage, d.eligible, d.path
                );
            }
        }
        Err(e) => eprintln!("Error: {e}"),
    }
    println!("Paquete RGB apagado (solo ejemplo): {}", protocol::hex(&protocol::rgb(protocol::Light::Off)));
    println!("Sin escrituras USB.");
}

#[cfg(not(windows))]
fn main() {
    println!("Diagnostico disponible para Windows.");
}

//! Windows HID communications for VID:PID 18F8:1286 only.
//! No firmware flashing, raw USB transport, or driver installation.

use crate::protocol::{hex, Report, Transport, PID, VID};
use hidapi::HidApi;
use std::{ffi::CString, thread, time::Duration};

#[derive(Debug, Clone)]
pub struct Candidate {
    pub path: String,
    pub product: String,
    pub manufacturer: String,
    pub interface_number: i32,
    pub usage_page: u16,
    pub usage: u16,
    pub eligible: bool,
}
impl Candidate {
    pub fn description(&self) -> String {
        format!(
            "{} [MI {} | Usage {:04X}:{:04X}]{}",
            self.product, self.interface_number, self.usage_page, self.usage,
            if self.eligible { "" } else { " (no escribible)" }
        )
    }
}

fn is_mouse_interface(path: &str, interface: i32, page: u16, usage: u16) -> bool {
    // Never send reports to keyboard HID collections.
    if page == 1 && usage == 6 { return false; }
    (page == 1 && usage == 2)
        || (interface == 0 && path.to_ascii_lowercase().contains("mi_00"))
}

pub fn scan() -> Result<Vec<Candidate>, String> {
    let api = HidApi::new().map_err(|e| format!("No se pudo iniciar HIDAPI: {e}"))?;
    let mut found: Vec<_> = api.device_list()
        .filter(|d| d.vendor_id() == VID && d.product_id() == PID)
        .map(|d| {
            let path = d.path().to_string_lossy().into_owned();
            Candidate {
                eligible: is_mouse_interface(&path, d.interface_number(), d.usage_page(), d.usage()),
                path,
                product: d.product_string().unwrap_or("Mouse 18F8:1286").to_owned(),
                manufacturer: d.manufacturer_string().unwrap_or("No informado").to_owned(),
                interface_number: d.interface_number(),
                usage_page: d.usage_page(),
                usage: d.usage(),
            }
        }).collect();
    found.sort_by_key(|d| (!d.eligible, d.path.clone()));
    Ok(found)
}

/// Re-enumerate and verify the path before every write session.
/// HIDAPI send_feature_report returns Result<(), HidError>, NOT a byte count.
pub fn apply(path: &str, reports: &[Report], transport: Transport) -> Result<String, String> {
    if reports.is_empty() { return Err("No hay configuraciones seleccionadas".into()); }
    // Encode all reports before opening the device; invalid modes never partially write.
    let frames: Vec<Vec<u8>> = reports.iter().map(|r| transport.encode(r)).collect::<Result<_, _>>()?;
    let api = HidApi::new().map_err(|e| format!("Error HIDAPI: {e}"))?;
    let Some(info) = api.device_list().find(|d| {
        d.vendor_id() == VID && d.product_id() == PID && d.path().to_bytes() == path.as_bytes()
    }) else { return Err("Mouse desconectado o ruta cambiada; vuelve a buscar".into()); };
    if !is_mouse_interface(path, info.interface_number(), info.usage_page(), info.usage()) {
        return Err("La coleccion seleccionada no es elegible para escribir".into());
    }
    let cpath = CString::new(path).map_err(|_| "Ruta USB invalida".to_owned())?;
    let dev = api.open_path(&cpath).map_err(|e| format!("Windows no permite abrir esta interfaz: {e}"))?;
    for (idx, (report, frame)) in reports.iter().zip(frames.iter()).enumerate() {
        dev.send_feature_report(frame).map_err(|e| {
            format!("Reporte {}/{} ({}) fallido: {e}; los anteriores podrian haberse aplicado",
                idx + 1, reports.len(), hex(report))
        })?;
        thread::sleep(Duration::from_millis(25));
    }
    Ok(format!(
        "{} reportes aceptados por HIDAPI. No sabemos si el mouse cambió; compruébalo físicamente.",
        reports.len()
    ))
}

/// Read-only USB HID report-descriptor inquiry; no HID SetFeature is issued.
pub fn read_report_descriptor(path: &str) -> Result<Vec<u8>, String> {
    let api = HidApi::new().map_err(|e| format!("HIDAPI: {e}"))?;
    if !api.device_list().any(|d|
        d.vendor_id() == VID && d.product_id() == PID && d.path().to_bytes() == path.as_bytes()
    ) {
        return Err("El dispositivo seleccionado ya no está disponible".into());
    }
    let cpath = CString::new(path).map_err(|_| "Ruta USB inválida".to_owned())?;
    let dev = api.open_path(&cpath).map_err(|e| format!("No se pudo abrir para diagnóstico: {e}"))?;
    let mut buf = vec![0u8; 4096];
    let len = dev.get_report_descriptor(&mut buf).map_err(|e| format!("No se pudo leer descriptor HID: {e}"))?;
    buf.truncate(len);
    Ok(buf)
}

//! Windows HID transport for vendor configuration collection FF01:0001 on MI 1.
//! No firmware flashing, driver replacement or access to normal mouse reports.

use crate::protocol::{feature_07_read_buffer, hex, is_verified_config_collection, Report, PID, VID};
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
    /// True ONLY if the report descriptor declares Feature ID 07 with 56 bits
    /// in the exact vendor configuration collection FF01:0001, MI 1.
    pub eligible: bool,
}
impl Candidate {
    pub fn description(&self) -> String {
        format!(
            "{} [MI {} | Usage {:04X}:{:04X}]{}",
            self.product, self.interface_number, self.usage_page, self.usage,
            if self.eligible { " — configuración HID 07" } else { " (solo diagnóstico)" }
        )
    }
}

fn matches_identity(interface: i32, page: u16, usage: u16) -> bool {
    interface == 1 && page == 0xFF01 && usage == 1
}

fn read_descriptor(dev: &hidapi::HidDevice) -> Result<Vec<u8>, String> {
    let mut buf = vec![0u8; 4096];
    let len = dev.get_report_descriptor(&mut buf)
        .map_err(|e| format!("No se pudo leer el descriptor HID: {e}"))?;
    buf.truncate(len);
    Ok(buf)
}

pub fn scan() -> Result<Vec<Candidate>, String> {
    let api = HidApi::new().map_err(|e| format!("No se pudo iniciar HIDAPI: {e}"))?;
    let mut found: Vec<_> = api.device_list()
        .filter(|d| d.vendor_id() == VID && d.product_id() == PID)
        .map(|d| {
            let path = d.path().to_string_lossy().into_owned();
            let eligible = if matches_identity(d.interface_number(), d.usage_page(), d.usage()) {
                api.open_path(d.path()).ok()
                    .and_then(|dev| read_descriptor(&dev).ok())
                    .is_some_and(|desc| is_verified_config_collection(
                        d.interface_number(), d.usage_page(), d.usage(), &desc
                    ))
            } else { false };
            Candidate {
                eligible,
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

/// Always reopen the exact selected HID collection and revalidate its report
/// descriptor before sending any report. No other collection can be written.
pub fn apply(path: &str, reports: &[Report], delay_ms: u16) -> Result<String, String> {
    if reports.is_empty() { return Err("No hay configuraciones seleccionadas".into()); }
    if reports.iter().any(|r| r.len() != 8 || r[0] != 0x07) {
        return Err("Solo se permiten reportes HID 07 de ocho bytes".into());
    }
    let api = HidApi::new().map_err(|e| format!("Error HIDAPI: {e}"))?;
    let Some(info) = api.device_list().find(|d| {
        d.vendor_id() == VID && d.product_id() == PID && d.path().to_bytes() == path.as_bytes()
    }) else { return Err("Mouse desconectado o ruta cambiada; vuelve a buscar".into()); };

    if !matches_identity(info.interface_number(), info.usage_page(), info.usage()) {
        return Err("Solo puede escribirse a MI 1 / FF01:0001, no al mouse ni al teclado".into());
    }
    let cpath = CString::new(path).map_err(|_| "Ruta HID inválida".to_owned())?;
    let dev = api.open_path(&cpath).map_err(|e| format!("Windows no permite abrir esta colección: {e}"))?;
    let descriptor = read_descriptor(&dev)?;
    if !is_verified_config_collection(
        info.interface_number(), info.usage_page(), info.usage(), &descriptor,
    ) {
        return Err("El descriptor ya no declara Feature 07 de 7 bytes; envío bloqueado".into());
    }

    for (idx, report) in reports.iter().enumerate() {
        dev.send_feature_report(report).map_err(|e| {
            format!("Reporte {}/{} ({}) fallido: {e}; los anteriores podrían haberse aplicado",
                idx + 1, reports.len(), hex(report))
        })?;
        if idx + 1 < reports.len() {
            thread::sleep(Duration::from_millis(u64::from(delay_ms)));
        }
    }
    Ok(format!(
        "{} reportes aceptados en MI 1 / FF01:0001 (Feature 07). Comprueba físicamente el resultado.",
        reports.len()
    ))
}

/// Optional, read-only Feature 07 probe with an 8- or 32-byte buffer.
/// HIDAPI's windows-native backend uses DeviceIoControl for GetFeature.
/// This is NOT called at startup or during normal scan, and never writes.
pub fn probe_feature_07(path: &str, length: usize) -> Result<Vec<u8>, String> {
    let mut buf = feature_07_read_buffer(length)?;
    let api = HidApi::new().map_err(|e| format!("HIDAPI: {e}"))?;
    let info = api.device_list().find(|dev|
        dev.vendor_id() == VID && dev.product_id() == PID
        && dev.path().to_bytes() == path.as_bytes()
    ).ok_or("No existe el HID seleccionado: vuelve a conectar el mouse")?;
    if !matches_identity(info.interface_number(), info.usage_page(), info.usage()) {
        return Err("La lectura opcional está limitada a MI 1 / FF01:0001".into());
    }
    let cpath = CString::new(path).map_err(|_| "Ruta HID inválida".to_owned())?;
    let dev = api.open_path(&cpath)
        .map_err(|e| format!("Windows no permite abrir la colección: {e}"))?;
    let descriptor = read_descriptor(&dev)?;
    if !is_verified_config_collection(
        info.interface_number(), info.usage_page(), info.usage(), &descriptor,
    ) {
        return Err("Descriptor inesperado: no se hace lectura de Report 07".into());
    }
    let count = dev.get_feature_report(&mut buf)
        .map_err(|e| format!("GetFeature 07 ({length} bytes) falló: {e}"))?;
    if count > buf.len() { return Err("HIDAPI informó una longitud imposible".into()); }
    Ok(buf[..count].to_vec())
}

/// Diagnostic only; opened without HID SetFeature and never used to send.
pub fn read_report_descriptor(path: &str) -> Result<Vec<u8>, String> {
    let api = HidApi::new().map_err(|e| format!("HIDAPI: {e}"))?;
    if !api.device_list().any(|d|
        d.vendor_id() == VID && d.product_id() == PID && d.path().to_bytes() == path.as_bytes()
    ) {
        return Err("El dispositivo seleccionado ya no está disponible".into());
    }
    let cpath = CString::new(path).map_err(|_| "Ruta USB inválida".to_owned())?;
    let dev = api.open_path(&cpath).map_err(|e| format!("No se pudo abrir para diagnóstico: {e}"))?;
    read_descriptor(&dev)
}

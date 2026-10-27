//! A704F reports from AmberIsFrozen/Instant-A704F-Mouse-Utilities/SPECS.md.
//! Each report consists of report ID 07 and seven payload bytes.
//! No firmware or arbitrary commands are implemented.

use serde::{Deserialize, Serialize};

pub const VID: u16 = 0x18F8;
pub const PID: u16 = 0x1286;
pub type Report = [u8; 8];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Light { Slow, Medium, Fast, Faster, VeryFast, Static, Off }
impl Light {
    pub const ALL: [Self; 7] = [
        Self::Slow, Self::Medium, Self::Fast, Self::Faster, Self::VeryFast, Self::Static, Self::Off
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Slow => "Respiración lenta (6s)",
            Self::Medium => "Respiración (5s)",
            Self::Fast => "Respiración (3s)",
            Self::Faster => "Respiración (2s)",
            Self::VeryFast => "Respiración rapida (1s)",
            Self::Static => "Fija",
            Self::Off => "Apagada",
        }
    }
    pub fn code(self) -> u8 {
        match self {
            Self::Slow => 0x10,
            Self::Medium => 0x11,
            Self::Fast => 0x13,
            Self::Faster => 0x14,
            Self::VeryFast => 0x15,
            Self::Static => 0x16,
            Self::Off => 0x17,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Wheel { Scroll, Volume }
impl Wheel {
    pub const ALL: [Self; 2] = [Self::Scroll, Self::Volume];
    pub fn label(self) -> &'static str {
        match self { Self::Scroll => "Desplazamiento", Self::Volume => "Volumen" }
    }
    pub fn code(self) -> u8 { match self { Self::Scroll => 0, Self::Volume => 1 } }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action { Back, Forward, DpiCycle, DpiPlus, DpiMinus, MiddleClick, DoubleClick }
impl Action {
    pub const ALL: [Self; 7] = [
        Self::Back, Self::Forward, Self::DpiCycle, Self::DpiPlus, Self::DpiMinus, Self::MiddleClick, Self::DoubleClick
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Back => "Atrás",
            Self::Forward => "Adelante",
            Self::DpiCycle => "Cambiar perfil DPI",
            Self::DpiPlus => "DPI +",
            Self::DpiMinus => "DPI -",
            Self::MiddleClick => "Clic central",
            Self::DoubleClick => "Doble clic",
        }
    }
    pub fn code(self) -> u8 {
        match self {
            Self::Back => 0x04,
            Self::Forward => 0x05,
            Self::DpiCycle => 0x06,
            Self::DpiPlus => 0x0A,
            Self::DpiMinus => 0x0B,
            Self::MiddleClick => 0x02,
            Self::DoubleClick => 0x08,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub profiles: [u8; 4],
    pub enabled: [bool; 4],
    pub active: usize,
    pub light: Light,
    pub wheel: Wheel,
    pub front: Action,
    pub rear: Action,
    pub apply_dpi: bool,
    pub apply_light: bool,
    pub apply_wheel: bool,
    pub apply_buttons: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            profiles: [3, 4, 6, 12], enabled: [true; 4], active: 0,
            light: Light::Slow, wheel: Wheel::Scroll,
            front: Action::Forward, rear: Action::Back,
            apply_dpi: false, apply_light: false, apply_wheel: false, apply_buttons: false,
        }
    }
}

impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        if self.active > 3 { return Err("Perfil activo fuera del rango".into()); }
        if self.apply_dpi {
            if self.profiles.iter().any(|&x| !(1..=15).contains(&x)) {
                return Err("DPI: solo 200..3000 CPI en pasos de 200".into());
            }
            if !self.enabled.iter().any(|&on| on) || !self.enabled[self.active] {
                return Err("Activa al menos un perfil, incluido el seleccionado".into());
            }
        }
        Ok(())
    }
    pub fn reports(&self) -> Result<Vec<Report>, String> {
        self.validate()?;
        let mut output = Vec::new();
        if self.apply_light { output.push(rgb(self.light)); }
        if self.apply_wheel { output.push(wheel(self.wheel)); }
        if self.apply_buttons {
            output.push(bind(0x05, self.front)?);
            output.push(bind(0x04, self.rear)?);
        }
        if self.apply_dpi {
            let mask = self.enabled.iter().enumerate().fold(0u8, |mask, (i, on)| {
                mask | if *on { 1 << i } else { 0 }
            });
            for (slot, &value) in self.profiles.iter().enumerate() {
                output.push(dpi(self.active as u8, slot as u8, value, mask)?);
            }
        }
        Ok(output)
    }
}

fn packet(kind: u8, x: u8, y: u8, z: u8) -> Report {
    [0x07, kind, x, y, z, 0, 0, 0]
}
pub fn rgb(light: Light) -> Report { packet(0x13, 0x7F, light.code(), 0) }
pub fn wheel(mode: Wheel) -> Report { packet(0x11, mode.code(), 0, 0) }
pub fn bind(physical: u8, action: Action) -> Result<Report, String> {
    if !matches!(physical, 0x04 | 0x05) {
        return Err("Solo se permite reasignar botones laterales".into());
    }
    Ok(packet(0x10, physical, action.code(), 0))
}
pub fn dpi(active: u8, slot: u8, unit: u8, mask: u8) -> Result<Report, String> {
    if active > 3 || slot > 3 || !(1..=15).contains(&unit) || mask == 0 || mask > 0x0F || mask & (1 << active) == 0 {
        return Err("Parámetros DPI inválidos".into());
    }
    Ok(packet(0x09, active, (unit << 4) | (0x08 + slot), mask))
}
/// Different, unverified HIDAPI payloads seen in the Linux reference and the USB capture.
/// The 16-byte form has NOT been confirmed to work via HidD_SetFeature on Windows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transport {
    Standard8,
    LinuxReference16,
}

impl Transport {
    pub fn label(self) -> &'static str {
        match self {
            Self::Standard8 => "Reporte de 8 bytes (inicial, sin verificar)",
            Self::LinuxReference16 => "Trama de referencia de 16 bytes (experimental, solo luz)",
        }
    }
    pub fn encode(self, report: &Report) -> Result<Vec<u8>, String> {
        match self {
            Self::Standard8 => Ok(report.to_vec()),
            Self::LinuxReference16 => {
                if report[1] != 0x13 {
                    return Err("La trama experimental de 16 bytes solo está habilitada para iluminación".into());
                }
                let mut frame = vec![0x21, 0x09, 0x07, 0x03, 0x01, 0x00, 0x08, 0x00];
                frame.extend_from_slice(report);
                Ok(frame)
            }
        }
    }
}

/// Parse the Report ID of HID Feature items from a descriptor without sending anything.
pub fn feature_report_ids(descriptor: &[u8]) -> Vec<u8> {
    let mut current_id = 0u8;
    let mut result = Vec::new();
    let mut i = 0;
    while i < descriptor.len() {
        let prefix = descriptor[i];
        i += 1;
        if prefix == 0xFE {
            if i + 2 > descriptor.len() { break; }
            let size = descriptor[i] as usize;
            i += 2;
            if i + size > descriptor.len() { break; }
            i += size;
            continue;
        }
        let size = match prefix & 3 { 0 => 0, 1 => 1, 2 => 2, _ => 4 };
        if i + size > descriptor.len() { break; }
        let kind = (prefix >> 2) & 3;
        let tag = prefix >> 4;
        if kind == 1 && tag == 8 && size == 1 { current_id = descriptor[i]; }
        if kind == 0 && tag == 11 && !result.contains(&current_id) {
            result.push(current_id);
        }
        i += size;
    }
    result.sort_unstable();
    result
}

pub fn hex(report: &Report) -> String {
    report.iter().map(|b| format!("{b:02X}")).collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn framing_matches_documented_linux_reference() {
        let rgb = rgb(Light::Off);
        assert_eq!(Transport::Standard8.encode(&rgb).unwrap(), vec![7, 0x13, 0x7f, 0x17, 0, 0, 0, 0]);
        assert_eq!(Transport::LinuxReference16.encode(&rgb).unwrap(),
            vec![0x21, 0x09, 0x07, 0x03, 0x01, 0, 0x08, 0, 0x07, 0x13, 0x7f, 0x17, 0, 0, 0, 0]);
        assert!(Transport::LinuxReference16.encode(&wheel(Wheel::Volume)).is_err());
        assert!(Transport::LinuxReference16.encode(&dpi(0, 0, 4, 15).unwrap()).is_err());
    }
    #[test]
    fn descriptor_feature_ids() {
        let desc = [0x05, 0x01, 0x85, 0x07, 0x09, 0x02, 0xB1, 0x02,
                    0x85, 0x21, 0xB1, 0x02, 0x85, 0x07, 0xB1, 0x02];
        assert_eq!(feature_report_ids(&desc), vec![7, 0x21]);
        assert_eq!(feature_report_ids(&[0x05, 0x01, 0x09, 0x02]), Vec::<u8>::new());
        assert_eq!(feature_report_ids(&[0x85]), Vec::<u8>::new());
    }
    #[test]
    fn rgb_off_vector() {
        assert_eq!(rgb(Light::Off), [7, 0x13, 0x7F, 0x17, 0, 0, 0, 0]);
    }
    #[test]
    fn dpi_vectors() {
        assert_eq!(dpi(0, 0, 3, 15).unwrap(), [7, 9, 0, 0x38, 15, 0, 0, 0]);
        assert_eq!(dpi(1, 2, 12, 15).unwrap(), [7, 9, 1, 0xCA, 15, 0, 0, 0]);
    }
    #[test]
    fn action_vectors_and_safety() {
        assert_eq!(bind(5, Action::Forward).unwrap(), [7, 0x10, 5, 5, 0, 0, 0, 0]);
        assert!(bind(1, Action::Forward).is_err());
        assert_eq!(wheel(Wheel::Volume), [7, 0x11, 1, 0, 0, 0, 0, 0]);
    }
    #[test]
    fn default_never_writes() {
        assert!(Settings::default().reports().unwrap().is_empty());
    }
    #[test]
    fn invalid_dpi_fails_closed() {
        let mut s = Settings::default();
        s.apply_dpi = true;
        s.enabled = [false; 4];
        assert!(s.reports().is_err());
        s.enabled = [true, false, false, false];
        s.active = 1;
        assert!(s.reports().is_err());
        s.active = 0;
        s.profiles[0] = 16;
        assert!(s.reports().is_err());
    }
}

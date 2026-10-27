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
            Self::VeryFast => "Respiración rápida (1s)",
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
    pub report_delay_ms: u16,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            profiles: [3, 4, 6, 12], enabled: [true; 4], active: 0,
            light: Light::Slow, wheel: Wheel::Scroll,
            front: Action::Forward, rear: Action::Back,
            apply_dpi: false, apply_light: false, apply_wheel: false, apply_buttons: false,
            report_delay_ms: 50,
        }
    }
}

impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        if self.report_delay_ms > 1000 { return Err("Pausa entre reportes: máximo 1000 ms".into()); }
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

    pub fn reports_with_experimental_followup(&self, enabled: bool) -> Result<Vec<Report>, String> {
        let mut reports = self.reports()?;
        if enabled && !reports.is_empty() {
            reports.push(experimental_followup());
        }
        Ok(reports)
    }
}

fn packet(kind: u8, x: u8, y: u8, z: u8) -> Report {
    [0x07, kind, x, y, z, 0, 0, 0]
}
/// Hypothesis from the Redgear A20 driver; unconfirmed for 18F8:1286.
pub fn experimental_followup() -> Report { packet(0x20, 0, 0, 0) }
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
/// Types of reports declared by one HID top-level collection.
/// A 0 report ID means the descriptor uses unnumbered reports.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReportInventory {
    pub input: Vec<u8>,
    pub output: Vec<u8>,
    pub feature: Vec<u8>,
}

/// Parse report declarations without interacting with USB. Track the HID
/// Global Report ID across Push/Pop items (needed for some composite devices).
/// This indicates what the descriptor *declares*, not what hardware accepts.
pub fn report_inventory(descriptor: &[u8]) -> ReportInventory {
    let mut result = ReportInventory::default();
    let mut current_id = 0u8;
    let mut id_stack = Vec::new();
    let mut i = 0;
    while i < descriptor.len() {
        let prefix = descriptor[i];
        i += 1;
        if prefix == 0xFE {
            // Long item: size, tag, data.
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
        if kind == 1 {
            match (tag, size) {
                (8, 1) => current_id = descriptor[i],
                (10, _) => id_stack.push(current_id), // PUSH
                (11, _) => { if let Some(id) = id_stack.pop() { current_id = id; } }, // POP
                _ => {}
            }
        }
        if kind == 0 {
            let ids = match tag {
                8 => Some(&mut result.input),
                9 => Some(&mut result.output),
                11 => Some(&mut result.feature),
                _ => None,
            };
            if let Some(ids) = ids {
                if !ids.contains(&current_id) { ids.push(current_id); }
            }
        }
        i += size;
    }
    result.input.sort_unstable();
    result.output.sort_unstable();
    result.feature.sort_unstable();
    result
}

/// Count payload bits for the given numbered HID Feature Report.
/// This returns None when the report is missing or the descriptor is malformed.
/// It is a structural check, not proof that the firmware will honor writes.
pub fn feature_payload_bits(descriptor: &[u8], wanted_id: u8) -> Option<u32> {
    let mut globals = (0u32, 0u32, 0u8); // report size, count, ID
    let mut stack = Vec::new();
    let mut bits = 0u32;
    let mut found = false;
    let mut i = 0usize;
    while i < descriptor.len() {
        let prefix = descriptor[i];
        i += 1;
        if prefix == 0xFE {
            if i + 2 > descriptor.len() { return None; }
            let size = descriptor[i] as usize;
            i += 2;
            if i + size > descriptor.len() { return None; }
            i += size;
            continue;
        }
        let size = match prefix & 3 { 0 => 0, 1 => 1, 2 => 2, _ => 4 };
        if i + size > descriptor.len() { return None; }
        let kind = (prefix >> 2) & 3;
        let tag = prefix >> 4;
        let mut value = 0u32;
        for (shift, byte) in descriptor[i..i + size].iter().enumerate() {
            value |= (*byte as u32) << (shift * 8);
        }
        if kind == 1 {
            match tag {
                7 => globals.0 = value, // Report Size
                8 if size == 1 => globals.2 = value as u8, // Report ID
                9 => globals.1 = value, // Report Count
                10 => stack.push(globals), // Push
                11 => { globals = stack.pop()?; }, // Pop
                _ => {}
            }
        } else if kind == 0 && tag == 11 && globals.2 == wanted_id {
            let chunk = globals.0.checked_mul(globals.1)?;
            bits = bits.checked_add(chunk)?;
            found = true;
        }
        i += size;
    }
    if found { Some(bits) } else { None }
}

/// Exactly the vendor-defined collection observed on the user's 18F8:1286.
/// The descriptor announces report 07 with seven bytes of payload.
pub fn is_verified_config_collection(
    interface: i32,
    usage_page: u16,
    usage: u16,
    descriptor: &[u8],
) -> bool {
    interface == 1 && usage_page == 0xFF01 && usage == 0x0001
        && feature_payload_bits(descriptor, 7) == Some(56)
}

pub fn feature_report_ids(descriptor: &[u8]) -> Vec<u8> {
    report_inventory(descriptor).feature
}

/// Reconstruct the sole decision expression visible after the vendor's
/// GetFeature(07, 32 bytes). A true value is NOT permission to send follow-up
/// commands: the purpose and side effects of those commands remain unknown.
pub fn instant_probe_condition(bytes: &[u8]) -> Option<bool> {
    if bytes.len() < 4 || bytes[0] != 7 { return None; }
    Some((bytes[3] & 0xFE) >= 0x08)
}

pub fn hex(report: &Report) -> String {
    report.iter().map(|b| format!("{b:02X}")).collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    const REAL_CONFIG_DESCRIPTOR: [u8; 23] = [
        0x06, 0x01, 0xFF, 0x09, 0x01, 0xA1, 0x01, 0x85,
        0x07, 0x09, 0x20, 0x15, 0x00, 0x26, 0xFF, 0x00,
        0x75, 0x08, 0x95, 0x07, 0xB1, 0x02, 0xC0,
    ];
    #[test]
    fn real_device_descriptor_matches_8_byte_feature_report() {
        assert_eq!(feature_payload_bits(&REAL_CONFIG_DESCRIPTOR, 7), Some(56));
        assert!(is_verified_config_collection(1, 0xFF01, 1, &REAL_CONFIG_DESCRIPTOR));
        assert!(!is_verified_config_collection(0, 0xFF01, 1, &REAL_CONFIG_DESCRIPTOR));
        assert!(!is_verified_config_collection(1, 0xFF00, 1, &REAL_CONFIG_DESCRIPTOR));
        assert!(!is_verified_config_collection(1, 0xFF01, 2, &REAL_CONFIG_DESCRIPTOR));
        assert!(!is_verified_config_collection(1, 0xFF01, 1, &[0x85, 0x07, 0x75, 0x08, 0x95, 0x08, 0xB1, 0x02]));
        assert_eq!(feature_payload_bits(&REAL_CONFIG_DESCRIPTOR, 0x21), None);
    }
    #[test]
    fn feature_payload_bits_handles_stack_and_malformed_data() {
        let with_stack = [
            0x85, 0x07, 0x75, 0x08, 0x95, 0x02, 0xB1, 0x02,
            0xA4, 0x85, 0x08, 0xB1, 0x02, 0xB4,
            0x95, 0x05, 0xB1, 0x02,
        ];
        assert_eq!(feature_payload_bits(&with_stack, 7), Some(56));
        assert_eq!(feature_payload_bits(&with_stack, 8), Some(16));
        assert_eq!(feature_payload_bits(&[0x85, 0x07, 0xB4], 7), None);
        assert_eq!(feature_payload_bits(&[0x85], 7), None);
    }
    #[test]
    fn report_inventory_recognizes_input_output_feature_and_unnumbered() {
        let desc = [
            0x05, 0x01, 0x09, 0x02,
            0x81, 0x02, // input, implicit report 0
            0x91, 0x02, // output, implicit report 0
            0x85, 0x07, 0xB1, 0x02, // feature 7
            0xA4, // push id 7
            0x85, 0x21, 0xB1, 0x02, // feature 33
            0xB4, 0x81, 0x02, // pop -> input 7
        ];
        let inv = report_inventory(&desc);
        assert_eq!(inv.input, vec![0, 7]);
        assert_eq!(inv.output, vec![0]);
        assert_eq!(inv.feature, vec![7, 33]);
        assert_eq!(report_inventory(&[0x85]).feature, Vec::<u8>::new());
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
    fn instant_getfeature_condition_is_read_only_and_checks_report_id() {
        assert_eq!(instant_probe_condition(&[7, 0, 0, 0x09]), Some(true));
        assert_eq!(instant_probe_condition(&[7, 0, 0, 0x08]), Some(true));
        assert_eq!(instant_probe_condition(&[7, 0, 0, 0x07]), Some(false));
        assert_eq!(instant_probe_condition(&[0, 0, 0, 0x09]), None);
        assert_eq!(instant_probe_condition(&[7, 0, 0]), None);
    }
    #[test]
    fn rgb_off_vector() {
        assert_eq!(rgb(Light::Off), [7, 0x13, 0x7F, 0x17, 0, 0, 0, 0]);
    }
    #[test]
    fn experimental_followup_is_opt_in_and_only_after_a_report() {
        assert_eq!(experimental_followup(), [7, 0x20, 0, 0, 0, 0, 0, 0]);
        let mut settings = Settings::default();
        assert!(settings.reports_with_experimental_followup(true).unwrap().is_empty());
        settings.apply_light = true;
        settings.light = Light::Off;
        assert_eq!(settings.reports_with_experimental_followup(false).unwrap(),
            vec![[7, 0x13, 0x7F, 0x17, 0, 0, 0, 0]]);
        assert_eq!(settings.reports_with_experimental_followup(true).unwrap(),
            vec![[7, 0x13, 0x7F, 0x17, 0, 0, 0, 0], [7, 0x20, 0, 0, 0, 0, 0, 0]]);
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
    fn report_delay_defaults_and_range() {
        assert_eq!(Settings::default().report_delay_ms, 50);
        let mut old_preset: Settings = serde_json::from_str("{}").unwrap();
        assert_eq!(old_preset.report_delay_ms, 50);
        old_preset.report_delay_ms = 1001;
        assert!(old_preset.validate().is_err());
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

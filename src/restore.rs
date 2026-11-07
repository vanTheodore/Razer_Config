//! Explicit, non-resident restoration of saved DPI at Windows logon.
//! This never restores experimental lighting, button, or wheel settings.

use crate::{device, protocol::Settings};
use std::{
    fs,
    path::Path,
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

fn settings_from_json(json: &str) -> Result<Settings, String> {
    let value: serde_json::Value =
        serde_json::from_str(json).map_err(|e| format!("Preset JSON inválido: {e}"))?;
    // Serde defaults are useful for upgrades, but headless writes must only use
    // an explicitly saved DPI preset, never the application's defaults.
    for field in ["profiles", "enabled", "active"] {
        if value.get(field).is_none() {
            return Err(format!("El preset no contiene '{field}'; guarda los perfiles desde la interfaz"));
        }
    }
    let settings: Settings =
        serde_json::from_value(value).map_err(|e| format!("Preset incompatible: {e}"))?;
    settings.dpi_reports()?;
    Ok(settings)
}

fn restore_from(dir: &Path) -> Result<String, String> {
    let json = fs::read_to_string(dir.join("a704f.json"))
        .map_err(|e| format!("No se pudo leer el preset guardado a704f.json: {e}"))?;
    let settings = settings_from_json(&json)?;
    let reports = settings.dpi_reports()?;
    debug_assert_eq!(reports.len(), 4);
    debug_assert!(reports.iter().all(|report| report[0] == 0x07 && report[1] == 0x09));

    // A mouse may appear slightly after Windows starts the user's startup apps.
    // This is bounded: no service, persistent background loop, or device path
    // cached across reboots. Never guess which mouse to write when ambiguous.
    for attempt in 0..4 {
        let candidates: Vec<_> = device::scan()?.into_iter().filter(|d| d.eligible).collect();
        match candidates.as_slice() {
            [mouse] => return device::apply(&mouse.path, &reports),
            [] if attempt < 3 => thread::sleep(Duration::from_millis(750)),
            [] => return Err("No se encontró el mouse compatible tras cuatro búsquedas".into()),
            _ => return Err("Hay varios canales de configuración compatibles; no se aplicó nada".into()),
        }
    }
    unreachable!("El bucle siempre devuelve un resultado")
}

/// One-shot operation; call only on explicit `--restore-dpi` invocation.
/// The status file records Windows' response, not verified physical DPI.
/// It never stores serial numbers, HID paths, or any extra user information.
pub fn restore_saved_dpi() -> Result<String, String> {
    let local_app_data = std::env::var_os("LOCALAPPDATA")
        .ok_or("LOCALAPPDATA no está definido; no se restauró el DPI")?;
    let dir = std::path::PathBuf::from(local_app_data).join("ClonControl");
    let result = restore_from(&dir);
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|t| t.as_secs())
        .unwrap_or_default();
    let details = match &result {
        Ok(message) => format!("ACEPTADO POR WINDOWS (no verificado físicamente)\n{message}"),
        Err(message) => format!("ERROR; NO SE CONFIRMA EL DPI\n{message}"),
    };
    // Logging problems must not cause an automatic second write to the mouse.
    let _ = fs::create_dir_all(&dir);
    let _ = fs::write(
        dir.join("ultima_restauracion_dpi.txt"),
        format!(
            "CLON//CONTROL — restauración DPI al inicio\nHora Unix UTC: {timestamp}\nVID:PID 18F8:1286\nSolo comandos DPI 07 09\n{details}\n"
        ),
    );
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_saved_profile_never_uses_defaults() {
        assert!(settings_from_json("{}").is_err());
        assert!(settings_from_json(r#"{"profiles":[3,4,6,12],"active":0}"#).is_err());
        assert!(settings_from_json("no es JSON").is_err());
    }

    #[test]
    fn only_dpi_reports_despite_experimental_flags() {
        let settings = Settings {
            apply_light: true,
            apply_buttons: true,
            apply_wheel: true,
            ..Settings::default()
        };
        let json = serde_json::to_string(&settings).unwrap();
        let restored = settings_from_json(&json).unwrap();
        let reports = restored.dpi_reports().unwrap();
        assert_eq!(reports.len(), 4);
        assert!(reports.iter().all(|report| report[0] == 7 && report[1] == 9));
    }

    #[test]
    fn invalid_dpi_settings_fail_closed() {
        let mut settings = Settings::default();
        settings.active = 7;
        assert!(settings_from_json(&serde_json::to_string(&settings).unwrap()).is_err());
        settings.active = 0;
        settings.enabled = [false; 4];
        assert!(settings_from_json(&serde_json::to_string(&settings).unwrap()).is_err());
    }
}

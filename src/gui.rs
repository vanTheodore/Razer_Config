use cloncontrol_a704f::{device, protocol};
use eframe::egui::{self, Color32, RichText, Stroke, Vec2};
use protocol::{Action, Light, Settings, Wheel};
use std::{fs, path::PathBuf, time::{SystemTime, UNIX_EPOCH}};

const ACCENT: Color32 = Color32::from_rgb(100, 224, 196);
const SECONDARY: Color32 = Color32::from_rgb(165, 181, 191);
const BACKGROUND: Color32 = Color32::from_rgb(17, 21, 29);
const PANEL: Color32 = Color32::from_rgb(29, 35, 44);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Page {
    Dpi,
    Experimental,
    Diagnostics,
}
impl Page {
    const ALL: [Self; 3] = [Self::Dpi, Self::Experimental, Self::Diagnostics];
    fn title(self) -> &'static str {
        match self {
            Self::Dpi => "Sensibilidad DPI",
            Self::Experimental => "Otras funciones",
            Self::Diagnostics => "Diagnóstico",
        }
    }
}

pub struct Configurator {
    settings: Settings,
    devices: Vec<device::Candidate>,
    selected: usize,
    page: Page,
    armed: bool,
    status: String,
    error: bool,
    last_hid_attempt: Option<String>,
}

fn config_path() -> PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("ClonControl")
        .join("a704f.json")
}
fn trace_path() -> PathBuf {
    config_path().with_file_name("ultima_prueba_hid.txt")
}

impl Configurator {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut theme = egui::Visuals::dark();
        theme.panel_fill = BACKGROUND;
        theme.window_fill = BACKGROUND;
        theme.extreme_bg_color = Color32::from_rgb(12, 15, 20);
        theme.selection.bg_fill = Color32::from_rgb(40, 105, 94);
        theme.widgets.active.bg_fill = Color32::from_rgb(39, 117, 99);
        cc.egui_ctx.set_visuals(theme);
        let (settings, warning) = match fs::read_to_string(config_path()) {
            Ok(json) => match serde_json::from_str::<Settings>(&json) {
                Ok(s) => (s, None),
                Err(e) => (Settings::default(), Some(format!("Preset local inválido: {e}"))),
            },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (Settings::default(), None),
            Err(e) => (Settings::default(), Some(format!("No se puede leer el preset: {e}"))),
        };
        let mut app = Self {
            settings,
            devices: Vec::new(),
            selected: 0,
            page: Page::Dpi,
            armed: false,
            status: warning.unwrap_or_else(|| "DPI listo. Aplica los cambios cuando quieras.".into()),
            error: false,
            last_hid_attempt: fs::read_to_string(trace_path()).ok(),
        };
        app.rescan();
        app
    }

    fn message(&mut self, text: impl Into<String>, error: bool) {
        self.status = text.into();
        self.error = error;
    }

    fn reset_consent(&mut self) {
        self.armed = false;
    }

    fn rescan(&mut self) {
        self.reset_consent();
        match device::scan() {
            Ok(found) => {
                self.devices = found;
                self.selected = self.devices.iter().position(|d| d.eligible).unwrap_or(0);
                self.message(
                    format!("Mouse: {}. {} colecciones USB encontradas.", if self.devices.iter().any(|d| d.eligible) { "listo para DPI" } else { "sin canal compatible" }, self.devices.len()),
                    false,
                );
            }
            Err(e) => {
                self.devices.clear();
                self.message(e, true);
            }
        }
    }

    fn save(&mut self) {
        let path = config_path();
        let result = (|| -> Result<(), String> {
            fs::create_dir_all(path.parent().ok_or("Ruta inválida")?)
                .map_err(|e| e.to_string())?;
            let json = serde_json::to_vec_pretty(&self.settings).map_err(|e| e.to_string())?;
            fs::write(&path, json).map_err(|e| e.to_string())?;
            Ok(())
        })();
        match result {
            Ok(()) => self.message(format!("Preset guardado LOCALMENTE: {}", path.display()), false),
            Err(e) => self.message(format!("No se pudo guardar: {e}"), true),
        }
    }

    /// Record the last REAL write request only. Never include USB paths, serials,
    /// usernames, keyboard events or any other identifying HID attributes.
    fn record_hid_result(&mut self, reports: &[protocol::Report], outcome: &Result<String, String>) -> Result<PathBuf, String> {
        let epoch = SystemTime::now().duration_since(UNIX_EPOCH)
            .map(|t| t.as_secs()).unwrap_or_default();
        let (state, details) = match outcome {
            Ok(text) => ("ACEPTADO POR HIDAPI (sin verificación física)", text.as_str()),
            Err(text) => ("ERROR (revisar detalle)", text.as_str()),
        };
        let commands = reports.iter()
            .enumerate()
            .map(|(i, bytes)| format!("  {:02}: {}", i + 1, protocol::hex(bytes)))
            .collect::<Vec<_>>().join("\n");
        let report = format!(
            "CLON//CONTROL 0.2.4 — último intento REAL de escritura HID\n\
             Hora Unix (UTC): {epoch}\n\
             VID:PID 18F8:1286\n\
             Colección seleccionada: MI 1 / FF01:0001 / Feature 07 (8 bytes)\n\
             Comandos previstos (hex):\n{commands}\n\
             Resultado de Windows: {state}\n\
             Detalle: {details}\n\
             Efecto físico: comprobar manualmente en el dispositivo\n\
             Nota: este registro no contiene rutas USB ni números de serie.\n"
        );
        self.last_hid_attempt = Some(report.clone());
        let path = trace_path();
        fs::create_dir_all(path.parent().ok_or("Ruta de informe inválida")?)
            .map_err(|e| format!("No se pudo crear el directorio de registro: {e}"))?;
        fs::write(&path, report).map_err(|e| format!("No se pudo guardar el registro HID: {e}"))?;
        Ok(path)
    }

    /// The explicit Apply button is the user's authorization for this operation.
    /// Changing a slider or saving the preset never writes to the device.
    fn send_reports(&mut self, reports: Result<Vec<protocol::Report>, String>, label: &str) {
        let reports = match reports {
            Ok(p) if !p.is_empty() => p,
            Ok(_) => {
                self.message("No hay cambios de esa categoría seleccionados.", true);
                return;
            }
            Err(e) => {
                self.message(e, true);
                return;
            }
        };
        let target = self.devices.get(self.selected).filter(|d| d.eligible).map(|d| d.path.clone());
        let result = match target {
            Some(path) => device::apply(&path, &reports),
            None => Err("No hay un canal HID de configuración válido. Pulsa Buscar mouse.".into()),
        };
        let record = self.record_hid_result(&reports, &result);
        self.reset_consent();
        let report_note = match record {
            Ok(_) => String::new(),
            Err(e) => format!(" No se pudo guardar el registro: {e}."),
        };
        match result {
            Ok(msg) => self.message(
                format!("{label}: Windows aceptó la solicitud. Comprueba el efecto en el mouse; no se mide el CPI real. {msg}{report_note}"),
                false
            ),
            Err(e) => self.message(format!("{label}: no se pudo aplicar. {e}.{report_note}"), true),
        }
    }

    fn apply_dpi(&mut self) {
        // Nothing selected under 'Other functions' can leak into a DPI write.
        self.send_reports(self.settings.dpi_reports(), "DPI");
    }

    fn apply_experimental(&mut self) {
        if !self.armed {
            self.message("Confirma que entiendes que estas funciones no están verificadas.", true);
            return;
        }
        self.armed = false;
        // Never include DPI in an experimental operation.
        self.send_reports(self.settings.experimental_reports(), "Función experimental");
    }

    fn header(&mut self, ui: &mut egui::Ui) {
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.heading(RichText::new("CLON//CONTROL").size(22.0).color(ACCENT));
            ui.separator();
            ui.label(RichText::new("A704F  /  WINDOWS").color(SECONDARY));
            ui.separator();
            let connected = self.devices.iter().any(|d| d.eligible);
            ui.colored_label(
                if connected { ACCENT } else { Color32::YELLOW },
                if connected { "Mouse conectado" } else { "Mouse no detectado" },
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Buscar mouse").clicked() { self.rescan(); }
            });
        });
        ui.add_space(8.0);
    }

    fn nav(&mut self, ui: &mut egui::Ui) {
        ui.add_space(20.0);
        ui.heading("TU MOUSE");
        ui.label(RichText::new("INSTANT / A704F").color(SECONDARY).small());
        ui.label(RichText::new("18F8 : 1286").monospace().color(ACCENT));
        ui.add_space(18.0);
        ui.separator();
        ui.add_space(10.0);
        for page in Page::ALL {
            if ui.add_sized(
                [170.0, 44.0],
                egui::Button::new(page.title()).selected(self.page == page),
            ).clicked() {
                self.page = page;
                self.reset_consent();
            }
        }
        ui.add_space(20.0);
        ui.separator();
        ui.label(RichText::new("DPI: probado en este mouse").color(ACCENT).small());
        ui.label(RichText::new("Otros controles: experimentales").color(SECONDARY).small());
        ui.add_space(12.0);
        ui.label(RichText::new("CLON//CONTROL").color(SECONDARY).small());
    }

    fn title(ui: &mut egui::Ui, title: &str, subtitle: &str) {
        ui.heading(RichText::new(title).size(27.0));
        ui.label(RichText::new(subtitle).color(SECONDARY));
        ui.add_space(17.0);
    }

    fn dpi(&mut self, ui: &mut egui::Ui) {
        Self::title(ui, "Tu sensibilidad", "Cambia el DPI sin simulaciones ni pasos innecesarios.");
        let current = self.settings.active.min(3);
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.heading(RichText::new(format!("{} CPI", self.settings.profiles[current] as u16 * 200))
                    .size(31.0).color(ACCENT));
                ui.vertical(|ui| {
                    ui.strong("Perfil seleccionado");
                    ui.label(RichText::new("Valor nominal; no medido con instrumentos").small().color(SECONDARY));
                });
            });
        });
        ui.add_space(12.0);
        ui.label(RichText::new("Elige el perfil activo y su sensibilidad (200 a 3000 CPI).")
            .color(SECONDARY));
        ui.add_space(10.0);
        for i in 0..4 {
            let mut changed = false;
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    if ui.radio_value(&mut self.settings.active, i, format!("Perfil {}", i + 1)).changed() {
                        self.settings.enabled[i] = true;
                        changed = true;
                    }
                    ui.separator();
                    if ui.checkbox(&mut self.settings.enabled[i], "Activo").changed() {
                        if self.settings.active == i && !self.settings.enabled[i] {
                            self.settings.enabled[i] = true; // active profile cannot be disabled
                        } else {
                            changed = true;
                        }
                    }
                    ui.label(RichText::new(format!("{} CPI", self.settings.profiles[i] as u16 * 200))
                        .color(if self.settings.active == i { ACCENT } else { SECONDARY }));
                });
                if ui.add(egui::Slider::new(&mut self.settings.profiles[i], 1..=15)
                    .text("Sensibilidad").suffix(" x200 CPI")).changed() {
                    changed = true;
                }
            });
            if changed {
                self.settings.apply_dpi = true;
                self.reset_consent();
            }
            ui.add_space(6.0);
        }
        ui.add_space(12.0);
        let eligible = self.devices.get(self.selected).is_some_and(|d| d.eligible);
        if ui.add_enabled(eligible,
            egui::Button::new(RichText::new("Aplicar DPI al mouse").size(17.0).strong())
                .min_size(Vec2::new(240.0, 48.0))
        ).clicked() {
            self.apply_dpi();
        }
        ui.label(RichText::new(
            "El botón envía solo los cuatro reportes DPI. No modifica luces, botones ni rueda."
        ).color(SECONDARY).small());
        if !eligible {
            ui.colored_label(Color32::YELLOW, "No se detecta el canal HID. Usa Buscar mouse.");
        }
    }

    fn experimental(&mut self, ui: &mut egui::Ui) {
        Self::title(ui, "Otras funciones", "Disponibles para explorar, pero sin resultado físico confirmado.");
        ui.colored_label(Color32::YELLOW,
            "La iluminación ya fue probada sin efecto visible. Botones y rueda aún no se han validado.");
        ui.label(RichText::new(
            "Estos controles están separados del DPI. No se enviará ninguno al aplicar sensibilidad."
        ).color(SECONDARY));
        ui.add_space(12.0);
        ui.collapsing("Iluminación (sin verificar)", |ui| self.lighting(ui));
        ui.collapsing("Botones laterales (sin verificar)", |ui| self.buttons(ui));
        ui.collapsing("Rueda (sin verificar)", |ui| self.wheel(ui));
        ui.add_space(16.0);
        let has_changes = self.settings.apply_light || self.settings.apply_buttons || self.settings.apply_wheel;
        if has_changes {
            ui.colored_label(Color32::YELLOW,
                "Estas operaciones pueden no producir cambios o alterar la asignación de botones.");
            ui.checkbox(&mut self.armed, "Entiendo que estas funciones son experimentales");
            if ui.add_enabled(self.armed && self.devices.get(self.selected).is_some_and(|d| d.eligible),
                egui::Button::new("Aplicar funciones experimentales")
            ).clicked() {
                self.apply_experimental();
            }
        } else {
            ui.label("Abre una sección y elige qué función quieres probar.");
        }
    }

    fn lighting(&mut self, ui: &mut egui::Ui) {
        ui.label("Modo de iluminación");
        for mode in Light::ALL {
            if ui.radio_value(&mut self.settings.light, mode, mode.label()).changed() {
                self.settings.apply_light = true;
                self.reset_consent();
            }
        }
        if ui.checkbox(&mut self.settings.apply_light, "Preparar iluminación").changed() {
            self.reset_consent();
        }
        ui.label(RichText::new("La orden de apagado fue aceptada por Windows, sin efecto visible.").color(SECONDARY).small());
    }

    fn buttons(&mut self, ui: &mut egui::Ui) {
        Self::title(ui, "Asignaciones", "Dos botones laterales con funciones claras.");
        ui.group(|ui| {
            ui.heading("Boton delantero");
            ui.label(RichText::new("Identificador físico 05").color(SECONDARY));
            egui::ComboBox::from_id_salt("front_assignment")
                .selected_text(self.settings.front.label())
                .show_ui(ui, |ui| {
                    for action in Action::ALL {
                        if ui.selectable_value(&mut self.settings.front, action, action.label()).changed() {
                            self.settings.apply_buttons = true;
                            self.reset_consent();
                        }
                    }
                });
        });
        ui.add_space(10.0);
        ui.group(|ui| {
            ui.heading("Boton trasero");
            ui.label(RichText::new("Identificador físico 04").color(SECONDARY));
            egui::ComboBox::from_id_salt("rear_assignment")
                .selected_text(self.settings.rear.label())
                .show_ui(ui, |ui| {
                    for action in Action::ALL {
                        if ui.selectable_value(&mut self.settings.rear, action, action.label()).changed() {
                            self.settings.apply_buttons = true;
                            self.reset_consent();
                        }
                    }
                });
        });
        ui.add_space(12.0);
        if ui.checkbox(&mut self.settings.apply_buttons, "Preparar botones").changed() { self.reset_consent(); }
        ui.label(RichText::new("Los laterales pueden venir invertidos en algunas copias. Los clics izquierdo y derecho nunca se reasignan aqui.").color(SECONDARY));
    }

    fn wheel(&mut self, ui: &mut egui::Ui) {
        Self::title(ui, "Rueda", "Una sola decision, dos comportamientos.");
        ui.group(|ui| {
            for mode in Wheel::ALL {
                if ui.radio_value(&mut self.settings.wheel, mode, mode.label()).changed() {
                    self.settings.apply_wheel = true;
                    self.reset_consent();
                }
            }
        });
        ui.add_space(12.0);
        if ui.checkbox(&mut self.settings.apply_wheel, "Preparar rueda").changed() { self.reset_consent(); }
    }

    fn diagnostics(&mut self, ui: &mut egui::Ui) {
        Self::title(ui, "Diagnóstico USB", "Información observable; sin lecturas de firmware.");
        ui.horizontal(|ui| {
            if ui.button("Buscar de nuevo").clicked() { self.rescan(); }
            ui.label(RichText::new("VID 18F8  •  PID 1286").monospace().color(ACCENT));
        });
        ui.add_space(10.0);
        if self.devices.is_empty() {
            ui.colored_label(Color32::YELLOW, "No hay interfaces compatibles detectadas.");
        } else {
            let chosen = self.devices.get(self.selected).map(|d| d.description()).unwrap_or_default();
            let mut selection_changed = false;
            egui::ComboBox::from_id_salt("hid_path")
                .selected_text(chosen)
                .show_ui(ui, |ui| {
                    for (i, d) in self.devices.iter().enumerate() {
                        if ui.selectable_value(&mut self.selected, i, d.description()).changed() {
                            selection_changed = true;
                        }
                    }
                });
            if selection_changed {
                self.reset_consent();
            }
            if let Some(d) = self.devices.get(self.selected) {
                ui.label(format!("Fabricante: {}", d.manufacturer));
                ui.label(format!("Interface: {}", d.interface_number));
                ui.label(format!("Usage: {:04X}:{:04X}", d.usage_page, d.usage));
                ui.label(format!("Colección de configuración validada: {}", if d.eligible { "Sí (Feature 07, ocho bytes)" } else { "No" }));
                ui.label(RichText::new(&d.path).monospace().small().color(SECONDARY));
            }
        }
        ui.add_space(17.0);
        ui.heading("Canal de configuración identificado");
        ui.label("MI 1 / FF01:0001 — Feature Report 07 — 7 bytes de datos + 1 byte de ID.");
        ui.label("El diagnóstico aportado confirmó que la colección #4 anuncia este reporte. La colección del mouse (MI 0) no lo anuncia.");
        ui.label(RichText::new("Se deshabilitó la trama experimental de 16 bytes: no coincide con el descriptor Windows observado.").color(SECONDARY));
        ui.label(RichText::new("El DPI produjo un cambio observable en este dispositivo; su valor CPI exacto no está medido. La luz sigue sin respuesta comprobada.").color(SECONDARY));
        ui.label("Ejecuta a704f-diag.exe para generar %LOCALAPPDATA%\\ClonControl\\diagnostico.txt sin escribir al USB.");
        ui.add_space(17.0);
        ui.heading("Vista previa de comandos");
        match self.settings.reports() {
            Ok(packets) if packets.is_empty() => {
                ui.label("Sin cambios seleccionados para aplicar.");
            }
            Ok(packets) => {
                for (i, packet) in packets.iter().enumerate() {
                    ui.monospace(format!("{:02}  {}", i + 1, protocol::hex(packet)));
                }
            }
            Err(e) => { ui.colored_label(Color32::LIGHT_RED, e); }
        }
        ui.add_space(15.0);
        ui.heading("Último resultado real de Windows");
        ui.label(RichText::new("Este registro aparece únicamente tras una solicitud HID real, nunca al simular.").color(SECONDARY));
        if let Some(trace) = &self.last_hid_attempt {
            if ui.button("Copiar registro de la última prueba").clicked() {
                ui.ctx().copy_text(trace.clone());
            }
            ui.label(RichText::new("También se guarda en %LOCALAPPDATA%\\ClonControl\\ultima_prueba_hid.txt").color(SECONDARY));
            ui.group(|ui| {
                ui.monospace(trace);
            });
        } else {
            ui.label("Todavía no hay ninguna escritura real registrada en este equipo.");
        }
        ui.add_space(15.0);
        ui.label(RichText::new("La compatibilidad física aún debe comprobarse. No se envían comandos sin consentimiento.").color(SECONDARY));
    }

    fn action_bar(&mut self, ui: &mut egui::Ui) {
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new("Los cambios solo se envían al pulsar Aplicar.").color(SECONDARY).small());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Guardar perfiles").clicked() { self.save(); }
            });
        });
        ui.colored_label(if self.error { Color32::LIGHT_RED } else { ACCENT }, &self.status);
        ui.add_space(7.0);
    }

}

impl eframe::App for Configurator {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("header").show(ui, |ui| self.header(ui));
        egui::Panel::left("navigation")
            .resizable(false)
            .show(ui, |ui| { ui.set_min_width(195.0); self.nav(ui); });
        egui::Panel::bottom("apply_dock").show(ui, |ui| self.action_bar(ui));
        egui::CentralPanel::default().show(ui, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.add_space(15.0);
                    match self.page {
                        Page::Dpi => self.dpi(ui),
                        Page::Experimental => self.experimental(ui),
                        Page::Diagnostics => self.diagnostics(ui),
                    }
                    ui.add_space(16.0);
                });
        });
    }
}

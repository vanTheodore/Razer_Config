use cloncontrol_a704f::{device, protocol};
use eframe::egui::{self, Color32, RichText};
use protocol::{Action, Light, Settings, Wheel};
use std::{fs, path::PathBuf};

const AQUA: Color32 = Color32::from_rgb(105, 215, 206);
const MUTED: Color32 = Color32::from_rgb(170, 180, 192);

pub struct Configurator {
    settings: Settings,
    devices: Vec<device::Candidate>,
    selected: usize,
    dry_run: bool,
    armed: bool,
    status: String,
    error: bool,
}
fn config_path() -> PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("ClonControl")
        .join("a704f.json")
}
impl Configurator {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        let (settings, warning) = match fs::read_to_string(config_path()) {
            Ok(json) => match serde_json::from_str::<Settings>(&json) {
                Ok(s) => (s, None),
                Err(e) => (Settings::default(), Some(format!("Preset local invalido: {e}"))),
            },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (Settings::default(), None),
            Err(e) => (Settings::default(), Some(format!("No se puede leer preset: {e}"))),
        };
        let mut app = Self {
            settings, devices: Vec::new(), selected: 0, dry_run: true, armed: false,
            status: warning.unwrap_or_else(|| "Modo simulacion; aun no se ha enviado nada".into()),
            error: false,
        };
        app.rescan();
        app
    }

    fn message(&mut self, text: impl Into<String>, error: bool) {
        self.status = text.into();
        self.error = error;
    }
    fn rescan(&mut self) {
        self.dry_run = true;
        self.armed = false;
        match device::scan() {
            Ok(found) => {
                self.devices = found;
                self.selected = self.devices.iter().position(|d| d.eligible).unwrap_or(0);
                self.message(format!("{} interfaces HID encontradas; simulacion activa", self.devices.len()), self.devices.is_empty());
            }
            Err(e) => { self.devices.clear(); self.message(e, true); }
        }
    }
    fn save(&mut self) {
        let path = config_path();
        let result = (|| -> Result<(), String> {
            fs::create_dir_all(path.parent().ok_or("Ruta local invalida")?).map_err(|e| e.to_string())?;
            let json = serde_json::to_vec_pretty(&self.settings).map_err(|e| e.to_string())?;
            fs::write(&path, json).map_err(|e| e.to_string())?;
            Ok(())
        })();
        match result {
            Ok(()) => self.message(format!("Guardado preset LOCAL: {}", path.display()), false),
            Err(e) => self.message(format!("Error guardando preset: {e}"), true),
        }
    }
    fn apply(&mut self) {
        let reports = match self.settings.reports() {
            Ok(p) if !p.is_empty() => p,
            Ok(_) => { self.message("Selecciona al menos una seccion para aplicar", true); return; }
            Err(e) => { self.message(e, true); return; }
        };
        if self.dry_run {
            self.message(format!("Simulacion: {} reportes generados; cero enviados", reports.len()), false);
            return;
        }
        if !self.armed {
            self.message("Autoriza expresamente la escritura HID", true);
            return;
        }
        let target = self.devices.get(self.selected).filter(|d| d.eligible).map(|d| d.path.clone());
        let result = match target {
            Some(path) => device::apply(&path, &reports),
            None => Err("No hay coleccion de mouse elegible".into()),
        };
        self.dry_run = true;
        self.armed = false;
        match result {
            Ok(msg) => self.message(format!("{msg} Se reestablecio modo simulacion"), false),
            Err(e) => self.message(format!("{e}. Simulacion reactivada."), true),
        }
    }
}

impl eframe::App for Configurator {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.heading(RichText::new("CLON//CONTROL").color(AQUA).size(27.0));
                    ui.label(format!("Rust | v{} | A704F", env!("CARGO_PKG_VERSION")));
                    if ui.button("Buscar mouse").clicked() { self.rescan(); }
                });
                ui.label(RichText::new("No oficial | Experimental | Solo USB 18F8:1286 | Nunca instala drivers").color(MUTED));
                ui.separator();
                if self.devices.is_empty() {
                    ui.colored_label(Color32::YELLOW, "Mouse 18F8:1286 no detectado");
                } else {
                    egui::ComboBox::from_id_salt("usb")
                        .selected_text(self.devices.get(self.selected).map(|d| d.description()).unwrap_or_default())
                        .show_ui(ui, |ui| {
                            for (i, d) in self.devices.iter().enumerate() {
                                ui.selectable_value(&mut self.selected, i, d.description());
                            }
                        });
                    if let Some(d) = self.devices.get(self.selected) {
                        ui.label(RichText::new(format!("Fabricante: {} | MI {} | Usage {:04X}:{:04X}", d.manufacturer, d.interface_number, d.usage_page, d.usage)).small().color(MUTED));
                    }
                }
                ui.colored_label(if self.error { Color32::LIGHT_RED } else { AQUA }, &self.status);
                ui.separator();

                ui.horizontal(|ui| {
                    ui.heading("01 / DPI");
                    ui.checkbox(&mut self.settings.apply_dpi, "Aplicar DPI");
                });
                ui.label("Cuatro perfiles: 200-3000 CPI nominales, pasos de 200");
                ui.add_enabled_ui(self.settings.apply_dpi, |ui| {
                    egui::Grid::new("dpi").num_columns(5).spacing([14.0, 8.0]).show(ui, |ui| {
                        ui.strong("Perfil"); ui.strong("Activo"); ui.strong("Habilitado"); ui.strong("Paso"); ui.strong("CPI"); ui.end_row();
                        for i in 0..4 {
                            ui.label(format!("{}", i + 1));
                            ui.radio_value(&mut self.settings.active, i, "");
                            ui.checkbox(&mut self.settings.enabled[i], "");
                            ui.add(egui::Slider::new(&mut self.settings.profiles[i], 1..=15));
                            ui.label(format!("{}", self.settings.profiles[i] as u16 * 200));
                            ui.end_row();
                        }
                    });
                });
                ui.separator();

                ui.horizontal(|ui| {
                    ui.heading("02 / Iluminacion");
                    ui.checkbox(&mut self.settings.apply_light, "Aplicar RGB");
                });
                ui.add_enabled_ui(self.settings.apply_light, |ui| {
                    egui::ComboBox::from_id_salt("rgb").selected_text(self.settings.light.label()).show_ui(ui, |ui| {
                        for mode in Light::ALL {
                            ui.selectable_value(&mut self.settings.light, mode, mode.label());
                        }
                    });
                });
                ui.separator();
                ui.horizontal(|ui| {
                    ui.heading("03 / Rueda");
                    ui.checkbox(&mut self.settings.apply_wheel, "Aplicar rueda");
                });
                ui.add_enabled_ui(self.settings.apply_wheel, |ui| {
                    egui::ComboBox::from_id_salt("wheel").selected_text(self.settings.wheel.label()).show_ui(ui, |ui| {
                        for mode in Wheel::ALL {
                            ui.selectable_value(&mut self.settings.wheel, mode, mode.label());
                        }
                    });
                });
                ui.separator();

                ui.horizontal(|ui| {
                    ui.heading("04 / Botones laterales");
                    ui.checkbox(&mut self.settings.apply_buttons, "Aplicar botones");
                });
                ui.label(RichText::new("Nunca reasigna clic izquierdo o derecho. Los dos laterales pueden estar invertidos en algunas copias.").small().color(MUTED));
                ui.add_enabled_ui(self.settings.apply_buttons, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Delantero (05):");
                        egui::ComboBox::from_id_salt("front").selected_text(self.settings.front.label()).show_ui(ui, |ui| {
                            for action in Action::ALL { ui.selectable_value(&mut self.settings.front, action, action.label()); }
                        });
                        ui.label("Trasero (04):");
                        egui::ComboBox::from_id_salt("rear").selected_text(self.settings.rear.label()).show_ui(ui, |ui| {
                            for action in Action::ALL { ui.selectable_value(&mut self.settings.rear, action, action.label()); }
                        });
                    });
                });
                ui.separator();

                ui.strong("Vista previa (solo lectura)");
                egui::ScrollArea::vertical().max_height(112.0).show(ui, |ui| {
                    match self.settings.reports() {
                        Ok(packets) => {
                            if packets.is_empty() { ui.label("Nada habilitado para escribir."); }
                            for (i, p) in packets.iter().enumerate() {
                                ui.monospace(format!("{:02}  {}", i + 1, protocol::hex(p)));
                            }
                        }
                        Err(e) => { ui.colored_label(Color32::LIGHT_RED, e); }
                    }
                });
                ui.separator();
                ui.horizontal_wrapped(|ui| {
                    ui.checkbox(&mut self.dry_run, "Simulacion (NO envia)");
                    ui.checkbox(&mut self.armed, "Autorizo escritura HID experimental");
                });
                ui.label(RichText::new("Comienza por RGB apagado; desmarca DPI, rueda y botones. La confirmacion se reinicia tras cada aplicacion.").small().color(MUTED));
                ui.horizontal(|ui| {
                    if ui.button("Guardar preset local").clicked() { self.save(); }
                    if ui.button("Valores de ejemplo").clicked() {
                        self.settings = Settings::default();
                        self.dry_run = true;
                        self.armed = false;
                        self.message("Valores de ejemplo restaurados; no enviados al mouse", false);
                    }
                    if ui.button(RichText::new("APLICAR / SIMULAR").color(AQUA)).clicked() { self.apply(); }
                });
            });
        });
    }
}

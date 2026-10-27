use cloncontrol_a704f::{device, protocol};
use eframe::egui::{self, Color32, RichText, Stroke, Vec2};
use protocol::{Action, Light, Settings, Wheel};
use std::{fs, path::PathBuf};

const ACCENT: Color32 = Color32::from_rgb(100, 224, 196);
const SECONDARY: Color32 = Color32::from_rgb(165, 181, 191);
const BACKGROUND: Color32 = Color32::from_rgb(17, 21, 29);
const PANEL: Color32 = Color32::from_rgb(29, 35, 44);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Page {
    Device,
    Dpi,
    Lights,
    Buttons,
    Wheel,
    Diagnostics,
}
impl Page {
    const ALL: [Self; 6] = [
        Self::Device, Self::Dpi, Self::Lights,
        Self::Buttons, Self::Wheel, Self::Diagnostics,
    ];
    fn title(self) -> &'static str {
        match self {
            Self::Device => "Inicio",
            Self::Dpi => "Sensibilidad",
            Self::Lights => "Iluminación",
            Self::Buttons => "Botones",
            Self::Wheel => "Rueda",
            Self::Diagnostics => "Diagnóstico",
        }
    }
    fn short(self) -> &'static str {
        match self {
            Self::Device => "Un dispositivo. Todo bajo control.",
            Self::Dpi => "Cuatro velocidades, sin complicaciones.",
            Self::Lights => "Personaliza la luz del mouse.",
            Self::Buttons => "Asigna funciones a los botones laterales.",
            Self::Wheel => "Elige como se comporta la rueda.",
            Self::Diagnostics => "Identidad USB y comandos preparados.",
        }
    }
}

pub struct Configurator {
    settings: Settings,
    devices: Vec<device::Candidate>,
    selected: usize,
    page: Page,
    dry_run: bool,
    transport: protocol::Transport,
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
            page: Page::Device,
            dry_run: true,
            transport: protocol::Transport::Standard8,
            armed: false,
            status: warning.unwrap_or_else(|| "Listo para simular. No se ha escrito en USB.".into()),
            error: false,
        };
        app.rescan();
        app
    }

    fn message(&mut self, text: impl Into<String>, error: bool) {
        self.status = text.into();
        self.error = error;
    }

    fn reset_consent(&mut self) {
        self.dry_run = true;
        self.armed = false;
    }

    fn rescan(&mut self) {
        self.reset_consent();
        match device::scan() {
            Ok(found) => {
                self.devices = found;
                self.selected = self.devices.iter().position(|d| d.eligible).unwrap_or(0);
                self.message(
                    format!("{} interfaz(es) HID encontradas. Símulación activa.", self.devices.len()),
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

    fn apply(&mut self) {
        let reports = match self.settings.reports() {
            Ok(p) if !p.is_empty() => p,
            Ok(_) => {
                self.message("Selecciona una opción de DPI, luces, botones o rueda.", true);
                return;
            }
            Err(e) => {
                self.message(e, true);
                return;
            }
        };
        if self.dry_run {
            self.message(
                format!("Símulación correcta: {} reportes, 0 escrituras al mouse.", reports.len()),
                false,
            );
            return;
        }
        if !self.armed {
            self.message("Marca la autorización antes de escribir en el HID.", true);
            return;
        }
        let target = self.devices.get(self.selected).filter(|d| d.eligible).map(|d| d.path.clone());
        let result = match target {
            Some(path) => device::apply(&path, &reports, self.transport),
            None => Err("No hay una interfaz USB del mouse válida para escribir.".into()),
        };
        self.reset_consent();
        match result {
            Ok(msg) => self.message(format!("{msg} Símulación reactivada."), false),
            Err(e) => self.message(format!("{e} Símulación reactivada."), true),
        }
    }

    fn pending_count(&self) -> usize {
        self.settings.reports().map(|r| r.len()).unwrap_or(0)
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
                if connected { "● USB 18F8:1286 encontrado" } else { "● USB no encontrado" },
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("↻ Buscar mouse").clicked() { self.rescan(); }
            });
        });
        ui.add_space(8.0);
    }

    fn nav(&mut self, ui: &mut egui::Ui) {
        ui.add_space(18.0);
        ui.label(RichText::new("DISPOSITIVO").small().color(SECONDARY));
        ui.add_space(8.0);
        ui.strong("Mouse gaming USB");
        ui.label(RichText::new("18F8 : 1286").monospace().color(ACCENT));
        ui.add_space(18.0);
        ui.separator();
        ui.label(RichText::new("CONFIGURACIÓN").small().color(SECONDARY));
        ui.add_space(8.0);
        for page in Page::ALL {
            if ui.add_sized(
                [176.0, 38.0],
                egui::Button::new(page.title()).selected(self.page == page),
            ).clicked() {
                self.page = page;
            }
        }
        ui.add_space(20.0);
        ui.separator();
        ui.label(RichText::new("CLON//CONTROL 0.2").color(SECONDARY).small());
        ui.label(RichText::new("No oficial • Rust").color(SECONDARY).small());
    }

    fn title(ui: &mut egui::Ui, title: &str, subtitle: &str) {
        ui.heading(RichText::new(title).size(27.0));
        ui.label(RichText::new(subtitle).color(SECONDARY));
        ui.add_space(17.0);
    }

    fn mouse_drawing(ui: &mut egui::Ui) {
        let (rect, _) = ui.allocate_exact_size(Vec2::new(205.0, 185.0), egui::Sense::hover());
        let painter = ui.painter();
        let center = rect.center();
        let body = egui::Rect::from_center_size(center, Vec2::new(106.0, 174.0));
        painter.rect_filled(body, 51.0, Color32::from_rgb(49, 58, 67));
        let top = egui::Rect::from_center_size(
            center + egui::vec2(0.0, -37.0), Vec2::new(93.0, 92.0),
        );
        painter.rect_filled(top, 43.0, PANEL);
        painter.line_segment(
            [center + egui::vec2(0.0, -85.0), center + egui::vec2(0.0, -10.0)],
            Stroke::new(2.0, Color32::from_rgb(95, 109, 119)),
        );
        painter.rect_filled(
            egui::Rect::from_center_size(center + egui::vec2(0.0, -43.0), Vec2::new(8.0, 24.0)),
            4.0,
            ACCENT,
        );
        painter.rect_filled(
            egui::Rect::from_center_size(center + egui::vec2(-53.0, 0.0), Vec2::new(10.0, 26.0)),
            4.0,
            ACCENT,
        );
        painter.circle_filled(center + egui::vec2(0.0, 47.0), 5.0, ACCENT);
    }

    fn overview(&mut self, ui: &mut egui::Ui) {
        Self::title(ui, "Tu mouse, tus reglas", "Todos los controles del mouse en un solo lugar.");
        ui.group(|ui| {
            ui.horizontal(|ui| {
                Self::mouse_drawing(ui);
                ui.vertical(|ui| {
                    ui.add_space(12.0);
                    ui.heading("Mouse USB 18F8:1286");
                    ui.label(RichText::new("ID USB asociado a Instant A704F; controladora aún sin verificar").color(SECONDARY));
                    ui.add_space(12.0);
                    let connected = self.devices.iter().any(|d| d.eligible);
                    ui.colored_label(
                        if connected { ACCENT } else { Color32::YELLOW },
                        if connected { "Interfaz HID encontrada" } else { "Conecta el mouse y pulsa Buscar" },
                    );
                    ui.label("No se ha leído el DPI real del mouse");
                    ui.add_space(14.0);
                    if ui.button("Ver diagnóstico USB  →").clicked() { self.page = Page::Diagnostics; }
                });
            });
        });
        ui.add_space(18.0);
        ui.heading("Accesos rapidos");
        ui.columns(2, |cols| {
            cols[0].group(|ui| {
                ui.heading("Sensibilidad");
                ui.label(format!("{} CPI (valor previsto)", self.settings.profiles.get(self.settings.active).copied().unwrap_or(3) as u16 * 200));
                ui.label(RichText::new("4 perfiles configurables").color(SECONDARY));
                if ui.button("Editar DPI  →").clicked() { self.page = Page::Dpi; }
            });
            cols[1].group(|ui| {
                ui.heading("Iluminación");
                ui.label(self.settings.light.label());
                ui.label(RichText::new("Cambiar efectos sin Synapse").color(SECONDARY));
                if ui.button("Editar luces  →").clicked() { self.page = Page::Lights; }
            });
        });
        ui.add_space(13.0);
        ui.group(|ui| {
            ui.heading("Primera prueba recomendada");
            ui.label("1. Abre Diagnóstico para confirmar el HID correcto.");
            ui.label("2. En Iluminación selecciona Apagada.");
            ui.label("3. Pulsa Símular y revisa el reporte.");
            ui.label("4. Solo después autoriza una escritura real.");
            if ui.button("Preparar prueba RGB apagado").clicked() {
                self.settings.apply_dpi = false;
                self.settings.apply_wheel = false;
                self.settings.apply_buttons = false;
                self.settings.apply_light = true;
                self.settings.light = Light::Off;
                self.reset_consent();
                self.page = Page::Lights;
                self.message("RGB apagado preparado. Símula antes de conectar realmente.", false);
            }
        });
    }

    fn dpi(&mut self, ui: &mut egui::Ui) {
        Self::title(ui, "Sensibilidad (DPI)", "Elige hasta cuatro velocidades y decide cual está activa.");
        ui.label(RichText::new("200 a 3000 CPI nominales, en incrementos de 200.").color(SECONDARY));
        ui.add_space(12.0);
        let active = self.settings.active.min(3);
        ui.group(|ui| {
            ui.heading(format!("DPI seleccionado: {} CPI", self.settings.profiles[active] as u16 * 200));
            ui.label(RichText::new("Valor previsto; no se puede leer el DPI real desde este configurador.").color(SECONDARY));
            let changed = ui.add(egui::Slider::new(&mut self.settings.profiles[active], 1..=15)
                .text("Ajustar sensibilidad principal (×200 CPI)")).changed();
            if changed {
                self.settings.enabled[active] = true;
                self.settings.apply_dpi = true;
                self.reset_consent();
            }
            ui.label("Para cambiar de perfil, marca «Usar» en uno de los cuatro perfiles inferiores.");
        });
        ui.add_space(12.0);
        for i in 0..4 {
            let mut changed = false;
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.heading(format!("Perfil {}", i + 1));
                    if ui.radio_value(&mut self.settings.active, i, "Usar").changed() { changed = true; }
                    if ui.checkbox(&mut self.settings.enabled[i], "Habilitado").changed() { changed = true; }
                    ui.colored_label(ACCENT, format!("{} CPI", self.settings.profiles[i] as u16 * 200));
                });
                if ui.add(egui::Slider::new(&mut self.settings.profiles[i], 1..=15).text("Velocidad")).changed() {
                    changed = true;
                }
            });
            if changed {
                self.settings.apply_dpi = true;
                self.reset_consent();
            }
            ui.add_space(6.0);
        }
        ui.label(RichText::new("Se envían los cuatro perfiles cuando aplicas el DPI. El activo debe estár habilitado.").color(SECONDARY));
        ui.checkbox(&mut self.settings.apply_dpi, "Incluir DPI en la siguiente aplicación");
    }

    fn lighting(&mut self, ui: &mut egui::Ui) {
        Self::title(ui, "Iluminación", "Elige un efecto y pruébalo; el cambio no es automático."); 
        if ui.button("Preparar prueba: apagar la luz (solo RGB)").clicked() {
            self.settings.apply_light = true;
            self.settings.light = Light::Off;
            self.settings.apply_dpi = false;
            self.settings.apply_wheel = false;
            self.settings.apply_buttons = false;
            self.reset_consent();
            self.message("Iluminación apagada preparada. Pulsa Símular; luego autoriza una prueba real.", false);
        }
        ui.add_space(8.0);
        ui.group(|ui| {
            ui.label(RichText::new("EFECTO").color(SECONDARY));
            ui.add_space(8.0);
            for mode in Light::ALL {
                if ui.radio_value(&mut self.settings.light, mode, mode.label()).changed() {
                    self.settings.apply_light = true;
                    self.reset_consent();
                }
            }
        });
        ui.add_space(16.0);
        ui.checkbox(&mut self.settings.apply_light, "Incluir iluminación en la siguiente aplicación");
        ui.label(RichText::new("Solo modos predefinidos. No es compatible con RGB por zona ni colores personalizados.").color(SECONDARY));
        ui.label(RichText::new("Sí no cambia al aplicarlo, visita Diagnóstico: el envío de 8 bytes no está verificado en Windows.").color(SECONDARY));
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
        ui.checkbox(&mut self.settings.apply_buttons, "Incluir botones en la siguiente aplicación");
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
        ui.checkbox(&mut self.settings.apply_wheel, "Incluir rueda en la siguiente aplicación");
    }

    fn diagnostics(&mut self, ui: &mut egui::Ui) {
        Self::title(ui, "Diagnóstico USB", "Informacion observable; sin lecturas de firmware.");
        ui.horizontal(|ui| {
            if ui.button("↻ Volver a buscar").clicked() { self.rescan(); }
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
                ui.label(format!("Escribible segun heuristica: {}", if d.eligible { "Sí" } else { "No" }));
                ui.label(RichText::new(&d.path).monospace().small().color(SECONDARY));
            }
        }
        ui.add_space(17.0);
        ui.heading("Formato de comunicación HID");
        ui.label("El proyecto original Linux utiliza 16 bytes; nuestra versión Windows comenzó con 8. Ninguno se ha válidado en tu hardware.");
        let changed = ui.radio_value(&mut self.transport, protocol::Transport::Standard8,
            protocol::Transport::Standard8.label()).changed()
            | ui.radio_value(&mut self.transport, protocol::Transport::LinuxReference16,
            protocol::Transport::LinuxReference16.label()).changed();
        if changed { self.reset_consent(); }
        if self.transport == protocol::Transport::LinuxReference16 {
            ui.colored_label(Color32::YELLOW,
                "EXPERIMENTAL: permite únicamente una prueba de iluminación. Se desconoce si Windows acepta el ID 0x21.");
        }
        ui.label("Ejecuta a704f-diag.exe para generar %LOCALAPPDATA%\\ClonControl\\diagnostico.txt sin escribir al USB.");
        ui.add_space(17.0);
        ui.heading("Vista previa de comandos");
        match self.settings.reports() {
            Ok(packets) if packets.is_empty() => {
                ui.label("Sín cambios seleccionados para aplicar.");
            }
            Ok(packets) => {
                for (i, packet) in packets.iter().enumerate() {
                    match self.transport.encode(packet) {
                        Ok(bytes) => ui.monospace(format!("{:02}  {}", i + 1,
                            bytes.iter().map(|b| format!("{b:02X}")).collect::<Vec<_>>().join(" "))),
                        Err(e) => ui.colored_label(Color32::LIGHT_RED, e),
                    };
                }
            }
            Err(e) => { ui.colored_label(Color32::LIGHT_RED, e); }
        }
        ui.add_space(15.0);
        ui.label(RichText::new("Estos comandos son experimentales y no garantizan efecto en tu unidad. Ninguna pantalla envía datos sin confirmacion.").color(SECONDARY));
    }

    fn action_bar(&mut self, ui: &mut egui::Ui) {
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new("REPORTES PREPARADOS").color(SECONDARY).small());
            ui.colored_label(ACCENT, format!("{} reportes", self.pending_count()));
            ui.separator();
            ui.checkbox(&mut self.dry_run, "Solo simular (no cambia nada)");
            ui.checkbox(&mut self.armed, "Autorizo una prueba real");
            if ui.button("Símular").clicked() {
                self.dry_run = true;
                self.apply();
            }
            if ui.add_enabled(!self.dry_run && self.armed && self.pending_count() > 0,
                egui::Button::new(RichText::new("APLICAR AL MOUSE").strong().color(ACCENT))
            ).clicked() {
                self.apply();
            }
            if ui.button("Guardar preset").clicked() { self.save(); }
        });
        ui.label(RichText::new(
            "Para cambiar el DPI o las luces: configura, pulsa Símular, desactiva «Solo simular», autoriza y aplica. El permiso se revoca tras cada envío.",
        ).color(SECONDARY).small());
        ui.colored_label(
            if self.error { Color32::LIGHT_RED } else { ACCENT },
            &self.status,
        );
        ui.add_space(9.0);
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
                        Page::Device => self.overview(ui),
                        Page::Dpi => self.dpi(ui),
                        Page::Lights => self.lighting(ui),
                        Page::Buttons => self.buttons(ui),
                        Page::Wheel => self.wheel(ui),
                        Page::Diagnostics => self.diagnostics(ui),
                    }
                    ui.add_space(16.0);
                });
        });
    }
}

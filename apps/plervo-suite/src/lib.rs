//! The Plervo Suite shell. Slow operations run outside the UI; headless checks use the same engine.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]
mod family;
mod locale;
use egui::{Color32, RichText, Vec2};
use locale::Language;
use plervo_suite_core::{
    Installed, Release,
    catalog::{CraftApp, PDF, SUITE},
    protocol::JobPhase,
};
use plervo_suite_native::{http::Stop, manager, paths::Paths};
use std::{
    collections::BTreeMap,
    fs::File,
    sync::{Arc, mpsc},
};

const INK: Color32 = Color32::from_rgb(25, 39, 33);
const PAPER: Color32 = Color32::from_rgb(247, 248, 244);
const LIME: Color32 = Color32::from_rgb(202, 232, 98);
const MUTED: Color32 = Color32::from_rgb(89, 102, 94);
#[derive(Clone, Copy)]
enum Action {
    Scan,
    Refresh,
    Install,
    Open,
    Remove,
}
enum Event {
    Scanned(BTreeMap<String, Installed>),
    Refreshed(Result<Release, String>, Option<Installed>),
    Progress(JobPhase),
    Finished(Result<(), String>, Option<Installed>),
}

fn acquire(paths: &Paths) -> Result<File, String> {
    std::fs::create_dir_all(&paths.data).map_err(|e| e.to_string())?;
    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(paths.data.join("suite.lock"))
        .map_err(|e| e.to_string())?;
    file.try_lock().map_err(|_| "Plervo Suite is already open. Close the other window first.".to_string())?;
    Ok(file)
}

pub struct SuiteApp {
    language: Language,
    system_language: Language,
    settings: bool,
    catalog: bool,
    detail: bool,
    selected: &'static family::Product,
    icons: std::collections::BTreeMap<&'static str, egui::TextureHandle>,
    release: Option<Release>,
    installed: Option<Installed>,
    installed_apps: BTreeMap<String, Installed>,
    paths: Option<Paths>,
    _lock: Option<File>,
    tx: mpsc::Sender<Event>,
    rx: mpsc::Receiver<Event>,
    stop: Arc<Stop>,
    busy: bool,
    progress: Option<JobPhase>,
    error: Option<String>,
    success: bool,
    confirm_remove: bool,
}
impl Default for SuiteApp {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            language: Language::Auto,
            system_language: Language::Auto.resolve(),
            settings: false,
            catalog: false,
            detail: false,
            selected: &family::PRODUCTS[0],
            icons: Default::default(),
            release: None,
            installed: None,
            installed_apps: BTreeMap::new(),
            paths: None,
            _lock: None,
            tx,
            rx,
            stop: Arc::new(Stop::default()),
            busy: false,
            progress: None,
            error: None,
            success: false,
            confirm_remove: false,
        }
    }
}
impl SuiteApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self::style(&cc.egui_ctx);
        let mut app = Self::default();
        if let Some(saved) = cc.storage.and_then(|s| s.get_string("language")) {
            app.language = serde_json::from_str(&saved).unwrap_or_default();
        }
        match Paths::new().and_then(|paths| acquire(&paths).map(|lock| (paths, lock))) {
            Ok((paths, lock)) => {
                app.paths = Some(paths);
                app._lock = Some(lock);
                app.start(Action::Scan, &cc.egui_ctx);
            }
            Err(e) => app.error = Some(e),
        }
        app
    }
    pub fn is_busy(&self) -> bool {
        self.busy
    }
    pub fn style(ctx: &egui::Context) {
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert("inter".into(), Arc::new(egui::FontData::from_static(include_bytes!("../../../assets/fonts/Inter-Regular.ttf"))));
        fonts.families.entry(egui::FontFamily::Proportional).or_default().insert(0, "inter".into());
        ctx.set_fonts(fonts);
        let mut style = (*ctx.style_of(egui::Theme::Light)).clone();
        style.visuals = egui::Visuals::light();
        style.visuals.override_text_color = Some(INK);
        style.visuals.panel_fill = PAPER;
        style.visuals.selection.bg_fill = LIME;
        style.visuals.selection.stroke = egui::Stroke::new(1.0, INK);
        style.spacing.item_spacing = Vec2::new(12.0, 12.0);
        style.spacing.button_padding = Vec2::new(17.0, 11.0);
        style.text_styles.insert(egui::TextStyle::Body, egui::FontId::proportional(15.0));
        style.text_styles.insert(egui::TextStyle::Button, egui::FontId::proportional(15.0));
        ctx.set_style_of(egui::Theme::Light, style);
        ctx.set_theme(egui::Theme::Light);
    }
    fn lang(&self) -> Language {
        if self.language == Language::Auto { self.system_language } else { self.language }
    }
    fn icon(&mut self, ui: &mut egui::Ui, id: &'static str, bytes: &'static [u8], size: f32) {
        if let std::collections::btree_map::Entry::Vacant(entry) = self.icons.entry(id)
            && let Ok(icon) = eframe::icon_data::from_png_bytes(bytes)
        {
            let pixels = egui::ColorImage::from_rgba_unmultiplied([icon.width as usize, icon.height as usize], &icon.rgba);
            entry.insert(ui.ctx().load_texture(id, pixels, egui::TextureOptions::LINEAR));
        }
        if let Some(texture) = self.icons.get(id) {
            ui.image((texture.id(), Vec2::splat(size)));
        } else {
            ui.label(id);
        }
    }
    fn catalog(&mut self, ui: &mut egui::Ui, installed_only: bool) {
        let l = self.lang();
        let t = |en, pt, es| l.text(en, pt, es);
        ui.heading(
            RichText::new(if installed_only {
                t("Your installed apps.", "Seus apps instalados.", "Tus apps instaladas.")
            } else {
                t("Your tools, together.", "Suas ferramentas, juntas.", "Tus herramientas, juntas.")
            })
            .size(30.0),
        );
        ui.label(if installed_only {
            t("Open an app or manage its updates.", "Abra um app ou gerencie suas atualizações.", "Abre una app o gestiona sus actualizaciones.")
        } else {
            t(
                "Choose an app to see its available downloads and updates.",
                "Escolha um app para ver os downloads e as atualizações disponíveis.",
                "Elige un app para ver las descargas y actualizaciones disponibles.",
            )
        });
        if installed_only {
            if ui.add_enabled(!self.busy, egui::Button::new(t("Refresh apps", "Atualizar lista", "Actualizar lista"))).clicked() {
                self.start(Action::Scan, ui.ctx());
            }
            if self.busy {
                ui.spinner();
            }
            if let Some(error) = &self.error {
                ui.label(error);
            }
        }
        ui.add_space(20.0);
        let products: Vec<_> =
            family::PRODUCTS.iter().filter(|p| !installed_only || self.installed_apps.contains_key(&format!("plervo-{}", p.id))).collect();
        if products.is_empty() && !self.busy {
            ui.label(t("No apps installed yet.", "Nenhum app instalado ainda.", "Todavía no hay apps instaladas."));
            if ui.button(t("Browse all apps", "Ver todos os apps", "Ver todas las apps")).clicked() {
                self.catalog = true;
            }
        }
        let count = if ui.available_width() >= 690.0 {
            3
        } else if ui.available_width() >= 440.0 {
            2
        } else {
            1
        };
        egui::ScrollArea::vertical().id_salt(if installed_only { "installed-apps" } else { "all-apps" }).show(ui, |ui| {
            for row in products.chunks(count) {
                ui.columns(count, |columns| {
                    for (product, column) in row.iter().zip(columns) {
                        egui::Frame::new().fill(Color32::WHITE).corner_radius(12).inner_margin(16).show(column, |ui| {
                            ui.set_min_width(ui.available_width());
                            ui.set_min_height(176.0);
                            self.icon(ui, product.id, product.icon, 48.0);
                            ui.label(RichText::new(product.name).size(19.0).strong());
                            ui.label(RichText::new(t(product.description[0], product.description[1], product.description[2])).color(MUTED));
                            if let Some(installed) = self.installed_apps.get(&format!("plervo-{}", product.id)) {
                                if let Some(version) = &installed.version {
                                    ui.label(RichText::new(version.to_string()).small().color(MUTED));
                                }
                                if installed_only
                                    && ui
                                        .add_enabled(
                                            !self.busy && self.paths.is_some(),
                                            egui::Button::new(format!("{} {}", t("Open", "Abrir", "Abrir"), product.name)).fill(LIME),
                                        )
                                        .clicked()
                                {
                                    self.selected = product;
                                    self.start(Action::Open, ui.ctx());
                                }
                            }
                            let label = format!("{} {}", t("Manage", "Gerenciar", "Gestionar"), product.name);
                            if ui.add_enabled(!self.busy, egui::Button::new(label)).clicked() {
                                self.select(product, ui.ctx());
                            }
                        });
                    }
                });
                ui.add_space(12.0);
            }
        });
    }
    fn select(&mut self, product: &'static family::Product, ctx: &egui::Context) {
        if self.busy {
            return;
        }
        self.selected = product;
        self.catalog = false;
        self.detail = true;
        self.release = None;
        self.installed = self.installed_apps.get(&format!("plervo-{}", product.id)).cloned();
        self.error = None;
        self.success = false;
        self.confirm_remove = false;
        self.start(Action::Refresh, ctx);
    }
    fn start(&mut self, action: Action, ctx: &egui::Context) {
        if self.busy {
            return;
        }
        let Some(paths) = self.paths.clone() else { return };
        let Some(app) = CraftApp::by_id(&format!("plervo-{}", self.selected.id)) else {
            self.error = Some("Unknown Plervo product".into());
            return;
        };
        let release = self.release.clone();
        if matches!(action, Action::Install) && release.is_none() {
            return;
        }
        self.busy = true;
        self.error = None;
        self.success = false;
        self.progress = None;
        self.stop = Arc::new(Stop::default());
        let tx = self.tx.clone();
        let stop = self.stop.clone();
        let context = ctx.clone();
        let result = std::thread::Builder::new().name("plervo-operation".into()).spawn(move || {
            let mut progress = |phase| {
                let _ = tx.send(Event::Progress(phase));
                context.request_repaint();
            };
            let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                if matches!(action, Action::Scan) {
                    let installed = SUITE.iter().filter_map(|app| manager::detected_for(app).map(|i| (app.id.to_owned(), i))).collect();
                    let _ = tx.send(Event::Scanned(installed));
                } else if matches!(action, Action::Refresh) {
                    let _ = tx.send(Event::Refreshed(manager::latest_for(app), manager::detected_for(app)));
                } else {
                    let result = match action {
                        Action::Install => match release {
                            Some(r) => manager::install_app(app, &r, &paths, &stop, &mut progress),
                            None => Err("No release selected".into()),
                        },
                        Action::Open => manager::open_app(app, &paths),
                        Action::Remove => manager::uninstall_app(app, &paths, &stop, &mut progress),
                        Action::Scan | Action::Refresh => Ok(()),
                    };
                    let _ = tx.send(Event::Finished(result, manager::detected_for(app)));
                }
            }));
            if outcome.is_err() {
                let _ = tx.send(Event::Finished(Err("The operation ended unexpectedly. Check the installed version before retrying.".into()), None));
            }
            context.request_repaint();
        });
        if let Err(e) = result {
            self.busy = false;
            self.error = Some(e.to_string());
        }
    }
    fn remember_selected(&mut self, installed: &Option<Installed>) {
        let id = format!("plervo-{}", self.selected.id);
        if let Some(installed) = installed {
            self.installed_apps.insert(id, installed.clone());
        } else {
            self.installed_apps.remove(&id);
        }
    }
    fn poll(&mut self) {
        while let Ok(event) = self.rx.try_recv() {
            match event {
                Event::Scanned(installed) => {
                    self.busy = false;
                    self.installed_apps = installed;
                    self.installed = self.installed_apps.get(&format!("plervo-{}", self.selected.id)).cloned();
                }
                Event::Refreshed(result, installed) => {
                    self.busy = false;
                    self.remember_selected(&installed);
                    self.installed = installed;
                    match result {
                        Ok(r) => self.release = Some(r),
                        Err(e) => self.error = Some(e),
                    }
                }
                Event::Progress(p) => self.progress = Some(p),
                Event::Finished(result, installed) => {
                    self.busy = false;
                    self.progress = None;
                    if result.is_ok() || installed.is_some() {
                        self.remember_selected(&installed);
                    }
                    self.installed = installed;
                    self.success = result.is_ok();
                    self.error = result.err();
                }
            }
        }
    }
    pub fn show(&mut self, root: &mut egui::Ui) {
        let ctx = &root.ctx().clone();
        root.set_style(ctx.style_of(egui::Theme::Light));
        self.poll();
        let l = self.lang();
        let t = |en, pt, es| l.text(en, pt, es);
        if self.busy && ctx.input(|i| i.viewport().close_requested()) {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        }
        egui::Panel::left("navigation").exact_size(190.0).resizable(false).frame(egui::Frame::new().fill(INK).inner_margin(24)).show(root, |ui| {
            ui.visuals_mut().override_text_color = Some(Color32::WHITE);
            ui.visuals_mut().hyperlink_color = LIME;
            ui.add_space(15.0);
            ui.label(RichText::new("Plervo").size(32.0).strong().color(Color32::WHITE));
            ui.label(RichText::new("S U I T E").size(11.0).color(LIME));
            ui.add_space(58.0);
            if ui
                .selectable_label(
                    !self.settings && !self.catalog,
                    RichText::new(t("My apps", "Meus apps", "Mis apps")).color(if self.settings || self.catalog { Color32::WHITE } else { INK }),
                )
                .clicked()
            {
                self.settings = false;
                self.catalog = false;
                self.detail = false;
                self.confirm_remove = false;
                self.start(Action::Scan, ctx);
            }
            ui.add_space(8.0);
            if ui
                .selectable_label(
                    self.catalog,
                    RichText::new(t("All apps", "Todos os apps", "Todos los apps")).color(if self.catalog { INK } else { Color32::WHITE }),
                )
                .clicked()
            {
                self.settings = false;
                self.catalog = true;
            }
            ui.add_space(8.0);
            if ui
                .selectable_label(
                    self.settings,
                    RichText::new(t("Settings", "Configurações", "Configuración")).color(if self.settings { INK } else { Color32::WHITE }),
                )
                .clicked()
            {
                self.settings = true;
                self.catalog = false;
            }
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.label(RichText::new(format!("Alpha · {}", env!("CARGO_PKG_VERSION"))).size(11.0).color(Color32::from_gray(180)));
                ui.add_space(8.0);
                ui.hyperlink_to(t("Visit website ↗", "Visitar site ↗", "Visitar web ↗"), "https://plervo.com");
            });
        });
        egui::CentralPanel::default().frame(egui::Frame::new().fill(PAPER).inner_margin(36)).show(root, |ui| {
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(t("Support", "Apoiar", "Apoyar")).on_hover_text("https://buymeacoffee.com/alport").clicked() {
                        ui.ctx().open_url(egui::OpenUrl::new_tab("https://buymeacoffee.com/alport"));
                    }
                });
            });
            ui.add_space(8.0);
            if self.settings {
                ui.heading(RichText::new(t("Make it yours.", "Do seu jeito.", "A tu manera.")).size(34.0));
                ui.add_space(25.0);
                ui.label(t("Interface language", "Idioma da interface", "Idioma de la interfaz"));
                egui::ComboBox::from_label(t("Language", "Idioma", "Idioma"))
                    .selected_text(match self.language {
                        Language::Auto => t("System language", "Idioma do sistema", "Idioma del sistema"),
                        Language::En => "English",
                        Language::Pt => "Português",
                        Language::Es => "Español",
                    })
                    .show_ui(ui, |ui| {
                        for (value, label) in [
                            (Language::Auto, t("System language", "Idioma do sistema", "Idioma del sistema")),
                            (Language::En, "English"),
                            (Language::Pt, "Português"),
                            (Language::Es, "Español"),
                        ] {
                            ui.selectable_value(&mut self.language, value, label);
                        }
                    });
                ui.add_space(12.0);
                ui.label(
                    RichText::new(t(
                        "New installations follow your operating system. You can choose another language here.",
                        "Novas instalações seguem o sistema operacional. Você pode escolher outro idioma aqui.",
                        "Las nuevas instalaciones siguen el sistema operativo. Puedes elegir otro idioma aquí.",
                    ))
                    .color(MUTED),
                );
                ui.add_space(36.0);
                ui.separator();
                ui.add_space(12.0);
                ui.label("Plervo Suite");
                ui.label(t("Published by Maintech Solutions LLC", "Publicado por Maintech Solutions LLC", "Publicado por Maintech Solutions LLC"));
                ui.hyperlink_to(t("Source & licenses ↗", "Código e licenças ↗", "Código y licencias ↗"), "https://github.com/alportblu/plervo-pdf");
                ui.label(RichText::new("Installation engine adapted from ArtCraft Launcher · MIT / Apache-2.0").small().color(MUTED));
                return;
            }
            if self.catalog {
                self.catalog(ui, false);
                return;
            }
            if !self.detail {
                self.catalog(ui, true);
                return;
            }
            ui.label(RichText::new(t("YOUR WORKSPACE", "SEU ESPAÇO DE TRABALHO", "TU ESPACIO DE TRABAJO")).size(11.0).color(MUTED));
            ui.add_space(7.0);
            ui.heading(RichText::new(t("Good tools. All yours.", "Boas ferramentas. Suas.", "Buenas herramientas. Tuyas.")).size(34.0));
            ui.add_space(8.0);
            ui.label(
                RichText::new(t(
                    "Install the apps you need. Keep them together.",
                    "Instale os apps que precisa. Encontre tudo aqui.",
                    "Instala los apps que necesitas. Todo en un lugar.",
                ))
                .color(MUTED),
            );
            ui.add_space(30.0);
            egui::Frame::new()
                .fill(Color32::WHITE)
                .stroke(egui::Stroke::new(1.0, Color32::from_rgb(217, 224, 214)))
                .corner_radius(12)
                .inner_margin(26)
                .show(ui, |ui| {
                    ui.set_min_width((ui.available_width() - 2.0).max(0.0));
                    ui.horizontal(|ui| {
                        self.icon(ui, self.selected.id, self.selected.icon, 68.0);
                        ui.add_space(10.0);
                        ui.vertical(|ui| {
                            ui.label(RichText::new(self.selected.name).size(24.0).strong());
                            ui.label(
                                RichText::new(t(self.selected.description[0], self.selected.description[1], self.selected.description[2]))
                                    .color(MUTED),
                            );
                        });
                    });
                    ui.add_space(24.0);
                    ui.label(t(
                        "Install, update or open this app from your Suite.",
                        "Instale, atualize ou abra este app pelo Suite.",
                        "Instala, actualiza o abre este app desde Suite.",
                    ));
                    ui.add_space(20.0);
                    ui.separator();
                    ui.add_space(8.0);
                    let installed_version = self.installed.as_ref().and_then(|i| i.version.as_ref());
                    let update = self.release.as_ref().and_then(|r| r.version.as_ref()).zip(installed_version).is_some_and(|(new, old)| new > old);
                    ui.horizontal_wrapped(|ui| {
                        let ready = !self.busy && self.paths.is_some();
                        if self.installed.is_some() {
                            if ui
                                .add_enabled(ready, egui::Button::new(format!("{} {}", t("Open", "Abrir", "Abrir"), self.selected.name)).fill(LIME))
                                .clicked()
                            {
                                self.start(Action::Open, ctx);
                            }
                            if update && ui.add_enabled(ready, egui::Button::new(t("Update", "Atualizar", "Actualizar"))).clicked() {
                                self.start(Action::Install, ctx);
                            }
                            if ui.add_enabled(ready, egui::Button::new(t("Uninstall…", "Desinstalar…", "Desinstalar…"))).clicked() {
                                self.confirm_remove = true;
                            }
                        } else if ui
                            .add_enabled(
                                ready && self.release.is_some() && manager::supported_platform().is_some(),
                                egui::Button::new(format!("{} {}", t("Install", "Instalar", "Instalar"), self.selected.name)).fill(LIME),
                            )
                            .clicked()
                        {
                            self.start(Action::Install, ctx);
                        }
                        if let Some(r) = &self.release {
                            ui.label(RichText::new(format!("{} · Alpha", r.version_label())).small().color(MUTED));
                        }
                    });
                    if manager::supported_platform().is_none() {
                        ui.label(t(
                            "Downloads for this platform are still being validated.",
                            "Os downloads deste sistema ainda estão em validação.",
                            "Las descargas para este sistema siguen en validación.",
                        ));
                    }
                });
            ui.add_space(18.0);
            if self.busy {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(match self.progress {
                        Some(JobPhase::Installing) => t(
                            "Installing. Keep this window open…",
                            "Instalando. Mantenha esta janela aberta…",
                            "Instalando. Mantén esta ventana abierta…",
                        ),
                        Some(JobPhase::Verifying) => t("Verifying download…", "Verificando download…", "Verificando descarga…"),
                        Some(JobPhase::Removing) => t("Removing app…", "Removendo app…", "Eliminando app…"),
                        Some(_) => t("Downloading…", "Baixando…", "Descargando…"),
                        None => t("Working…", "Aguarde…", "Un momento…"),
                    });
                });
                if let Some(JobPhase::Downloading { done, total } | JobPhase::Reconnecting { done, total }) = self.progress {
                    ui.add(egui::ProgressBar::new(if total > 0 { done as f32 / total as f32 } else { 0.0 }).show_percentage());
                    if ui.button(t("Cancel download", "Cancelar download", "Cancelar descarga")).clicked() {
                        self.stop.cancel();
                    }
                }
            } else if self.success {
                ui.label(t("Done.", "Pronto.", "Listo."));
            }
            if let Some(error) = &self.error {
                egui::Frame::new().fill(Color32::from_rgb(255, 238, 230)).inner_margin(14).corner_radius(8).show(ui, |ui| {
                    ui.label(t("We couldn't complete this step.", "Não foi possível concluir esta etapa.", "No se pudo completar este paso."));
                    ui.label(RichText::new(error).small());
                });
            }
            ui.add_space(18.0);
            ui.horizontal(|ui| {
                if ui.add_enabled(!self.busy, egui::Button::new(t("Check for updates", "Verificar atualizações", "Buscar actualizaciones"))).clicked()
                {
                    self.start(Action::Refresh, ctx);
                }
                ui.label(
                    RichText::new(t(
                        "More apps will appear here as they become ready.",
                        "Novos apps aparecerão aqui quando estiverem prontos.",
                        "Aparecerán más apps cuando estén listos.",
                    ))
                    .small()
                    .color(MUTED),
                );
            });
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.label(
                    RichText::new(t(
                        "Early access · Keep backups of important documents.",
                        "Acesso antecipado · Guarde cópias dos documentos importantes.",
                        "Acceso anticipado · Guarda copias de los documentos importantes.",
                    ))
                    .size(11.0)
                    .color(MUTED),
                );
            });
        });
        if self.confirm_remove {
            egui::Window::new(format!("{} {}?", t("Uninstall", "Desinstalar", "Desinstalar"), self.selected.name))
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(t(
                        "Your documents and preferences will be kept.",
                        "Seus documentos e preferências serão mantidos.",
                        "Se conservarán tus documentos y preferencias.",
                    ));
                    ui.horizontal(|ui| {
                        if ui.button(t("Cancel", "Cancelar", "Cancelar")).clicked() {
                            self.confirm_remove = false;
                        }
                        if ui.button(t("Uninstall", "Desinstalar", "Desinstalar")).clicked() {
                            self.confirm_remove = false;
                            self.start(Action::Remove, ctx);
                        }
                    });
                });
        }
    }
}
impl eframe::App for SuiteApp {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        self.show(ui);
    }
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        if let Ok(s) = serde_json::to_string(&self.language) {
            storage.set_string("language", s);
        }
    }
}

/// Explicit commands only: no listener, background service, login item or agent registration.
pub fn headless(args: &[String]) -> Result<(), String> {
    let cmd = args.first().map(String::as_str).unwrap_or_default();
    let legacy = matches!(cmd, "--install-pdf" | "--open-pdf" | "--uninstall-pdf");
    let product = args.get(1).map(String::as_str);
    let valid = match cmd {
        "--check" | "--status" => args.len() == 1 || args.len() == 2,
        "--install" | "--open" | "--uninstall" => args.len() == 2,
        _ => legacy && args.len() == 1,
    };
    if !valid {
        return Err("Usage: plervo-suite [--version | --check [PRODUCT] | --status [PRODUCT] | --install PRODUCT | --open PRODUCT | --uninstall PRODUCT]. PRODUCT: pdf, photo, vector, video, develop, motion, publish, write, slides, sheets, cad, audio, canvas. Legacy PDF commands remain supported.".into());
    }
    let app = match product {
        Some(id) => CraftApp::by_id(&format!("plervo-{id}")).ok_or("Unknown Plervo product")?,
        None => &PDF,
    };
    match cmd {
        "--check" => {
            let r = manager::latest_for(app)?;
            println!("{}", serde_json::to_string_pretty(&r).map_err(|e| e.to_string())?);
            Ok(())
        }
        "--status" => {
            println!("{}", serde_json::to_string_pretty(&manager::detected_for(app)).map_err(|e| e.to_string())?);
            Ok(())
        }
        _ => {
            let paths = Paths::new()?;
            let _lock = acquire(&paths)?;
            let stop = Stop::default();
            let mut progress = |p| eprintln!("{p:?}");
            match cmd {
                "--install" | "--install-pdf" => manager::install_app(app, &manager::latest_for(app)?, &paths, &stop, &mut progress),
                "--open" | "--open-pdf" => manager::open_app(app, &paths),
                _ => manager::uninstall_app(app, &paths, &stop, &mut progress),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::{Harness, kittest::Queryable};
    fn harness() -> Harness<'static, SuiteApp> {
        let app = SuiteApp { system_language: Language::En, ..Default::default() };
        Harness::builder().with_size(egui::vec2(1000.0, 700.0)).build_ui_state(|ui, app| app.show(ui), app)
    }
    #[test]
    fn my_apps_keeps_all_detected_apps_when_offline() {
        let mut h = harness();
        for id in ["pdf", "photo"] {
            h.state_mut().selected = family::PRODUCTS.iter().find(|p| p.id == id).unwrap();
            let installed = Installed {
                version: plervo_suite_core::version::parse("0.1.0-alpha.5"),
                source: plervo_suite_core::status::InstallSource::Launcher,
                location: format!("/synthetic/plervo-{id}"),
                package: None,
            };
            h.state().tx.send(Event::Refreshed(Err("Offline".into()), Some(installed))).unwrap();
            h.state_mut().poll();
        }
        h.run();
        h.get_by_label("My apps").click();
        h.run();
        h.get_by_label("Plervo PDF");
        h.get_by_label("Plervo Photo");
        assert!(h.query_by_label("Plervo Video").is_none());
        if let Some(path) = std::env::var_os("PLERVO_LIBRARY_PREVIEW") {
            SuiteApp::style(&h.ctx);
            h.run();
            h.render().unwrap().save(path).unwrap();
        }
        h.get_by_label("Manage Plervo PDF").click();
        h.run();
        assert_eq!(h.state().selected.id, "pdf");
        h.get_by_label("Open Plervo PDF");
    }
    #[test]
    fn library_scan_and_removal_keep_each_apps_state_separate() {
        let mut app = SuiteApp::default();
        let installed = || Installed {
            version: plervo_suite_core::version::parse("0.1.0-alpha.5"),
            source: plervo_suite_core::status::InstallSource::Launcher,
            location: "/synthetic/app".into(),
            package: None,
        };
        let inventory = SUITE.iter().map(|p| (p.id.to_owned(), installed())).collect();
        app.tx.send(Event::Scanned(inventory)).unwrap();
        app.poll();
        assert_eq!(app.installed_apps.len(), 13);
        app.tx.send(Event::Finished(Err("busy".into()), None)).unwrap();
        app.poll();
        assert_eq!(app.installed_apps.len(), 13);
        app.tx.send(Event::Finished(Ok(()), None)).unwrap();
        app.poll();
        assert!(!app.installed_apps.contains_key("plervo-pdf"));
        assert!(app.installed_apps.contains_key("plervo-cad"));
        app.tx.send(Event::Finished(Ok(()), Some(installed()))).unwrap();
        app.poll();
        assert_eq!(app.installed_apps.len(), 13);
        app.tx.send(Event::Scanned(BTreeMap::new())).unwrap();
        app.poll();
        assert!(app.installed_apps.is_empty());
        assert!(app.installed.is_none());
    }
    #[test]
    fn empty_library_offers_the_full_catalog_in_every_language() {
        for (language, empty, browse, manage) in [
            (Language::En, "No apps installed yet.", "Browse all apps", "Manage Plervo PDF"),
            (Language::Pt, "Nenhum app instalado ainda.", "Ver todos os apps", "Gerenciar Plervo PDF"),
            (Language::Es, "Todavía no hay apps instaladas.", "Ver todas las apps", "Gestionar Plervo PDF"),
        ] {
            let app = SuiteApp { language, ..Default::default() };
            let mut h = Harness::builder().with_size(egui::vec2(760.0, 580.0)).build_ui_state(|ui, app: &mut SuiteApp| app.show(ui), app);
            h.run();
            h.get_by_label(empty);
            h.get_by_label(browse).click();
            h.run();
            h.get_by_label(manage);
        }
    }
    #[test]
    fn support_is_visible_on_every_page_and_opens_only_after_click() {
        for (language, label) in [(Language::En, "Support"), (Language::Pt, "Apoiar"), (Language::Es, "Apoyar")] {
            for (settings, catalog, detail) in [(false, false, false), (false, false, true), (true, false, false), (false, true, false)] {
                let app = SuiteApp { language, settings, catalog, detail, ..Default::default() };
                let mut h = Harness::builder().with_size(egui::vec2(760.0, 580.0)).build_ui_state(|ui, app: &mut SuiteApp| app.show(ui), app);
                h.run_steps(2);
                assert!(h.output().platform_output.commands.is_empty());
                let button = h.get_by_role_and_label(egui::accesskit::Role::Button, label);
                assert!(button.rect().max.y < 100.0);
                button.click();
                let mut urls = Vec::new();
                for _ in 0..3 {
                    h.run_steps(1);
                    for command in &h.output().platform_output.commands {
                        if let egui::OutputCommand::OpenUrl(url) = command {
                            urls.push((url.url.clone(), url.new_tab));
                        }
                    }
                }
                assert_eq!(urls, [("https://buymeacoffee.com/alport".into(), true)]);
            }
        }
    }
    #[test]
    fn settings_offer_system_english_portuguese_and_spanish() {
        let mut h = harness();
        h.get_by_label("Settings").click();
        h.run();
        h.get_by_role(egui::accesskit::Role::ComboBox).click();
        h.run();
        h.get_by_label("Português").click();
        h.run();
        assert_eq!(h.state().language, Language::Pt);
        h.get_by_label("Meus apps");
        h.get_by_role(egui::accesskit::Role::ComboBox).click();
        h.run();
        h.get_by_label("Español").click();
        h.run();
        assert_eq!(h.state().language, Language::Es);
        h.get_by_label("Mis apps");
    }
    #[test]
    fn cancel_uninstall_does_not_start_a_worker_or_remove_anything() {
        let mut h = harness();
        h.state_mut().confirm_remove = true;
        h.run();
        h.get_by_label("Your documents and preferences will be kept.");
        h.get_by_label("Cancel").click();
        h.run();
        assert!(!h.state().confirm_remove);
        assert!(!h.state().busy);
    }
    #[test]
    fn unknown_commands_do_not_start_the_gui_or_an_installer() {
        assert!(headless(&["--install-pdf".into(), "https://evil.test".into()]).is_err());
        assert!(headless(&["--install".into(), "../other".into()]).is_err());
        assert!(headless(&["--open".into(), "https://evil.test".into()]).is_err());
    }
    #[test]
    fn catalog_selects_products_without_reusing_another_apps_release() -> Result<(), String> {
        use egui_kittest::kittest::NodeT;
        let mut h = harness();
        SuiteApp::style(&h.ctx);
        h.run();
        h.get_by_label("All apps").click();
        h.run();
        h.get_by_label("Plervo Photo");
        if let Some(path) = std::env::var_os("PLERVO_CATALOG_PREVIEW") {
            h.render()?.save(path).map_err(|e| e.to_string())?;
        }
        assert!(!h.state().busy);
        let release = serde_json::json!({"tag_name":"v0.1.0-alpha.2","draft":false,"assets":[]});
        h.state_mut().release = Some(plervo_suite_core::release::parse_github_release(&release.to_string(), PDF.repo).unwrap());
        h.get_by_label("Manage Plervo Photo").click();
        h.run();
        assert!(!h.state().catalog);
        assert_eq!(h.state().selected.id, "photo");
        assert!(h.state().release.is_none());
        assert!(h.get_by_label("Install Plervo Photo").accesskit_node().is_disabled());
        h.get_by_label("All apps").click();
        h.run();
        h.get_by_label("Manage Plervo PDF").click();
        h.run();
        assert_eq!(h.state().selected.id, "pdf");
        h.get_by_label("Install Plervo PDF");
        Ok(())
    }

    #[test]
    fn candidate_catalog_uses_the_selected_language_at_minimum_window_size() -> Result<(), String> {
        let app = SuiteApp { system_language: Language::Pt, catalog: true, ..Default::default() };
        let mut h = Harness::builder().with_size(egui::vec2(760.0, 580.0)).build_ui_state(|ui, app: &mut SuiteApp| app.show(ui), app);
        SuiteApp::style(&h.ctx);
        h.run();
        h.get_by_label("Suas ferramentas, juntas.");
        h.get_by_label("Gerenciar Plervo PDF");
        h.get_by_label("Gerenciar Plervo Photo");
        assert!(!h.state().busy);
        if let Some(path) = std::env::var_os("PLERVO_CATALOG_PREVIEW_SMALL") {
            h.render()?.save(path).map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}

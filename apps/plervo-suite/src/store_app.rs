//! The Store build has its own action surface: it cannot reach the external installer UI/CLI.
use crate::{INK, LIME, MUTED, PAPER, SuiteApp, family, locale::Language};
use egui::{RichText, Vec2};
use plervo_suite_native::store;
use std::{
    collections::BTreeMap,
    sync::mpsc,
    time::{Duration, Instant},
};

enum Action {
    Scan,
    Open(&'static str),
    Store(String),
}
type Inventory = BTreeMap<String, String>;
pub struct StoreApp {
    language: Language,
    system_language: Language,
    settings: bool,
    catalog: bool,
    installed: Inventory,
    icons: BTreeMap<&'static str, egui::TextureHandle>,
    busy: bool,
    error: Option<String>,
    next_scan: Instant,
    tx: mpsc::Sender<Result<Inventory, String>>,
    rx: mpsc::Receiver<Result<Inventory, String>>,
}
impl Default for StoreApp {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            language: Language::Auto,
            system_language: Language::Auto.resolve(),
            settings: false,
            catalog: false,
            installed: BTreeMap::new(),
            icons: BTreeMap::new(),
            busy: false,
            error: None,
            next_scan: Instant::now() + Duration::from_secs(30),
            tx,
            rx,
        }
    }
}
impl StoreApp {
    /// Called only by the explicit native screenshot probe; it never fabricates installed apps.
    pub fn set_probe_view(&mut self, language: &str, view: &str) {
        self.language = crate::locale::from_tags([language.to_owned()]);
        self.catalog = view == "catalog";
        self.settings = view == "settings";
    }
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        SuiteApp::style(&cc.egui_ctx);
        let mut app = Self::default();
        if let Some(saved) = cc.storage.and_then(|s| s.get_string("language")) {
            app.language = serde_json::from_str(&saved).unwrap_or_default();
        }
        app.start(Action::Scan, &cc.egui_ctx);
        app
    }
    pub fn is_busy(&self) -> bool {
        self.busy
    }
    fn start(&mut self, action: Action, ctx: &egui::Context) {
        if self.busy {
            return;
        }
        self.busy = true;
        self.error = None;
        self.next_scan = Instant::now() + Duration::from_secs(30);
        let tx = self.tx.clone();
        let ctx = ctx.clone();
        let spawned = std::thread::Builder::new().name("plervo-store".into()).spawn(move || {
            let result = std::panic::catch_unwind(|| {
                match action {
                    Action::Scan => {}
                    Action::Open(id) => store::open(id)?,
                    Action::Store(uri) => store::launch_uri(&uri)?,
                }
                store::scan()
            })
            .unwrap_or_else(|_| Err("Windows could not complete this action. Please try again.".into()));
            let _ = tx.send(result);
            ctx.request_repaint();
        });
        if let Err(error) = spawned {
            self.busy = false;
            self.error = Some(error.to_string());
        }
    }
    fn icon(&mut self, ui: &mut egui::Ui, product: &family::Product) {
        if let std::collections::btree_map::Entry::Vacant(entry) = self.icons.entry(product.id)
            && let Ok(icon) = eframe::icon_data::from_png_bytes(product.icon)
        {
            entry.insert(ui.ctx().load_texture(
                product.id,
                egui::ColorImage::from_rgba_unmultiplied([icon.width as usize, icon.height as usize], &icon.rgba),
                egui::TextureOptions::LINEAR,
            ));
        }
        if let Some(icon) = self.icons.get(product.id) {
            ui.image((icon.id(), Vec2::splat(48.0)));
        }
    }
    pub fn show(&mut self, root: &mut egui::Ui) {
        let ctx = root.ctx().clone();
        while let Ok(result) = self.rx.try_recv() {
            self.busy = false;
            match result {
                Ok(installed) => self.installed = installed,
                Err(error) => self.error = Some(error),
            }
        }
        if Instant::now() >= self.next_scan && !self.busy {
            self.start(Action::Scan, &ctx);
        }
        ctx.request_repaint_after(Duration::from_secs(1));
        let l = if self.language == Language::Auto { self.system_language } else { self.language };
        let t = |en, pt, es| l.text(en, pt, es);
        egui::Panel::left("store-navigation").exact_size(190.0).resizable(false).frame(egui::Frame::new().fill(INK).inner_margin(24)).show(
            root,
            |ui| {
                ui.visuals_mut().override_text_color = Some(egui::Color32::WHITE);
                ui.add_space(15.0);
                ui.label(RichText::new("Plervo").size(32.0).strong().color(egui::Color32::WHITE));
                ui.label(RichText::new("S U I T E").size(11.0).color(LIME));
                ui.add_space(58.0);
                for (label, tab) in [
                    (t("My apps", "Meus apps", "Mis apps"), 0),
                    (t("All apps", "Todos os apps", "Todas las apps"), 1),
                    (t("Settings", "Configurações", "Configuración"), 2),
                ] {
                    let selected = match tab {
                        0 => !self.catalog && !self.settings,
                        1 => self.catalog,
                        _ => self.settings,
                    };
                    if ui.selectable_label(selected, RichText::new(label).color(if selected { INK } else { egui::Color32::WHITE })).clicked() {
                        self.catalog = tab == 1;
                        self.settings = tab == 2;
                        if tab == 0 {
                            self.start(Action::Scan, &ctx);
                        }
                    }
                    ui.add_space(8.0);
                }
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.label(RichText::new("Microsoft Store · Preview").size(11.0).color(LIME));
                });
            },
        );
        egui::CentralPanel::default().frame(egui::Frame::new().fill(PAPER).inner_margin(30)).show(root, |ui| {
            ui.horizontal(|ui| {
                if ui.add_enabled(!self.busy, egui::Button::new(t("Store updates", "Atualizações da Store", "Actualizaciones de Store"))).clicked()
                {
                    self.start(Action::Store(store::UPDATES_URI.into()), &ctx);
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.hyperlink_to(t("Support", "Apoiar", "Apoyar"), "https://buymeacoffee.com/alport");
                });
            });
            ui.add_space(16.0);
            if self.busy {
                ui.spinner();
            }
            if let Some(error) = &self.error {
                ui.label(t(
                    "This action could not be completed. You can retry below.",
                    "Não foi possível concluir. Você pode tentar novamente abaixo.",
                    "No se pudo completar la acción. Puedes volver a intentarlo abajo.",
                ));
                ui.label(RichText::new(error).small());
            }
            egui::ScrollArea::vertical().id_salt("store-content").show(ui, |ui| {
                if self.settings {
                    ui.heading(t("Make it yours.", "Do seu jeito.", "A tu manera."));
                    ui.add_space(20.0);
                    egui::ComboBox::from_label(t("Language", "Idioma", "Idioma"))
                        .selected_text(match self.language {
                            Language::Auto => t("System default", "Padrão do sistema", "Predeterminado del sistema"),
                            Language::En => "English",
                            Language::Pt => "Português",
                            Language::Es => "Español",
                        })
                        .show_ui(ui, |ui| {
                            for (value, label) in [
                                (Language::Auto, t("System default", "Padrão do sistema", "Predeterminado del sistema")),
                                (Language::En, "English"),
                                (Language::Pt, "Português"),
                                (Language::Es, "Español"),
                            ] {
                                ui.selectable_value(&mut self.language, value, label);
                            }
                        });
                    ui.add_space(20.0);
                    ui.label(t(
                        "Microsoft Store manages installation and updates for this edition and its Store apps.",
                        "A Microsoft Store gerencia a instalação e as atualizações desta edição e de seus apps da Store.",
                        "Microsoft Store gestiona la instalación y las actualizaciones de esta edición y sus apps de Store.",
                    ));
                    if ui
                        .add_enabled(
                            !self.busy,
                            egui::Button::new(t("Manage apps in Windows", "Gerenciar apps no Windows", "Gestionar apps en Windows")),
                        )
                        .clicked()
                    {
                        self.start(Action::Store(store::SETTINGS_URI.into()), &ctx);
                    }
                    ui.add_space(20.0);
                    ui.label(t(
                        "Published by Ricardo Bonetti · Plervo / AlporT",
                        "Publicado por Ricardo Bonetti · Plervo / AlporT",
                        "Publicado por Ricardo Bonetti · Plervo / AlporT",
                    ));
                    ui.hyperlink_to(t("Privacy", "Privacidade", "Privacidad"), "https://plervo.com/privacy/");
                    ui.hyperlink_to(t("Source & licenses", "Código e licenças", "Código y licencias"), "https://github.com/alportblu/plervo-pdf");
                    ui.label(
                        RichText::new("Plervo Suite · MIT / Apache-2.0 · ArtCraft Launcher attribution retained in NOTICE").small().color(MUTED),
                    );
                    return;
                }
                ui.heading(
                    RichText::new(if self.catalog {
                        t("Your tools, together.", "Suas ferramentas, juntas.", "Tus herramientas, juntas.")
                    } else {
                        t("Your installed apps.", "Seus apps instalados.", "Tus apps instaladas.")
                    })
                    .size(30.0),
                );
                ui.label(t(
                    "Your Plervo apps from Microsoft Store. Installations and updates stay with the Store.",
                    "Seus apps Plervo da Microsoft Store. Instalações e atualizações ficam com a Store.",
                    "Tus apps Plervo de Microsoft Store. La Store gestiona instalaciones y actualizaciones.",
                ));
                if ui.add_enabled(!self.busy, egui::Button::new(t("Refresh apps", "Atualizar lista", "Actualizar lista"))).clicked() {
                    self.start(Action::Scan, &ctx);
                }
                ui.add_space(16.0);
                let products: Vec<_> = family::PRODUCTS
                    .iter()
                    .filter(|p| self.installed.contains_key(p.id) || (self.catalog && store::acquisition_uri(p.id).is_ok()))
                    .collect();
                if products.is_empty() && !self.busy {
                    ui.label(t(
                        "No Plervo apps from Microsoft Store are installed yet.",
                        "Nenhum app Plervo da Microsoft Store está instalado ainda.",
                        "Aún no hay apps Plervo de Microsoft Store instaladas.",
                    ));
                    if ui.button(t("Browse all apps", "Ver todos os apps", "Ver todas las apps")).clicked() {
                        self.catalog = true;
                    }
                }
                let columns = if ui.available_width() >= 690.0 {
                    3
                } else if ui.available_width() >= 440.0 {
                    2
                } else {
                    1
                };
                for row in products.chunks(columns) {
                    ui.columns(columns, |columns| {
                        for (product, ui) in row.iter().zip(columns) {
                            egui::Frame::new().fill(egui::Color32::WHITE).inner_margin(16).corner_radius(12).show(ui, |ui| {
                                ui.set_min_width(ui.available_width());
                                ui.set_min_height(200.0);
                                self.icon(ui, product);
                                ui.label(RichText::new(product.name).strong().size(19.0));
                                ui.label(t(product.description[0], product.description[1], product.description[2]));
                                if let Some(version) = self.installed.get(product.id) {
                                    ui.label(RichText::new(version).small().color(MUTED));
                                    if ui
                                        .add_enabled(
                                            !self.busy,
                                            egui::Button::new(format!("{} {}", t("Open", "Abrir", "Abrir"), product.name)).fill(LIME),
                                        )
                                        .clicked()
                                    {
                                        self.start(Action::Open(product.id), &ctx);
                                    }
                                }
                                if let Ok(uri) = store::acquisition_uri(product.id) {
                                    if ui
                                        .add_enabled(
                                            !self.busy,
                                            egui::Button::new(t("View in Microsoft Store", "Ver na Microsoft Store", "Ver en Microsoft Store")),
                                        )
                                        .clicked()
                                    {
                                        self.start(Action::Store(uri), &ctx);
                                    }
                                } else {
                                    ui.label(
                                        RichText::new(t(
                                            "Not available in Microsoft Store yet.",
                                            "Ainda não disponível na Microsoft Store.",
                                            "Aún no disponible en Microsoft Store.",
                                        ))
                                        .small()
                                        .color(MUTED),
                                    );
                                }
                            });
                        }
                    });
                    ui.add_space(12.0);
                }
            });
        });
    }
}
impl eframe::App for StoreApp {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        self.show(ui);
    }
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        if let Ok(language) = serde_json::to_string(&self.language) {
            storage.set_string("language", language);
        }
    }
}
pub fn headless(args: &[String]) -> Result<(), String> {
    match args.iter().map(String::as_str).collect::<Vec<_>>().as_slice() {
        ["--status"] => { println!("{}", serde_json::to_string_pretty(&store::scan()?).map_err(|e| e.to_string())?); Ok(()) },
        ["--store-link", id] => { println!("{}", store::acquisition_uri(id)?); Ok(()) },
        ["--install", id] => store::launch_uri(&store::acquisition_uri(id)?),
        ["--open", id] => store::open(id),
        ["--check-updates"] => store::launch_uri(store::UPDATES_URI),
        ["--manage"] => store::launch_uri(store::SETTINGS_URI),
        _ => Err("Store edition: --status | --store-link PRODUCT | --install PRODUCT | --open PRODUCT | --check-updates | --manage. Installation and removal require confirmation in Microsoft Store / Windows Settings.".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::{Harness, kittest::Queryable};
    #[test]
    fn all_installed_store_apps_appear_and_unsupported_actions_are_absent() {
        let mut app = StoreApp { system_language: Language::En, ..Default::default() };
        for id in ["pdf", "video", "cad"] {
            app.installed.insert(id.into(), "1.0.7.0".into());
        }
        let mut h = Harness::builder().with_size(Vec2::new(1000.0, 700.0)).build_ui_state(|ui, app| app.show(ui), app);
        h.run_steps(2);
        for label in ["Open Plervo PDF", "Open Plervo Video", "Open Plervo CAD"] {
            assert!(h.query_by_label(label).is_some(), "{label}");
        }
        assert!(h.query_by_label("Install Plervo Photo").is_none());
        assert!(h.query_by_label("Download Suite update").is_none());
        assert!(headless(&["--uninstall".into(), "pdf".into()]).is_err());
    }
    #[test]
    fn store_settings_have_three_languages_and_no_github_update_toggle() {
        for (language, label) in [(Language::En, "Make it yours."), (Language::Pt, "Do seu jeito."), (Language::Es, "A tu manera.")] {
            let app = StoreApp { settings: true, language, ..Default::default() };
            let mut h = Harness::builder().with_size(Vec2::new(1000.0, 700.0)).build_ui_state(|ui, app| app.show(ui), app);
            h.run_steps(2);
            assert!(h.query_by_label(label).is_some());
            assert!(h.query_by_label("Check for updates automatically").is_none());
        }
    }
}

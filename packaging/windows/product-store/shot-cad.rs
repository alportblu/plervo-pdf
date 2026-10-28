//! Render the real CAD UI without a desktop window; this helper is never packaged.
use cadcraft_engine::Session;
use cadcraft_ui_egui::{CadApp, Services, i18n::Preference};
use egui_kittest::Harness;

fn main() -> Result<(), String> {
    let path = std::env::args().nth(1).ok_or("usage: plervo_store_shot <output.png> <language>")?;
    let language = std::env::args().nth(2).ok_or("missing language")?;
    let language = if language == "pt-br" { "pt-BR" } else { &language };
    let mut app = CadApp::new(Session::default(), Services::default());
    app.ui.interface_language = Preference::parse(language).ok_or("unsupported language")?;
    app.run("ui.sample", serde_json::json!({}))?;
    let mut h =
        Harness::builder().with_size(egui::vec2(1600.0, 1000.0)).with_pixels_per_point(1.0).with_theme(egui::Theme::Dark).wgpu().build_ui_state(
            |ui, app: &mut CadApp| {
                app.logic(&ui.ctx().clone());
                app.ui(ui);
            },
            app,
        );
    h.run_steps(10);
    let image = h.render()?;
    if image.dimensions() != (1600, 1000) {
        return Err("unexpected listing render dimensions".into());
    }
    image.save(path).map_err(|error| error.to_string())
}

//! Render the real Video UI without a desktop window; this helper is never packaged.
use egui_kittest::Harness;
use filmcraft_engine::Session;
use filmcraft_ui_egui::{FilmcraftApp, i18n::Language};

fn main() -> Result<(), String> {
    let path = std::env::args().nth(1).ok_or("usage: plervo_store_shot <output.png> <language>")?;
    let language = std::env::args().nth(2).ok_or("missing language")?;
    let mut session = Session::default();
    session.execute("file.openDemoProject", serde_json::json!({})).map_err(|error| error.to_string())?;
    let mut app = FilmcraftApp::new(session);
    app.ui.language = Language::parse(&language).ok_or("unsupported language")?;
    let mut h = Harness::builder()
        .with_size(egui::vec2(1600.0, 1000.0))
        .with_pixels_per_point(1.0)
        .with_theme(egui::Theme::Dark)
        .wgpu()
        .build_eframe(move |_cc| app);
    // Allow the real asynchronous preview/thumbnail workers to finish their first frames.
    for _ in 0..180 {
        h.run_steps(2);
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
    let image = h.render()?;
    if image.dimensions() != (1600, 1000) {
        return Err("unexpected listing render dimensions".into());
    }
    image.save(path).map_err(|error| error.to_string())
}

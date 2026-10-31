//! Render the actual app UI for Store listing review; never bundled in the app.
use deckcraft_engine::Session;
use deckcraft_ui_egui::{Services, SlideApp};
use egui_kittest::Harness;

fn main() -> Result<(), String> {
    let path = std::env::args().nth(1).ok_or("missing output path")?;
    let mut session = Session::with_new();
    deckcraft_engine::sample::open_sample(&mut session).map_err(|e| e.to_string())?;
    let app = SlideApp::new(session, Services::default());
    let mut harness = Harness::builder().with_size(egui::vec2(1600.0, 1000.0)).with_pixels_per_point(1.0).with_max_steps(128).wgpu().build_ui_state(
        |ui, app: &mut SlideApp| {
            app.logic(&ui.ctx().clone());
            app.ui(ui);
        },
        app,
    );
    harness.run_steps(20);
    let image = harness.render()?;
    if image.dimensions() != (1600, 1000) {
        return Err("unexpected listing dimensions".into());
    }
    image.save(path).map_err(|e| e.to_string())
}

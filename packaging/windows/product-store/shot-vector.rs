//! Render the actual app UI for Store listing review; never bundled in the app.
use egui_kittest::Harness;
use vectorcraft_engine::Session;
use vectorcraft_ui_egui::{Services, VectorcraftApp};

fn main() -> Result<(), String> {
    let path = std::env::args().nth(1).ok_or("missing output path")?;
    let app = VectorcraftApp::new(Session::new(), Services::default());
    let mut harness = Harness::builder().with_size(egui::vec2(1600.0, 1000.0)).with_pixels_per_point(1.0).with_max_steps(128).wgpu().build_ui_state(
        |ui, app: &mut VectorcraftApp| {
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

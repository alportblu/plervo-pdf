//! Explicit native rendering check for disposable acceptance environments. No listener.
use std::path::PathBuf;
use std::time::{Duration, Instant};

pub struct Probe {
    app: plervo_suite::SuiteApp,
    output: PathBuf,
    start: Instant,
    requested: bool,
    success: std::sync::Arc<std::sync::atomic::AtomicBool>,
}
impl Probe {
    pub fn new(cc: &eframe::CreationContext<'_>, output: PathBuf, success: std::sync::Arc<std::sync::atomic::AtomicBool>) -> Self {
        Self { app: plervo_suite::SuiteApp::new(cc), output, start: Instant::now(), requested: false, success }
    }
}
impl eframe::App for Probe {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        eframe::App::ui(&mut self.app, ui, frame);
        let ctx = ui.ctx();
        for event in ctx.input(|i| i.events.clone()) {
            if let egui::Event::Screenshot { image, .. } = event {
                if let Err(e) = save(&self.output, &image) {
                    eprintln!("Native screenshot failed: {e}");
                } else {
                    self.success.store(true, std::sync::atomic::Ordering::Relaxed);
                }
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
        if !self.requested && self.start.elapsed() >= Duration::from_secs(2) && !self.app.is_busy() {
            self.requested = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
        }
        if self.start.elapsed() > Duration::from_secs(45) {
            eprintln!("Native rendering check timed out");
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        ctx.request_repaint_after(Duration::from_millis(100));
    }
}
fn save(path: &std::path::Path, image: &egui::ColorImage) -> Result<(), String> {
    let width = u32::try_from(image.width()).map_err(|e| e.to_string())?;
    let height = u32::try_from(image.height()).map_err(|e| e.to_string())?;
    let file = std::fs::OpenOptions::new().write(true).create_new(true).open(path).map_err(|e| e.to_string())?;
    let mut encoder = png::Encoder::new(file, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
    let pixels: Vec<u8> = image.pixels.iter().flat_map(|c| c.to_array()).collect();
    writer.write_image_data(&pixels).map_err(|e| e.to_string())?;
    writer.finish().map_err(|e| e.to_string())
}

//! Capture the real Canvas renderer on a disposable CI desktop. Never packaged.
use figcraft_engine::Session;
use figcraft_ui_egui::{FigApp, NoServices};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

struct Capture {
    app: FigApp,
    path: String,
    started: Instant,
    requested: bool,
    result: Arc<Mutex<Option<Result<(), String>>>>,
}
impl eframe::App for Capture {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // FigApp::ui initializes fonts and defers drawing for their first frame, as in the app.
        // Calling FigApp::logic here first bypasses that guard and draws an unbound font family.
        let image = ctx.input(|input| {
            input.events.iter().find_map(|event| if let egui::Event::Screenshot { image, .. } = event { Some(image.clone()) } else { None })
        });
        let result = if let Some(image) = image {
            Some((|| {
                let [width, height] = image.size;
                if width < 1366 || height < 768 {
                    return Err("native render is below Store minimum dimensions".into());
                }
                let width = u32::try_from(width).map_err(|e| e.to_string())?;
                let height = u32::try_from(height).map_err(|e| e.to_string())?;
                let pixels: Vec<u8> = image.pixels.iter().flat_map(|p| p.to_array()).collect();
                image::RgbaImage::from_raw(width, height, pixels).ok_or("invalid renderer output")?.save(&self.path).map_err(|e| e.to_string())
            })())
        } else if self.started.elapsed() > Duration::from_secs(60) {
            Some(Err("renderer capture timed out".into()))
        } else {
            None
        };
        if let Some(result) = result {
            *self.result.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = Some(result);
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        } else if !self.requested && self.started.elapsed() > Duration::from_secs(3) {
            self.requested = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
        }
        ctx.request_repaint_after(Duration::from_millis(30));
    }
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.app.ui(ui);
    }
}
fn main() -> Result<(), String> {
    if std::env::var("GITHUB_ACTIONS").as_deref() != Ok("true") || std::env::var("RUNNER_OS").as_deref() != Ok("Windows") {
        return Err("this capture requires a disposable Windows CI desktop".into());
    }
    let path = std::env::args().nth(1).ok_or("missing output path")?;
    let result = Arc::new(Mutex::new(None));
    let app_result = result.clone();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1600.0, 1000.0]).with_min_inner_size([1600.0, 1000.0]).with_decorations(false),
        ..Default::default()
    };
    eframe::run_native(
        "Plervo Canvas listing capture",
        options,
        Box::new(move |_| {
            Ok(Box::new(Capture {
                app: {
                    let mut app = FigApp::new(Session::new(), Box::new(NoServices));
                    app.ui.home = false;
                    app.ui.title_bar = figcraft_ui_egui::TitleBar::Custom;
                    app
                },
                path,
                started: Instant::now(),
                requested: false,
                result: app_result,
            }))
        }),
    )
    .map_err(|e| e.to_string())?;
    result.lock().unwrap_or_else(std::sync::PoisonError::into_inner).take().ok_or("capture closed without an image")?
}

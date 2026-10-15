//! Capture the actual Suite UI without opening a desktop window or using global input.
use egui_kittest::Harness;
use std::time::{Duration, Instant};
fn main() -> Result<(), String> {
    let path = std::env::args().nth(1).ok_or("usage: shot <output.png>")?;
    let mut h = Harness::builder().with_size(egui::vec2(1000.0, 700.0)).with_pixels_per_point(2.0).build_eframe(|cc| plervo_suite::SuiteApp::new(cc));
    let start = Instant::now();
    loop {
        h.run_steps(2);
        if !h.state().is_busy() || start.elapsed() > Duration::from_secs(40) {
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    h.run_steps(4);
    h.render()?.save(path).map_err(|e| e.to_string())
}

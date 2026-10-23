#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod probe;

#[cfg(feature = "microsoft-store")]
use plervo_suite::{StoreApp as App, store_headless as headless};
#[cfg(not(feature = "microsoft-store"))]
use plervo_suite::{SuiteApp as App, headless};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().is_some_and(|s| s == "--version") {
        println!("Plervo Suite {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    let probe_path =
        if args.len() == 2 && args.first().is_some_and(|s| s == "--native-smoke-test") { args.get(1).map(std::path::PathBuf::from) } else { None };
    if probe_path.as_ref().is_some_and(|p| p.exists()) {
        eprintln!("The screenshot destination already exists.");
        std::process::exit(1);
    }
    if !args.is_empty() && probe_path.is_none() {
        if let Err(error) = headless(&args) {
            eprintln!("{error}");
            std::process::exit(1);
        }
        return;
    }
    // Windows' stock OpenGL driver is 1.1, so use wgpu's native graphics backend there.
    #[cfg(windows)]
    let renderer = eframe::Renderer::Wgpu;
    #[cfg(not(windows))]
    let renderer = eframe::Renderer::Glow;
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_app_id("com.plervo.suite")
            .with_title("Plervo Suite")
            .with_icon(eframe::icon_data::from_png_bytes(include_bytes!("../../../assets/products/suite/icon-256.png")).unwrap_or_default())
            // Store desktop screenshots require at least 1366x768 physical pixels.
            .with_inner_size(if probe_path.is_some() { [1440.0, 900.0] } else { [1000.0, 700.0] })
            .with_min_inner_size([760.0, 580.0]),
        renderer,
        ..Default::default()
    };
    let capture = probe_path.clone();
    let success = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let recorded = success.clone();
    if let Err(error) = eframe::run_native(
        "Plervo Suite",
        options,
        Box::new(move |cc| if let Some(path) = capture { Ok(Box::new(probe::Probe::new(cc, path, recorded))) } else { Ok(Box::new(App::new(cc))) }),
    ) {
        eprintln!("{error}");
        std::process::exit(1);
    }
    if probe_path.is_some() && !success.load(std::sync::atomic::Ordering::Relaxed) {
        std::process::exit(1);
    }
}

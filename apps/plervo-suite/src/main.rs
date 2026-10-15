#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().is_some_and(|s| s == "--version") {
        println!("Plervo Suite {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    if !args.is_empty() {
        if let Err(error) = plervo_suite::headless(&args) {
            eprintln!("{error}");
            std::process::exit(1);
        }
        return;
    }
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_app_id("com.plervo.suite")
            .with_title("Plervo Suite")
            .with_inner_size([1000.0, 700.0])
            .with_min_inner_size([760.0, 580.0]),
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    if let Err(error) = eframe::run_native("Plervo Suite", options, Box::new(|cc| Ok(Box::new(plervo_suite::SuiteApp::new(cc))))) {
        eprintln!("{error}");
    }
}

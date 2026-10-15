//! Exercise the release installer with a verified cached candidate in a disposable runner.
#[cfg(target_os = "linux")]
fn main() -> Result<(), String> {
    use plervo_suite_core::{Asset, Release};
    use plervo_suite_native::{
        http::{self, Stop},
        install, manager,
        paths::Paths,
    };
    if std::env::var("PLERVO_ACCEPTANCE").as_deref() != Ok("1") {
        return Err("Only for disposable acceptance environments (PLERVO_ACCEPTANCE=1).".into());
    }
    let package = std::env::args().nth(1).ok_or("Expected a PDF candidate archive")?;
    let package = std::path::Path::new(&package);
    let name = package.file_name().and_then(|n| n.to_str()).ok_or("Invalid filename")?;
    let version = env!("CARGO_PKG_VERSION");
    let tag = format!("v{version}");
    let asset = Asset {
        name: name.into(),
        size: package.metadata().map_err(|e| e.to_string())?.len(),
        url: format!("https://github.com/alportblu/plervo-pdf/releases/download/{tag}/{name}"),
        sha256: Some(http::sha256_file(package)?),
    };
    let release = Release { tag, version: plervo_suite_core::version::parse(version), assets: vec![asset.clone()], ..Release::default() };
    let paths = Paths::new()?;
    std::fs::create_dir_all(paths.downloads()).map_err(|e| e.to_string())?;
    std::fs::copy(package, install::download_path(&paths, &asset)).map_err(|e| e.to_string())?;
    manager::install_pdf(&release, &paths, &Stop::default(), &mut |phase| eprintln!("{phase:?}"))?;
    let found = manager::detected().ok_or("Installed PDF not detected")?;
    if found.version != release.version {
        return Err("Installed version mismatch".into());
    }
    println!("{}", found.location);
    Ok(())
}
#[cfg(not(target_os = "linux"))]
fn main() {}

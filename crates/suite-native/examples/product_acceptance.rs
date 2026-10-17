//! Check the real Mac bundle identity or install the real candidate through Suite on Linux.
//! This example is a validation tool and is never included in public app packages.

#[cfg(target_os = "linux")]
fn main() -> Result<(), String> {
    use plervo_suite_core::{Asset, Release, catalog::SUITE};
    use plervo_suite_native::{
        http::{self, Stop},
        install, managed_linux, manager,
        paths::Paths,
    };
    use std::path::PathBuf;
    use std::process::Command;

    if std::env::var("PLERVO_ACCEPTANCE").as_deref() != Ok("1") {
        return Err("Only for disposable Linux acceptance environments (PLERVO_ACCEPTANCE=1).".into());
    }
    let product = std::env::args().nth(1).ok_or("Expected product ID")?;
    let id = format!("plervo-{product}");
    let app = SUITE.iter().find(|app| app.id == id).ok_or("Unknown product")?;
    let package = PathBuf::from(std::env::args_os().nth(2).ok_or("Expected candidate archive")?);
    let name = package.file_name().and_then(|name| name.to_str()).ok_or("Invalid archive filename")?;
    let version = env!("CARGO_PKG_VERSION");
    let tag = format!("v{version}");
    let asset = Asset {
        name: name.into(),
        size: package.metadata().map_err(|e| e.to_string())?.len(),
        url: format!("https://github.com/alportblu/plervo-pdf/releases/download/{tag}/{name}"),
        sha256: Some(http::sha256_file(&package)?),
    };
    let release = Release { tag, version: plervo_suite_core::version::parse(version), assets: vec![asset.clone()], ..Release::default() };
    let temp = tempfile::tempdir().map_err(|e| e.to_string())?;
    let paths = Paths::under(temp.path());
    let document = temp.path().join("document-preserved.txt");
    let preferences = paths.data.join("preferences-preserved.txt");
    std::fs::create_dir_all(&paths.data).map_err(|e| e.to_string())?;
    std::fs::write(&document, b"original document").map_err(|e| e.to_string())?;
    std::fs::write(&preferences, b"original preferences").map_err(|e| e.to_string())?;

    for operation in ["install", "reinstall"] {
        std::fs::create_dir_all(paths.downloads()).map_err(|e| e.to_string())?;
        std::fs::copy(&package, install::download_path(&paths, &asset)).map_err(|e| e.to_string())?;
        manager::install_app(app, &release, &paths, &Stop::default(), &mut |phase| eprintln!("{operation}: {phase:?}"))?;
        let found = managed_linux::detected_for(app, &paths)?.ok_or("Installed app not detected")?;
        if found.version != release.version {
            return Err("Installed version mismatch".into());
        }
        let binary = PathBuf::from(found.location);
        let cli = binary.parent().ok_or("Missing binary directory")?.join(format!("{}-cli", app.id));
        for executable in [&binary, &cli] {
            let result = Command::new(executable).arg("--version").output().map_err(|e| e.to_string())?;
            let output = String::from_utf8_lossy(&result.stdout);
            if !result.status.success() || !output.contains(version) {
                return Err(format!("{} --version failed: {output}", executable.display()));
            }
        }
        let result = Command::new(&cli).arg("--help").output().map_err(|e| e.to_string())?;
        if !result.status.success() || result.stdout.is_empty() {
            return Err("Installed CLI help failed".into());
        }
        println!("{} {operation}: passed", app.name);
    }
    manager::uninstall_app(app, &paths, &Stop::default(), &mut |phase| eprintln!("{phase:?}"))?;
    if managed_linux::detected_for(app, &paths)?.is_some()
        || std::fs::read(&document).map_err(|e| e.to_string())? != b"original document"
        || std::fs::read(&preferences).map_err(|e| e.to_string())? != b"original preferences"
    {
        return Err("Removal did not preserve documents and preferences".into());
    }
    println!("{}: Suite install, installed executables, reinstall and removal passed; documents/preferences preserved.", app.name);
    Ok(())
}

#[cfg(target_os = "macos")]
fn main() -> Result<(), String> {
    use plervo_suite_core::{catalog::SUITE, version};
    use plervo_suite_native::detect;
    use std::path::PathBuf;

    if std::env::var("PLERVO_ACCEPTANCE").as_deref() != Ok("1") {
        return Err("Only for explicit bundle acceptance (PLERVO_ACCEPTANCE=1).".into());
    }
    let product = std::env::args().nth(1).ok_or("Expected product ID")?;
    let id = format!("plervo-{product}");
    let app = SUITE.iter().find(|app| app.id == id).ok_or("Unknown product")?;
    let bundle = PathBuf::from(std::env::args_os().nth(2).ok_or("Expected candidate bundle")?);
    let found = detect::macos::read_bundle(app, &bundle).ok_or("Suite did not recognize the candidate bundle")?;
    let expected = version::parse(env!("CARGO_PKG_VERSION")).ok_or("Invalid source version")?;
    if found.version.as_ref() != Some(&expected) {
        return Err(format!("Suite detected {:?}, expected {expected:?}", found.version));
    }
    println!("{}: Suite recognized the bundle and exact release version {}.", app.name, env!("CARGO_PKG_VERSION"));
    Ok(())
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn main() -> Result<(), String> {
    Err("This acceptance helper requires Linux or macOS".into())
}

//! The GUI and CLI share this deliberately small, Plervo-only orchestration layer.
#[cfg(target_os = "macos")]
use std::path::Path;

use plervo_suite_core::catalog::PDF;
use plervo_suite_core::platform::{Arch, Os, Platform, pick_asset};
use plervo_suite_core::protocol::JobPhase;
use plervo_suite_core::{Installed, PackageKind, Release};

use crate::http::{Fetched, Http, Stop};
use crate::install::{self, Job};
#[cfg(not(target_os = "linux"))]
use crate::manifest::Manifest;
use crate::paths::Paths;

/// Platforms with a qualified installer. Windows remains gated on publisher signing.
pub fn supported_platform() -> Option<Platform> {
    if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        Some(Platform { os: Os::MacOs, arch: Arch::Arm64 })
    } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        Some(Platform { os: Os::Linux, arch: Arch::X64 })
    } else {
        None
    }
}

pub fn latest() -> Result<Release, String> {
    let http = Http::new()?;
    match http.get_api("https://api.github.com/repos/alportblu/plervo-pdf/releases?per_page=20", None)? {
        Fetched::Body { text, .. } => select_release(&text),
        Fetched::RateLimited { .. } => Err("GitHub's request limit was reached. Please try again later.".into()),
        _ => Err("No published release is available yet.".into()),
    }
}

/// A Suite-only release must never hide a newer usable PDF release. Ignore drafts and entries
/// without a PDF package; all asset URLs still pass the upstream repository confinement parser.
pub fn select_release(json: &str) -> Result<Release, String> {
    select_release_for(json, supported_platform().unwrap_or(Platform { os: Os::MacOs, arch: Arch::Arm64 }))
}

fn kinds(platform: Platform) -> &'static [PackageKind] {
    if platform.os == Os::Linux { &[PackageKind::TarGz] } else { &[PackageKind::Dmg] }
}

fn select_release_for(json: &str, platform: Platform) -> Result<Release, String> {
    let values: serde_json::Value = serde_json::from_str(json).map_err(|_| "Unreadable release catalog")?;
    let list = values.as_array().ok_or("Invalid release catalog")?;
    let mut best: Option<Release> = None;
    for value in list.iter().take(100) {
        if value["draft"].as_bool().unwrap_or(false) {
            continue;
        }
        let Ok(text) = serde_json::to_string(value) else { continue };
        let Ok(release) = plervo_suite_core::release::parse_github_release(&text, PDF.repo) else { continue };
        let Some(version) = release.version.as_ref() else { continue };
        if pick_asset(&release, &[PDF.id], platform, kinds(platform)).is_none() {
            continue;
        }
        if best.as_ref().and_then(|r| r.version.as_ref()).is_none_or(|old| version > old) {
            best = Some(release);
        }
    }
    best.ok_or_else(|| "No published PDF release is available yet.".into())
}

pub fn detected() -> Option<Installed> {
    #[cfg(target_os = "linux")]
    {
        Paths::new().ok().and_then(|paths| crate::managed_linux::detected(&paths).ok().flatten())
    }
    #[cfg(not(target_os = "linux"))]
    crate::detect::detect(&PDF, &Manifest::default(), &crate::detect::Snapshot::take())
}

/// No path provided by the GUI, a manifest, or a release feed can redirect an operation.
#[cfg(not(target_os = "linux"))]
fn verify_installed(found: &Installed) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let path = Path::new(&found.location);
        if !crate::detect::macos::candidates(&PDF).contains(&path.to_path_buf())
            || path.symlink_metadata().map_err(|e| e.to_string())?.file_type().is_symlink()
            || !install::macos::signed_by(path, install::macos::TEAM_ID)
        {
            return Err("This installation could not be verified as Plervo PDF. No files were changed.".into());
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = found;
        return Err("App management is not released on this platform yet.".into());
    }
    #[allow(unreachable_code)]
    Ok(())
}

pub fn install_pdf(release: &Release, paths: &Paths, stop: &Stop, phase: &mut dyn FnMut(JobPhase)) -> Result<(), String> {
    let platform = supported_platform().ok_or("App installation is not released on this platform yet.")?;
    let (asset, kind) = pick_asset(release, &[PDF.id], platform, kinds(platform)).ok_or("No compatible PDF download in this release.")?;
    validate_download(release, asset)?;
    #[cfg(target_os = "linux")]
    {
        let existing = crate::managed_linux::detected(paths)?;
        if let Some(found) = existing {
            refuse_downgrade(found.version.as_ref(), release.version.as_ref())?;
        }
        let file = install::download(release, asset, &mut Job { http: &Http::new()?, paths, stop, phase })?;
        (phase)(JobPhase::Installing);
        crate::managed_linux::install_archive(&file, &release.version_label(), paths)?;
        let _ = std::fs::remove_file(file);
        let _ = kind;
        Ok(())
    }
    #[cfg(not(target_os = "linux"))]
    {
        let existing = detected();
        if let Some(found) = &existing {
            verify_installed(found)?;
            refuse_downgrade(found.version.as_ref(), release.version.as_ref())?;
        }
        install::install(&PDF, release, asset, kind, existing.as_ref(), None, &mut Job { http: &Http::new()?, paths, stop, phase })?;
        let found = detected().ok_or("The installed app could not be located.")?;
        verify_installed(&found)
    }
}

fn refuse_downgrade(
    installed: Option<&plervo_suite_core::version::Version>,
    offered: Option<&plervo_suite_core::version::Version>,
) -> Result<(), String> {
    if installed.zip(offered).is_some_and(|(old, new)| new < old) {
        return Err("A newer version is already installed. It was left unchanged.".into());
    }
    Ok(())
}

fn validate_download(release: &Release, asset: &plervo_suite_core::Asset) -> Result<(), String> {
    let expected = format!("https://github.com/{}/releases/download/{}/{}", PDF.repo, release.tag, asset.name);
    let named_version =
        plervo_suite_core::platform::parse_asset_name(&asset.name, &[PDF.id]).and_then(|name| plervo_suite_core::version::parse(&name.version));
    if named_version.is_none()
        || named_version != release.version
        || !plervo_suite_core::release::valid_file_name(&asset.name)
        || asset.url != expected
        || release.tag.contains(['/', '\\', '?', '#', '%'])
        || !release.tag.starts_with('v')
        || asset.size == 0
        || asset.size > 512 * 1024 * 1024
        || asset.sha256.as_ref().is_none_or(|s| s.len() != 64 || !s.bytes().all(|b| b.is_ascii_hexdigit()))
    {
        return Err("The release has an untrusted download, checksum, or size. Nothing was installed.".into());
    }
    Ok(())
}

pub fn open_pdf(paths: &Paths) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        crate::managed_linux::open(paths)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let found = detected().ok_or("Plervo PDF is not installed.")?;
        verify_installed(&found)?;
        install::launch(&PDF, &found, paths)
    }
}

pub fn uninstall_pdf(paths: &Paths, stop: &Stop, phase: &mut dyn FnMut(JobPhase)) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        stop.check()?;
        (phase)(JobPhase::Removing);
        crate::managed_linux::uninstall(paths)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let found = detected().ok_or("Plervo PDF is not installed.")?;
        verify_installed(&found)?;
        // Native macOS removal moves just the application to Trash. Documents and settings stay put.
        install::uninstall(&PDF, &found, None, paths, stop, phase)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn item(version: &str, name: &str) -> serde_json::Value {
        json!({"tag_name": version, "prerelease": true, "draft": false,
            "assets": [{"name": name, "size": 10, "digest": format!("sha256:{}", "a".repeat(64)),
                "browser_download_url": format!("https://github.com/alportblu/plervo-pdf/releases/download/{version}/{name}")}]})
    }
    #[test]
    fn a_linux_release_never_hides_the_mac_release_or_another_architecture() {
        let list = json!([
            item("v0.1.0-alpha.2", "plervo-pdf-0.1.0-alpha.2-macos-aarch64.dmg"),
            item("v0.1.0-alpha.3", "plervo-pdf-0.1.0-alpha.3-linux-x86_64.tar.gz"),
            item("v0.1.0-alpha.4", "plervo-pdf-0.1.0-alpha.4-linux-aarch64.tar.gz"),
            item("v0.1.0-alpha.5", "plervo-suite-0.1.0-alpha.5-linux-x86_64.tar.gz")
        ])
        .to_string();
        assert_eq!(select_release_for(&list, Platform { os: Os::MacOs, arch: Arch::Arm64 }).unwrap().tag, "v0.1.0-alpha.2");
        assert_eq!(select_release_for(&list, Platform { os: Os::Linux, arch: Arch::X64 }).unwrap().tag, "v0.1.0-alpha.3");
    }
    #[test]
    fn older_releases_cannot_replace_a_newer_installation() {
        let newer = plervo_suite_core::version::parse("0.2.0").unwrap();
        let older = plervo_suite_core::version::parse("0.1.0-alpha.1").unwrap();
        assert!(refuse_downgrade(Some(&newer), Some(&older)).is_err());
        assert!(refuse_downgrade(Some(&older), Some(&newer)).is_ok());
        assert!(refuse_downgrade(Some(&older), Some(&older)).is_ok());
    }
    #[test]
    fn catalog_ignores_suite_only_drafts_and_foreign_downloads() {
        let pdf = item("v0.1.0-alpha.1", "plervo-pdf-0.1.0-alpha.1-macos-aarch64.dmg");
        let mut draft = item("v8.0.0", "plervo-pdf-8.0.0-macos-aarch64.dmg");
        draft["draft"] = json!(true);
        let mut foreign = item("v9.0.0", "plervo-pdf-9.0.0-macos-aarch64.dmg");
        foreign["assets"][0]["browser_download_url"] = json!("https://evil.test/payload.dmg");
        let list = json!([item("v1.0.0", "plervo-suite-1.0.0-macos-aarch64.dmg"), draft, foreign, pdf]);
        let r = select_release_for(&list.to_string(), Platform { os: Os::MacOs, arch: Arch::Arm64 }).unwrap();
        assert_eq!(r.tag, "v0.1.0-alpha.1");
        validate_download(&r, &r.assets[0]).unwrap();
    }
    #[test]
    fn refuses_unsigned_metadata_traversal_and_unbounded_downloads() {
        let r = select_release_for(
            &json!([item("v0.1.0", "plervo-pdf-0.1.0-macos-aarch64.dmg")]).to_string(),
            Platform { os: Os::MacOs, arch: Arch::Arm64 },
        )
        .unwrap();
        let a = &r.assets[0];
        let mut changed = a.clone();
        changed.sha256 = None;
        assert!(validate_download(&r, &changed).is_err());
        let mut changed = a.clone();
        changed.size = u64::MAX;
        assert!(validate_download(&r, &changed).is_err());
        let mut changed = a.clone();
        changed.url.push_str("?redirect=evil");
        assert!(validate_download(&r, &changed).is_err());
        let mut changed = a.clone();
        changed.name = "plervo-pdf-0.2.0-macos-aarch64.dmg".into();
        changed.url = format!("https://github.com/{}/releases/download/{}/{}", PDF.repo, r.tag, changed.name);
        assert!(validate_download(&r, &changed).is_err());
        let mut changed = r.clone();
        changed.tag = "v1/../evil".into();
        assert!(validate_download(&changed, a).is_err());
    }
}

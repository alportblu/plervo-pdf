//! Per-user Linux PDF payloads. Only our fixed destination is ever executed or replaced.
use std::io::Read;
use std::os::unix::fs::PermissionsExt;
use std::path::{Component, Path, PathBuf};

use plervo_suite_core::{InstallSource, Installed};
use serde::{Deserialize, Serialize};

use crate::{http, install, paths::Paths, process};

const APP: &str = "plervo-pdf";
const RECEIPT: &str = ".plervo-receipt.json";
const MAX_EXPANDED: u64 = 2 * 1024 * 1024 * 1024;

#[derive(Serialize, Deserialize)]
struct Receipt {
    version: String,
    binary_sha256: String,
}

fn real_dir(path: &Path) -> Result<(), String> {
    match path.symlink_metadata() {
        Ok(m) if m.is_dir() && !m.file_type().is_symlink() => Ok(()),
        Ok(_) => Err(format!("{} is not a regular installation directory; nothing was changed", path.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

fn root(paths: &Paths) -> Result<PathBuf, String> {
    for path in [&paths.data, &paths.apps(), &paths.apps().join(APP), &paths.apps().join(APP).join("current")] {
        real_dir(path)?;
    }
    Ok(paths.apps().join(APP))
}

pub fn detected(paths: &Paths) -> Result<Option<Installed>, String> {
    let current = root(paths)?.join("current");
    if !current.exists() {
        return Ok(None);
    }
    let receipt_path = current.join(RECEIPT);
    let metadata = receipt_path.symlink_metadata().map_err(|e| format!("Installation receipt missing: {e}"))?;
    if !metadata.is_file() || metadata.len() > 4096 {
        return Err("Invalid installation receipt; the app was left unchanged.".into());
    }
    let receipt: Receipt =
        serde_json::from_slice(&std::fs::read(receipt_path).map_err(|e| e.to_string())?).map_err(|_| "Unreadable installation receipt")?;
    let version = plervo_suite_core::version::parse(&receipt.version).ok_or("Invalid installed version")?;
    real_dir(&current.join("bin"))?;
    let binary = current.join("bin").join(APP);
    let metadata = binary.symlink_metadata().map_err(|e| e.to_string())?;
    if !metadata.is_file() || metadata.len() > 512 * 1024 * 1024 || http::sha256_file(&binary)? != receipt.binary_sha256 {
        return Err("The installed PDF binary changed; it was not opened or replaced.".into());
    }
    Ok(Some(Installed { version: Some(version), source: InstallSource::Launcher, location: binary.display().to_string(), package: None }))
}

fn require_stopped(paths: &Paths) -> Result<(), String> {
    if detected(paths)?.is_some() && process::is_running(&root(paths)?.join("current/bin").join(APP)) {
        return Err("Plervo PDF is running. Quit it, then try again.".into());
    }
    Ok(())
}

/// Install a checksum-verified archive. Stage the complete payload on the same filesystem,
/// then atomically replace it; preferences and documents live outside this directory.
pub fn install_archive(file: &Path, version: &str, paths: &Paths) -> Result<(), String> {
    plervo_suite_core::version::parse(version).ok_or("Invalid release version")?;
    require_stopped(paths)?;
    let root = root(paths)?;
    std::fs::create_dir_all(&root).map_err(|e| e.to_string())?;
    let stage = tempfile::Builder::new().prefix(".stage-").tempdir_in(&root).map_err(|e| e.to_string())?;
    let tree = unpack(file, stage.path())?;
    let binary = tree.join("bin").join(APP);
    if binary.metadata().map_err(|e| e.to_string())?.len() > 512 * 1024 * 1024 {
        return Err("The PDF executable exceeds the size limit.".into());
    }
    let mut header = [0u8; 20];
    std::fs::File::open(&binary).and_then(|mut f| f.read_exact(&mut header)).map_err(|e| format!("Missing PDF executable: {e}"))?;
    // ELF64 little-endian x86_64, the architecture qualified for this release.
    if header.get(..6) != Some(b"\x7fELF\x02\x01") || header.get(18..20) != Some(&[62, 0]) {
        return Err("This archive does not contain a Linux x86_64 PDF executable.".into());
    }
    let receipt = Receipt { version: version.into(), binary_sha256: http::sha256_file(&binary)? };
    let bytes = serde_json::to_vec(&receipt).map_err(|e| e.to_string())?;
    std::fs::write(tree.join(RECEIPT), bytes).map_err(|e| e.to_string())?;
    // Check again after download/extraction, before replacing the installed copy.
    require_stopped(paths)?;
    install::swap_into_place(&tree, &root.join("current"))?;
    detected(paths)?.ok_or("The installed app could not be verified.")?;
    Ok(())
}

fn unpack(file: &Path, stage: &Path) -> Result<PathBuf, String> {
    let input = std::fs::File::open(file).map_err(|e| e.to_string())?;
    let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(input));
    let mut total = 0u64;
    let mut top: Option<PathBuf> = None;
    for (count, entry) in archive.entries().map_err(|e| e.to_string())?.enumerate() {
        if count >= 10_000 {
            return Err("The archive contains too many entries.".into());
        }
        let mut entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path().map_err(|e| e.to_string())?.into_owned();
        let components: Vec<_> = path.components().collect();
        if components.is_empty() || components.iter().any(|c| !matches!(c, Component::Normal(_))) {
            return Err("Unsafe path in the archive.".into());
        }
        let first = PathBuf::from(components.first().ok_or("Empty archive path")?.as_os_str());
        if top.as_ref().is_some_and(|expected| expected != &first) {
            return Err("The archive has multiple top-level folders.".into());
        }
        top = Some(first);
        let kind = entry.header().entry_type();
        if !kind.is_dir() && !kind.is_file() {
            return Err("Links and special files are not allowed in app archives.".into());
        }
        if components.len() == 1 && !kind.is_dir() || components.get(1).is_some_and(|c| c.as_os_str() != "bin" && c.as_os_str() != "share") {
            return Err("Unexpected files in the app archive.".into());
        }
        total = total.checked_add(entry.size()).ok_or("Archive too large")?;
        if total > MAX_EXPANDED {
            return Err("The unpacked app exceeds the size limit.".into());
        }
        if !entry.unpack_in(stage).map_err(|e| e.to_string())? {
            return Err("Archive entry escaped its destination.".into());
        }
        let mode = if kind.is_dir() || components.get(1).is_some_and(|c| c.as_os_str() == "bin") { 0o755 } else { 0o644 };
        std::fs::set_permissions(stage.join(path), std::fs::Permissions::from_mode(mode)).map_err(|e| e.to_string())?;
    }
    Ok(stage.join(top.ok_or("Empty app archive")?))
}

pub fn open(paths: &Paths) -> Result<(), String> {
    detected(paths)?.ok_or("Plervo PDF is not installed.")?;
    process::launch_binary(&root(paths)?.join("current/bin").join(APP), &[])
}

pub fn uninstall(paths: &Paths) -> Result<(), String> {
    require_stopped(paths)?;
    detected(paths)?.ok_or("Plervo PDF is not installed.")?;
    std::fs::remove_dir_all(root(paths)?.join("current")).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::{Compression, write::GzEncoder};

    fn archive(path: &Path, link: bool, marker: u8) {
        let mut tar = tar::Builder::new(GzEncoder::new(std::fs::File::create(path).unwrap(), Compression::fast()));
        let mut bytes = [0u8; 21];
        bytes[..6].copy_from_slice(b"\x7fELF\x02\x01");
        bytes[18] = 62;
        bytes[20] = marker;
        let mut header = tar::Header::new_gnu();
        header.set_size(bytes.len() as u64);
        header.set_mode(0o755);
        header.set_cksum();
        tar.append_data(&mut header, "app/bin/plervo-pdf", bytes.as_slice()).unwrap();
        if link {
            let mut header = tar::Header::new_gnu();
            header.set_entry_type(tar::EntryType::Symlink);
            header.set_size(0);
            header.set_mode(0o777);
            header.set_link_name("/tmp").unwrap();
            header.set_cksum();
            tar.append_data(&mut header, "app/share/escape", std::io::empty()).unwrap();
        }
        tar.into_inner().unwrap().finish().unwrap();
    }
    #[test]
    fn update_and_uninstall_preserve_documents_and_reject_bad_archives() {
        let temp = tempfile::tempdir().unwrap();
        let paths = Paths::under(temp.path());
        let package = temp.path().join("app.tar.gz");
        std::fs::create_dir_all(paths.apps().join(APP).join("home")).unwrap();
        let document = paths.apps().join(APP).join("home/document.pdf");
        std::fs::write(&document, b"user document").unwrap();
        archive(&package, false, 1);
        install_archive(&package, "0.1.0-alpha.2", &paths).unwrap();
        let installed = detected(&paths).unwrap().unwrap();
        archive(&package, true, 2);
        assert!(install_archive(&package, "0.1.0-alpha.3", &paths).is_err());
        assert_eq!(detected(&paths).unwrap().unwrap(), installed);
        archive(&package, false, 3);
        install_archive(&package, "0.1.0-alpha.3", &paths).unwrap();
        assert_eq!(detected(&paths).unwrap().unwrap().version.unwrap().to_string(), "0.1.0-alpha.3");
        uninstall(&paths).unwrap();
        assert!(detected(&paths).unwrap().is_none());
        assert_eq!(std::fs::read(&document).unwrap(), b"user document");
    }
    #[test]
    fn symlinked_destinations_and_modified_binaries_are_refused() {
        let temp = tempfile::tempdir().unwrap();
        let paths = Paths::under(temp.path());
        let package = temp.path().join("app.tar.gz");
        archive(&package, false, 1);
        install_archive(&package, "0.1.0-alpha.3", &paths).unwrap();
        let binary = PathBuf::from(detected(&paths).unwrap().unwrap().location);
        std::fs::write(&binary, b"changed").unwrap();
        assert!(detected(&paths).is_err());
        assert!(install_archive(&package, "0.1.0-alpha.3", &paths).is_err());
        assert!(uninstall(&paths).is_err());
        let outside = temp.path().join("outside");
        std::fs::rename(paths.apps().join(APP), &outside).unwrap();
        std::os::unix::fs::symlink(&outside, paths.apps().join(APP)).unwrap();
        assert!(install_archive(&package, "0.1.0-alpha.3", &paths).is_err());
        assert!(uninstall(&paths).is_err());
        assert!(outside.join("current/bin/plervo-pdf").is_file());
    }
}

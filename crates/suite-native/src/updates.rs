//! Read-only update discovery. The Suite's installer is opened only after a user's click.
use std::collections::BTreeMap;

use plervo_suite_core::{
    Asset, PackageKind, Platform, Release,
    catalog::{LAUNCHER, SUITE},
    distro::{native_package, parse_os_release},
    platform::pick_asset,
    version::{Version, parse},
};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct SuiteUpdate {
    pub version: Version,
    pub downloads: Vec<Asset>,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct Updates {
    pub suite: Option<SuiteUpdate>,
    pub apps: BTreeMap<String, Release>,
}

pub fn check() -> Result<Updates, String> {
    let platform = crate::manager::supported_platform().ok_or("Updates for this platform are managed separately.")?;
    let current = parse(env!("CARGO_PKG_VERSION")).ok_or("Unknown installed Suite version")?;
    let distro = std::fs::read_to_string("/etc/os-release").unwrap_or_default();
    from_catalog(&crate::manager::release_catalog()?, platform, &distro, &current)
}

fn suite_kinds(platform: Platform, distro: &str) -> &'static [PackageKind] {
    match platform.os {
        plervo_suite_core::Os::MacOs => &[PackageKind::Dmg],
        plervo_suite_core::Os::Linux => match native_package(&parse_os_release(distro)) {
            Some(PackageKind::Deb) => &[PackageKind::Deb],
            Some(PackageKind::Rpm) => &[PackageKind::Rpm],
            _ => &[],
        },
        _ => &[],
    }
}

fn from_catalog(json: &str, platform: Platform, distro: &str, current: &Version) -> Result<Updates, String> {
    let values: serde_json::Value = serde_json::from_str(json).map_err(|_| "Unreadable release catalog")?;
    let list = values.as_array().ok_or("Invalid release catalog")?;
    let mut updates = Updates::default();
    for value in list.iter().take(100) {
        if value["draft"].as_bool().unwrap_or(false) {
            continue;
        }
        let text = serde_json::to_string(value).map_err(|e| e.to_string())?;
        let Ok(release) = plervo_suite_core::release::parse_github_release(&text, LAUNCHER.repo) else { continue };
        let Some(version) = release.version.as_ref() else { continue };
        for app in &SUITE {
            let kinds = if platform.os == plervo_suite_core::Os::Linux { &[PackageKind::TarGz][..] } else { &[PackageKind::Dmg][..] };
            if let Some((asset, _)) = pick_asset(&release, &[app.id], platform, kinds)
                && crate::manager::validate_download_for(app, &release, asset).is_ok()
                && updates.apps.get(app.id).and_then(|r| r.version.as_ref()).is_none_or(|old| version > old)
            {
                updates.apps.insert(app.id.to_owned(), release.clone());
            }
        }
        // Stable installations never move to previews automatically; existing alpha users see newer alphas.
        if version <= current || (current.pre.is_empty() && (release.prerelease || !version.pre.is_empty())) {
            continue;
        }
        let downloads: Vec<_> = suite_kinds(platform, distro)
            .iter()
            .filter_map(|kind| pick_asset(&release, &[LAUNCHER.id], platform, &[*kind]).map(|(asset, _)| asset))
            .filter(|asset| crate::manager::validate_download_for(&LAUNCHER, &release, asset).is_ok())
            .cloned()
            .collect();
        if !downloads.is_empty() && updates.suite.as_ref().is_none_or(|old| version > &old.version) {
            updates.suite = Some(SuiteUpdate { version: version.clone(), downloads });
        }
    }
    Ok(updates)
}

#[cfg(test)]
mod tests {
    use super::*;
    use plervo_suite_core::{Arch, Os};
    use serde_json::json;

    fn release(version: &str, files: &[&str]) -> serde_json::Value {
        json!({"tag_name":format!("v{version}"),"draft":false,"prerelease":version.contains('-'),
            "assets":files.iter().map(|name|json!({"name":name,"size":10,"digest":format!("sha256:{}","a".repeat(64)),
                "browser_download_url":format!("https://github.com/alportblu/plervo-pdf/releases/download/v{version}/{name}")})).collect::<Vec<_>>()})
    }
    fn check_for(items: serde_json::Value, os: Os, distro: &str, current: &str) -> Updates {
        from_catalog(
            &items.to_string(),
            Platform { os, arch: if os == Os::MacOs { Arch::Arm64 } else { Arch::X64 } },
            distro,
            &parse(current).unwrap(),
        )
        .unwrap()
    }
    #[test]
    fn suite_and_each_app_choose_their_own_newest_compatible_release() {
        let list = json!([
            release("0.1.0-alpha.9", &["plervo-pdf-0.1.0-alpha.9-macos-aarch64.dmg"]),
            release("0.1.0-alpha.8", &["plervo-suite-0.1.0-alpha.8-macos-aarch64.dmg", "plervo-photo-0.1.0-alpha.8-macos-aarch64.dmg"]),
            release("0.1.0-alpha.7", &["plervo-suite-0.1.0-alpha.7-macos-aarch64.dmg"])
        ]);
        let result = check_for(list, Os::MacOs, "", "0.1.0-alpha.5");
        assert_eq!(result.suite.unwrap().version.to_string(), "0.1.0-alpha.8");
        assert_eq!(result.apps["plervo-pdf"].version_label(), "0.1.0-alpha.9");
        assert_eq!(result.apps["plervo-photo"].version_label(), "0.1.0-alpha.8");
    }
    #[test]
    fn linux_uses_the_distro_package_and_never_guesses_an_unknown_distribution() {
        let list = json!([release("0.2.0", &["plervo-suite-0.2.0-linux-x86_64.deb", "plervo-suite-0.2.0-linux-x86_64.rpm"])]);
        for (distro, suffix) in [("ID=ubuntu", ".deb"), ("ID=linuxmint\nID_LIKE=\"ubuntu debian\"", ".deb"), ("ID=fedora", ".rpm")] {
            let result = check_for(list.clone(), Os::Linux, distro, "0.1.0");
            assert!(result.suite.unwrap().downloads[0].name.ends_with(suffix));
        }
        assert!(check_for(list, Os::Linux, "ID=arch", "0.1.0").suite.is_none());
    }
    #[test]
    fn no_downgrades_drafts_foreign_urls_missing_checksums_or_wrong_architecture() {
        let base = release("0.2.0", &["plervo-suite-0.2.0-macos-aarch64.dmg"]);
        for change in 0..5 {
            let mut bad = base.clone();
            match change {
                0 => bad["draft"] = json!(true),
                1 => bad["assets"][0]["browser_download_url"] = json!("https://example.com/payload.dmg"),
                2 => bad["assets"][0]["digest"] = json!(null),
                3 => bad = release("0.2.0", &["plervo-suite-0.2.0-macos-x86_64.dmg"]),
                _ => bad = release("0.2.0", &["plervo-pdf-0.2.0-macos-aarch64.dmg"]),
            }
            assert!(check_for(json!([bad]), Os::MacOs, "", "0.1.0").suite.is_none());
        }
        for current in ["0.2.0", "0.3.0"] {
            assert!(check_for(json!([base.clone()]), Os::MacOs, "", current).suite.is_none());
        }
        let preview = release("0.3.0-alpha.1", &["plervo-suite-0.3.0-alpha.1-macos-aarch64.dmg"]);
        assert!(check_for(json!([preview]), Os::MacOs, "", "0.2.0").suite.is_none());
        assert!(crate::manager::known_app(&LAUNCHER).is_err(), "self-replacement must remain disabled");
    }
}

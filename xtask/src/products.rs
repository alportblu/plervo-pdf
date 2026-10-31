//! Reproducible source overlays for the Plervo product family. This never publishes an app.
use anyhow::{Context, Result, bail, ensure};
use serde::Deserialize;
use std::{
    collections::BTreeSet,
    fs,
    path::{Component, Path},
    process::Command,
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Product {
    app: String,
    product: String,
    name: String,
    source_repository: String,
    tag: String,
    commit: String,
    release_version: String,
    status: String,
    windows_upgrade_code: String,
    fonts_commit: String,
    patch_sha256: String,
    notices_sha256: String,
    notice_manifest_sha256: String,
    icons: Vec<Icon>,
    remove: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Icon {
    to: String,
    from: String,
    sha256: String,
}

fn hex(value: &str, size: usize) -> bool {
    value.len() == size && value.bytes().all(|b| b.is_ascii_hexdigit())
}

fn slug(value: &str) -> bool {
    !value.is_empty() && value.len() <= 40 && value.bytes().all(|b| b.is_ascii_lowercase() || b == b'-')
}

fn relative(value: &str, prefix: &str) -> bool {
    value.starts_with(prefix)
        && !value.contains(['\\', '\n', '\r', '\0', ':'])
        && !value.split('/').any(|part| part.is_empty() || part == "." || part == "..")
        && Path::new(value).components().all(|c| matches!(c, Component::Normal(_)))
}

fn validate(product: &Product) -> Result<()> {
    ensure!(slug(&product.product) && slug(&product.app), "invalid product identifier");
    ensure!(product.source_repository == format!("storytold/{}", product.app), "source repository must be the pinned public upstream");
    ensure!(hex(&product.commit, 40) && hex(&product.fonts_commit, 40) && hex(&product.patch_sha256, 64), "invalid source or patch pin");
    ensure!(product.tag.starts_with('v') && !product.tag.contains(['/', '\\', ' ', '\n']), "invalid upstream tag");
    ensure!(product.name.starts_with("Plervo ") && !product.name.contains(['\n', '\r']), "invalid product name");
    ensure!(
        matches!(product.release_version.as_str(), "0.1.0-alpha.8" | "0.1.0-alpha.9") && product.status == "preparing",
        "unqualified products must stay internal"
    );
    ensure!(hex(&product.notices_sha256, 64) && hex(&product.notice_manifest_sha256, 64), "invalid notice inventory pins");
    ensure!(
        product.windows_upgrade_code.len() == 36 && product.windows_upgrade_code.bytes().all(|b| b.is_ascii_hexdigit() || b == b'-'),
        "invalid Windows identity"
    );
    ensure!(!product.icons.is_empty() && product.icons.len() <= 100 && product.remove.len() <= 100, "invalid asset inventory");
    let mut destinations = BTreeSet::new();
    for icon in &product.icons {
        ensure!(relative(&icon.to, "assets/app-icon/") && relative(&icon.from, &format!("assets/products/{}/", product.product)), "unsafe icon path");
        ensure!(hex(&icon.sha256, 64) && destinations.insert(&icon.to), "invalid or duplicate icon");
    }
    for path in &product.remove {
        let old_dmg_asset = matches!(
            path.as_str(),
            "packaging/macos/dmg/background.svg"
                | "packaging/macos/dmg/background.tiff"
                | "packaging/macos/dmg/dmg-layout.DS_Store"
                | "packaging/macos/dmg/generate.py"
        );
        ensure!(relative(path, "docs/brand/") || relative(path, "assets/app-icon/") || old_dmg_asset, "unsafe removal path");
        ensure!(!destinations.contains(path), "an icon cannot also be removed");
    }
    Ok(())
}

fn checked_file(root: &Path, relative: &str) -> Result<()> {
    let mut path = root.to_path_buf();
    for part in Path::new(relative).components() {
        ensure!(matches!(part, Component::Normal(_)), "invalid path component");
        path.push(part);
        match fs::symlink_metadata(&path) {
            Ok(metadata) => ensure!(!metadata.file_type().is_symlink(), "refusing a symlink: {}", path.display()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
            Err(e) => return Err(e.into()),
        }
    }
    Ok(())
}

fn git_output(root: &Path, args: &[&str]) -> Result<String> {
    let result = Command::new("git").arg("-C").arg(root).args(args).output().context("running git")?;
    ensure!(result.status.success(), "git operation failed: {}", String::from_utf8_lossy(&result.stderr));
    Ok(String::from_utf8(result.stdout)?.trim().to_owned())
}

fn audit_policy(product: &Product) -> Result<String> {
    let mut policy: toml::Value = toml::from_str(include_str!("../../products/deny.toml"))?;
    let ignores =
        policy.get_mut("advisories").and_then(|v| v.get_mut("ignore")).and_then(toml::Value::as_array_mut).context("missing advisory policy")?;
    let mut add = |id: &str, reason: &str| {
        ignores.push(toml::Value::Table(toml::map::Map::from_iter([
            ("id".into(), toml::Value::String(id.into())),
            ("reason".into(), toml::Value::String(reason.into())),
        ])));
    };
    if matches!(product.product.as_str(), "video" | "develop") {
        add(
            "RUSTSEC-2023-0071",
            "Reviewed 2026-10-16: model-download TLS verifies public server keys only; no client authentication, and the provider rejects every private key. Valid-key regression required. RSA itself is not patched; this exception does not authorize RSA signing/decryption.",
        );
    }
    if product.product == "develop" {
        add(
            "RUSTSEC-2026-0192",
            "Reviewed 2026-10-16: ab_glyph watermark rasterization parses only compile-time embedded Inter and pinned craft-fonts; no user font input. Maintenance advisory, not a known vulnerability. Retain watermark regression tests; migrate rasterization through a reviewed upstream update.",
        );
    }
    Ok(toml::to_string_pretty(&policy)?)
}

pub fn run(args: &[String]) -> Result<()> {
    let root = crate::gates::root();
    let products: Vec<Product> = serde_json::from_slice(&fs::read(root.join("products/sources.json"))?)?;
    ensure!(products.len() == 12, "the source catalog must contain the twelve additional apps");
    let mut names = BTreeSet::new();
    let mut upgrade_codes = BTreeSet::new();
    for product in &products {
        validate(product)?;
        ensure!(names.insert(&product.product) && upgrade_codes.insert(&product.windows_upgrade_code), "duplicate product identity");
        let patch = fs::read(root.join(format!("products/patches/{}.patch", product.product)))?;
        ensure!(crate::assets::sha256_hex(&patch) == product.patch_sha256, "patch checksum differs for {}", product.product);
        for (file, expected) in [("THIRD_PARTY_NOTICES.txt", &product.notices_sha256), ("manifest.json", &product.notice_manifest_sha256)] {
            let source = root.join("products/licenses").join(&product.product).join(file);
            ensure!(crate::assets::sha256_hex(&fs::read(source)?) == *expected, "notice checksum differs for {}: {file}", product.product);
        }
        for icon in &product.icons {
            ensure!(crate::assets::sha256_hex(&fs::read(root.join(&icon.from))?) == icon.sha256, "icon checksum differs: {}", icon.from);
        }
    }
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    match args.as_slice() {
        [] => println!("12 pinned product sources, unique identities, patches and icon hashes verified; all remain internal."),
        ["--matrix"] => {
            let include: Vec<_> = products
                .iter()
                .map(|p| serde_json::json!({"app":p.app,"product":p.product,"upstream":p.source_repository,"commit":p.commit,"fonts":p.fonts_commit}))
                .collect();
            println!("{}", serde_json::json!({"include":include}));
        }
        ["--audit-policy", name] => {
            let product = products.iter().find(|p| p.product == *name).context("unknown product")?;
            print!("{}", audit_policy(product)?);
        }
        ["--prepare", name, checkout] => {
            let product = products.iter().find(|p| p.product == *name).context("unknown product")?;
            let checkout = Path::new(checkout).canonicalize()?;
            ensure!(
                Path::new(&git_output(&checkout, &["rev-parse", "--show-toplevel"])?).canonicalize()? == checkout,
                "checkout must be a repository root"
            );
            ensure!(git_output(&checkout, &["rev-parse", "HEAD"])? == product.commit, "checkout does not match the reviewed upstream pin");
            ensure!(
                git_output(&checkout, &["status", "--porcelain", "--untracked-files=all"])?.is_empty(),
                "checkout has existing changes; nothing was changed"
            );
            let patch = root.join(format!("products/patches/{}.patch", product.product));
            let patch = patch.to_str().context("non-UTF-8 patch path")?;
            git_output(&checkout, &["apply", "--check", patch])?;
            for icon in &product.icons {
                checked_file(&checkout, &icon.to)?;
            }
            for path in &product.remove {
                checked_file(&checkout, path)?;
            }
            git_output(&checkout, &["apply", patch])?;
            for path in &product.remove {
                checked_file(&checkout, path)?;
                match fs::remove_file(checkout.join(path)) {
                    Ok(()) => (),
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
                    Err(e) => return Err(e.into()),
                }
            }
            for icon in &product.icons {
                checked_file(&checkout, &icon.to)?;
                let destination = checkout.join(&icon.to);
                fs::create_dir_all(destination.parent().context("icon has no parent")?)?;
                fs::copy(root.join(&icon.from), destination)?;
            }
            for (from, to) in [
                (format!("{}/THIRD_PARTY_NOTICES.txt", product.product), "THIRD_PARTY_NOTICES.txt"),
                (format!("{}/manifest.json", product.product), "packaging/licenses/manifest.json"),
                ("verify.py".to_owned(), "packaging/licenses/verify.py"),
            ] {
                checked_file(&checkout, to)?;
                let destination = checkout.join(to);
                fs::create_dir_all(destination.parent().context("notice has no parent")?)?;
                fs::copy(root.join("products/licenses").join(from), destination)?;
            }
            crate::licenses::verify_at(&checkout)?;
            checked_file(&checkout, "PLERVO-BUILD.json")?;
            fs::write(
                checkout.join("PLERVO-BUILD.json"),
                serde_json::to_vec_pretty(&serde_json::json!({
                    "product":product.name, "version":product.release_version, "upstream":product.source_repository,
                    "upstream_commit":product.commit, "patch_sha256":product.patch_sha256,
                    "fonts_commit":product.fonts_commit,
                    "notices_sha256":product.notices_sha256,
                    "notice_manifest_sha256":product.notice_manifest_sha256,
                    "icons":product.icons.iter().map(|icon| serde_json::json!({"path":icon.to,"sha256":icon.sha256})).collect::<Vec<_>>(),
                    "qualification":"internal candidate; not approved for distribution"
                }))?,
            )?;
            // Upstream asset gates inspect the Git index. Stage this fresh, prepared snapshot
            // so removed original brand files are not mistaken for files still being shipped.
            git_output(&checkout, &["add", "--all"])?;
            println!("Prepared {} from {}; no app was installed or published.", product.name, product.commit);
        }
        _ => bail!("usage: cargo xtask products [--matrix | --audit-policy PRODUCT | --prepare PRODUCT CLEAN_CHECKOUT]"),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn applicability_exceptions_cannot_spread_to_unreviewed_products() {
        let products: Vec<Product> = serde_json::from_str(include_str!("../../products/sources.json")).unwrap();
        for product in products {
            let policy = audit_policy(&product).unwrap();
            let parsed: toml::Value = toml::from_str(&policy).unwrap();
            assert_eq!(policy.contains("RUSTSEC-2023-0071"), matches!(product.product.as_str(), "video" | "develop"));
            assert_eq!(policy.contains("RUSTSEC-2026-0192"), product.product == "develop");
            assert!(!policy.contains("RUSTSEC-2026-0206"));
            let advisory_ids: Vec<_> = parsed["advisories"]["ignore"].as_array().unwrap().iter().map(|v| v["id"].as_str().unwrap()).collect();
            for id in ["RUSTSEC-2026-0370", "RUSTSEC-2026-0371", "RUSTSEC-2026-0372"] {
                assert_eq!(advisory_ids.iter().filter(|&&found| found == id).count(), 1);
            }
            assert_eq!(
                advisory_ids.len(),
                5 + usize::from(matches!(product.product.as_str(), "video" | "develop")) + usize::from(product.product == "develop")
            );
            assert_eq!(parsed["sources"]["unknown-git"].as_str(), Some("deny"));
            assert!(parsed["licenses"]["exceptions"].as_array().unwrap().len() >= 20);
        }
    }

    #[test]
    fn source_boundaries_reject_traversal_foreign_repositories_and_unqualified_release_flags() {
        let mut products: Vec<Product> = serde_json::from_str(include_str!("../../products/sources.json")).unwrap();
        let mut p = products.remove(0);
        validate(&p).unwrap();
        p.remove.push("packaging/macos/dmg/background.svg".into());
        validate(&p).unwrap();
        *p.remove.last_mut().unwrap() = "packaging/macos/dmg/../../outside".into();
        assert!(validate(&p).is_err());
        p.remove.pop();
        p.icons[0].to = "assets/app-icon/../../outside".into();
        assert!(validate(&p).is_err());
        p.icons[0].to = "assets/app-icon/icon.png".into();
        p.source_repository = "unrelated/company".into();
        assert!(validate(&p).is_err());
        p.source_repository = format!("storytold/{}", p.app);
        p.status = "released".into();
        assert!(validate(&p).is_err());
        assert!(!relative("assets/app-icon/a\\b", "assets/app-icon/"));
        assert!(!relative("/assets/app-icon/a.png", "assets/app-icon/"));
        assert!(!relative("assets/app-icon/a/../b", "assets/app-icon/"));
    }

    #[cfg(unix)]
    #[test]
    fn a_linked_asset_directory_cannot_redirect_writes() {
        let root = std::env::temp_dir().join(format!("plervo-products-links-{}", std::process::id()));
        fs::create_dir_all(root.join("assets")).unwrap();
        std::os::unix::fs::symlink(std::env::temp_dir(), root.join("assets/app-icon")).unwrap();
        assert!(checked_file(&root, "assets/app-icon/icon.png").is_err());
        fs::remove_file(root.join("assets/app-icon")).unwrap();
        fs::remove_dir(root.join("assets")).unwrap();
        fs::remove_dir(root).unwrap();
    }
}

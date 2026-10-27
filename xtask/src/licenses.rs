//! Frozen distribution notices: dependency changes require a fresh notice review.

use std::collections::BTreeMap;

use crate::assets::sha256_hex;
use anyhow::{Context, ensure};
use serde::Deserialize;

#[derive(Deserialize)]
struct Manifest {
    inputs: BTreeMap<String, String>,
    notices_lf_sha256: String,
}

fn verify(bytes: &[u8], expected: &str, name: &str) -> anyhow::Result<()> {
    let text = std::str::from_utf8(bytes).with_context(|| format!("{name} must be UTF-8"))?.replace("\r\n", "\n");
    ensure!(
        sha256_hex(text.as_bytes()) == expected,
        "{name} changed: review and refresh THIRD_PARTY_NOTICES.txt and packaging/licenses/manifest.json before packaging"
    );
    Ok(())
}

pub fn run(args: &[String]) -> anyhow::Result<()> {
    ensure!(args.is_empty(), "usage: cargo xtask licenses");
    let root = crate::gates::root();
    verify_at(&root)
}

pub(crate) fn verify_at(root: &std::path::Path) -> anyhow::Result<()> {
    let manifest: Manifest = serde_json::from_slice(&std::fs::read(root.join("packaging/licenses/manifest.json"))?)?;
    ensure!(manifest.inputs.contains_key("Cargo.lock"), "the notice inventory must pin Cargo.lock");
    for (name, hash) in &manifest.inputs {
        let path = std::path::Path::new(name);
        ensure!(path.components().all(|c| matches!(c, std::path::Component::Normal(_))), "notice input must be a relative repository path: {name}");
        verify(&std::fs::read(root.join(path)).with_context(|| format!("read notice input {name}"))?, hash, name)?;
    }
    verify(&std::fs::read(root.join("THIRD_PARTY_NOTICES.txt"))?, &manifest.notices_lf_sha256, "THIRD_PARTY_NOTICES.txt")?;
    println!("Third-party distribution notices match all {} reviewed inputs.", manifest.inputs.len());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dependency_or_notice_changes_require_review() {
        let original = b"reviewed dependency versions and notices";
        let hash = sha256_hex(original);
        assert!(verify(original, &hash, "Cargo.lock").is_ok());
        for changed in [b"new dependency version".as_slice(), b"truncated license notice", b""] {
            assert!(verify(changed, &hash, "Cargo.lock").is_err());
        }
        assert!(verify(b"notice\r\n", &sha256_hex(b"notice\n"), "notice").is_ok());
    }
}

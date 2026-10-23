#!/usr/bin/env python3
"""Apply the small, pinned Store overlay after `cargo xtask products --prepare`.

The public product overlay remains the base. Store-specific changes and every
license are recorded with their exact hashes; this does not publish a package.
"""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import tomllib


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def prepare(product, source):
    root = Path(__file__).resolve().parents[3]
    definition = next(p for p in json.loads((root / "products/sources.json").read_text()) if p["product"] == product)
    source = source.resolve(strict=True)
    metadata = json.loads((source / "PLERVO-BUILD.json").read_text())
    head = subprocess.check_output(["git", "-C", str(source), "rev-parse", "HEAD"], text=True).strip()
    if head != definition["commit"] or metadata["patch_sha256"] != definition["patch_sha256"]:
        raise ValueError("Prepared source differs from the pinned product catalog")
    expected = json.loads((Path(__file__).parent / "overlays.json").read_text())
    overlay = None
    if product in expected:
        patch = Path(__file__).parent / expected[product]["file"]
        if sha(patch) != expected[product]["sha256"]:
            raise ValueError("Store overlay checksum differs")
        for mode in ("--check", "--index"):
            subprocess.run(["git", "-C", str(source), "apply", mode, str(patch)], check=True)
        overlay = expected[product]
    manifest = tomllib.loads((source / "Cargo.toml").read_text())
    if manifest["workspace"]["package"]["version"] != definition["release_version"]:
        raise ValueError("Prepared product version differs")
    licenses = {}
    for name in ("LICENSE-MIT", "LICENSE-APACHE"):
        original = subprocess.check_output(["git", "-C", str(source), "show", f"{head}:{name}"])
        if (source / name).read_bytes() != original:
            raise ValueError(f"Original source license was modified: {name}")
        licenses[name] = sha(source / name)
    provenance = {
        "product": product,
        "upstream_commit": head,
        "upstream_repository": definition["source_repository"],
        "base_patch_sha256": definition["patch_sha256"],
        "store_overlay": overlay,
        "lock_sha256": sha(source / "Cargo.lock"),
        "fonts_commit": definition["fonts_commit"],
        "release_version": definition["release_version"],
        "licenses": licenses,
        "features": "default only; desktop local-control is excluded",
    }
    (source / "PLERVO-STORE-BUILD.json").write_text(json.dumps(provenance, indent=2) + "\n")
    print(json.dumps(provenance, indent=2))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("product", choices=("video", "cad"))
    parser.add_argument("source", type=Path)
    args = parser.parse_args()
    prepare(args.product, args.source)

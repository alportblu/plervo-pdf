"""Fetch only the exact native-passed MSI candidates pinned for GUI diagnostics."""
import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import stat
import subprocess
import zipfile

REPO = "alportblu/plervo-pdf"
PRODUCTS = {"pdf", "photo", "vector", "video", "develop", "motion", "publish", "write", "slides", "sheets", "cad", "audio", "canvas"}


def validate(row):
    product = row["product"]
    assert product in PRODUCTS
    title = product.upper() if product in {"pdf", "cad"} else product.title()
    assert row["name"] == "Plervo " + title
    assert row["version"] == "0.1.0-alpha.4"
    assert row["msi"] == f"plervo-{product}-{row['version']}-windows-x64.msi"
    assert re.fullmatch(r"[0-9a-f]{40}", row["head"])
    assert re.fullmatch(r"sha256:[0-9a-f]{64}", row["artifact_digest"])
    assert re.fullmatch(r"[0-9a-f]{64}", row["msi_sha256"])
    assert all(type(row[key]) is int and row[key] > 0 for key in ("run", "job", "artifact_id"))


def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def extract_msi(archive, name, destination, expected):
    # Never extract an archive tree or accept another file through a basename match.
    with zipfile.ZipFile(archive) as source:
        matches = [entry for entry in source.infolist() if entry.filename == name]
        assert len(matches) == 1, "Missing or duplicate pinned MSI"
        entry = matches[0]
        assert PurePosixPath(name).name == name and "\\" not in name and ":" not in name
        assert not entry.is_dir() and not stat.S_ISLNK(entry.external_attr >> 16)
        assert 0 < entry.file_size <= 1024**3, "Oversized candidate"
        with source.open(entry) as original, destination.open("xb") as target:
            shutil.copyfileobj(original, target)
    assert digest(destination) == expected, "MSI differs from the reviewed candidate"


def api(route):
    return json.loads(subprocess.check_output(["gh", "api", f"repos/{REPO}/" + route], timeout=60))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--product", default="all", choices=["all", *sorted(PRODUCTS)])
    args = parser.parse_args()
    assert os.environ.get("GITHUB_ACTIONS") == "true" and os.environ.get("RUNNER_OS") == "Windows", "Disposable Windows CI only"
    rows = json.loads(Path(__file__).with_name("candidates.json").read_text())
    assert len({row["product"] for row in rows}) == len(rows)
    for row in rows:
        validate(row)
    selected = [row for row in rows if args.product in {"all", row["product"]}]
    assert selected, "No pinned qualified candidate for this product"
    out = Path("dist/windows-gui")
    out.mkdir(parents=True, exist_ok=False)
    for row in selected:
        job = api(f"actions/jobs/{row['job']}")
        assert job["run_id"] == row["run"] and job["name"] == row["job_name"]
        assert job["status"] == "completed" and job["conclusion"] == "success", "Native qualification did not pass"
        artifact = api(f"actions/artifacts/{row['artifact_id']}")
        assert not artifact["expired"] and artifact["name"] == row["artifact_name"]
        assert artifact["workflow_run"]["id"] == row["run"] and artifact["workflow_run"]["head_sha"] == row["head"]
        assert artifact["digest"] == row["artifact_digest"]
        folder = out / row["product"]
        folder.mkdir()
        archive = folder / "native-artifact.zip"
        with archive.open("xb") as stream:
            subprocess.run(["gh", "api", f"repos/{REPO}/actions/artifacts/{row['artifact_id']}/zip"], stdout=stream, check=True, timeout=600)
        assert "sha256:" + digest(archive) == row["artifact_digest"], "Artifact hash mismatch"
        extract_msi(archive, row["msi"], folder / row["msi"], row["msi_sha256"])
        archive.unlink()
        (folder / "provenance.json").write_text(json.dumps({"candidate": row, "native_job": job, "artifact": artifact, "hashes_verified": True}, indent=2) + "\n")
        print(row["product"], "native-passed MSI provenance and bytes verified", flush=True)
    (out / "selected.json").write_text(json.dumps(selected, indent=2) + "\n")


if __name__ == "__main__":
    main()

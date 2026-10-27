"""Reject stale distribution notices before any product is packaged (Python 3)."""
import hashlib
import json
from pathlib import Path, PurePosixPath
import re
import sys


def verify(root):
    root = Path(root).resolve()
    manifest = json.loads((root / "packaging/licenses/manifest.json").read_text(encoding="utf-8"))
    inputs = manifest["inputs"]
    if not {"Cargo.lock", "Cargo.toml"}.issubset(inputs):
        raise ValueError("The notice inventory must pin the dependency lock and workspace manifest")
    checks = {**inputs, "THIRD_PARTY_NOTICES.txt": manifest["notices_lf_sha256"]}
    for name, expected in checks.items():
        path = PurePosixPath(name)
        if not name or path.is_absolute() or ".." in path.parts or "\\" in name or ":" in name:
            raise ValueError("Invalid notice input path: " + name)
        source = root / path
        if source.resolve() != source or not re.fullmatch(r"[0-9a-f]{64}", expected):
            raise ValueError("Invalid notice input or digest: " + name)
        text = source.read_text(encoding="utf-8").replace("\r\n", "\n")
        actual = hashlib.sha256(text.encode()).hexdigest()
        if actual != expected:
            raise ValueError(name + " changed; review and refresh the third-party notices before packaging")
    return len(inputs)


if __name__ == "__main__":
    try:
        if len(sys.argv) != 2:
            raise ValueError("usage: python verify.py SOURCE_ROOT")
        print("Third-party notices match", verify(sys.argv[1]), "reviewed source inputs.")
    except (OSError, ValueError, KeyError, TypeError) as error:
        sys.exit("Notice verification failed: " + str(error))

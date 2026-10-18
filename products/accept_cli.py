#!/usr/bin/env python3
"""Exercise a supplied product CLI with private, original synthetic fixtures.

This checks actual release CLIs with independent file readers.
It is representative acceptance, not a claim of complete feature coverage.
All generated media are original work licensed MIT OR Apache-2.0. The script
never launches a desktop application or changes an installed application.
"""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import struct
import subprocess
import time
import wave
import zipfile

from PIL import Image, ImageDraw, ImageStat
from pypdf import PdfReader


parser = argparse.ArgumentParser()
parser.add_argument("--product", required=True)
parser.add_argument("--binary", required=True, type=Path)
parser.add_argument("--output", required=True, type=Path)
parser.add_argument("--source", required=True)
parser.add_argument("--expected-version", required=True)
args = parser.parse_args()
binary = args.binary.resolve(strict=True)
root = args.output.resolve()
root.mkdir(parents=True, exist_ok=False)
fixtures = root / "fixtures"
fixtures.mkdir()
env = os.environ.copy()
env.update({"FIGCRAFT_NO_SYSTEM_FONTS": "1", "PDFCRAFT_SYSTEM_FONTS": "0"})
for prefix in ["PLERVO_PHOTO", "PLERVO_VECTOR", "PLERVO_VIDEO", "PLERVO_DEVELOP", "PLERVO_MOTION", "PLERVO_PUBLISH", "PLERVO_WRITE", "PLERVO_SLIDES", "PLERVO_SHEETS", "PLERVO_CAD", "PLERVO_AUDIO", "PLERVO_CANVAS"]:
    env[prefix + "_CONFIG_DIR"] = str(root / "settings" / prefix.lower())
records = []


def run(argv, *, expected=0, external=False, timeout=90):
    command = [str(x) for x in argv]
    if not external:
        command.insert(0, str(binary))
    start = time.monotonic()
    result = subprocess.run(command, cwd=root, env=env, capture_output=True, timeout=timeout)
    record = {"command": command, "exit": result.returncode, "seconds": round(time.monotonic() - start, 3)}
    index = len(records) + 1
    (root / f"{index:02d}-stdout.txt").write_bytes(result.stdout)
    (root / f"{index:02d}-stderr.txt").write_bytes(result.stderr)
    record["stdout"] = f"{index:02d}-stdout.txt"
    record["stderr"] = f"{index:02d}-stderr.txt"
    records.append(record)
    print(args.product, index, command[1:3], result.returncode, flush=True)
    if expected == "error":
        assert result.returncode in (1, 2), (command, result.returncode, result.stderr[-2000:])
        assert b"panicked at" not in result.stderr, result.stderr[-2000:]
    else:
        assert result.returncode == expected, (command, result.returncode, result.stderr[-2000:])
    return result.stdout.decode("utf-8", errors="replace")


def image_ok(path, size=None, *, line_drawing=False):
    with Image.open(path) as image:
        image.load()
        assert image.width > 0 and image.height > 0
        if size:
            assert image.size == size, (image.size, size)
        # Sparse CAD strokes occupy far fewer pixels than a photo or a slide.
        minimum = 1 if line_drawing else 5
        assert max(ImageStat.Stat(image.convert("RGB")).stddev) > minimum, "blank or nearly blank image"
        records.append({"oracle": "Pillow", "file": str(path), "size": list(image.size), "nonblank": True})


def pdf_ok(path, contains=None):
    reader = PdfReader(path, strict=True)
    assert len(reader.pages) > 0
    text = "\n".join(page.extract_text() for page in reader.pages)
    assert path.stat().st_size > 200
    if contains:
        assert contains in text, text[:2000]
    records.append({"oracle": "pypdf", "file": str(path), "text_checked": contains})


image = Image.new("RGB", (320, 200), (20, 30, 50))
draw = ImageDraw.Draw(image)
draw.rectangle((20, 20, 140, 180), fill=(230, 70, 30))
draw.ellipse((165, 35, 295, 165), fill=(20, 210, 110))
image.save(fixtures / "shapes.png")
(fixtures / "shapes.svg").write_text('<svg xmlns="http://www.w3.org/2000/svg" width="320" height="200" viewBox="0 0 320 200"><rect width="320" height="200" fill="#141e32"/><rect x="20" y="20" width="120" height="160" fill="#e6461e"/><circle cx="230" cy="100" r="65" fill="#14d26e"/></svg>')
(fixtures / "text.txt").write_text("Plervo acceptance 2026\nOriginal synthetic document.\nSecond paragraph with numbers 12345.\n", encoding="utf-8")
(fixtures / "values.csv").write_text("Item,Quantity,Price\nAlpha,2,10\nBeta,3,5\n", encoding="utf-8")
(fixtures / "invalid.png").write_bytes(b"\x89PNG\r\n\x1a\ntruncated")
with wave.open(str(fixtures / "tone.wav"), "wb") as audio:
    audio.setnchannels(1)
    audio.setsampwidth(2)
    audio.setframerate(48000)
    audio.writeframes(b"".join(struct.pack("<h", round(10000 * math.sin(2 * math.pi * 440 * i / 48000))) for i in range(48000)))
fixture_hashes = {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in fixtures.iterdir()}
(fixtures / "ATTRIBUTION.json").write_text(json.dumps({"author": "Plervo contributors", "source": "Original procedural geometry, text, numeric table and sine wave", "license": "MIT OR Apache-2.0", "sha256": fixture_hashes}, indent=2) + "\n")


def photo():
    for extension in ["pcraft", "psd", "tif", "webp"]:
        target = root / ("roundtrip." + extension)
        run(["convert", fixtures / "shapes.png", target])
        run(["info", target])
        png = root / (extension + ".png")
        run(["convert", target, png])
        image_ok(png, (320, 200))
    run(["info", fixtures / "invalid.png"], expected="error")


def vector():
    native = root / "drawing.vectorcraft"
    run(["convert", fixtures / "shapes.svg", native])
    run(["info", native])
    for extension in ["png", "pdf", "svg"]:
        run(["convert", native, root / ("drawing." + extension)])
    image_ok(root / "drawing.png")
    pdf_ok(root / "drawing.pdf")
    run(["info", fixtures / "invalid.png"], expected="error")


def video():
    common = ["--data-dir", root / "video-data"]
    run(common + ["probe", fixtures / "tone.wav"])
    project = root / "acceptance.fcproj"
    # `inspect` is read-only and deliberately does not call the CLI's save hook.
    run(common + ["--demo", "exec", "file.saveAs", "path=" + str(project)])
    assert project.is_file(), "save command did not produce the project"
    run(common + ["--project", project, "render", "--seconds", "1", "--scale", "0.25", "--out", root / "frame.png"])
    image_ok(root / "frame.png")
    run(common + ["probe", fixtures / "invalid.png"], expected="error")


def develop():
    run(["render", fixtures / "shapes.png", "-o", root / "developed.png", "--set", "light.exposure=0.5", "--size", "320"])
    image_ok(root / "developed.png", (320, 200))
    with Image.open(fixtures / "shapes.png") as original, Image.open(root / "developed.png") as developed:
        assert original.convert("RGB").tobytes() != developed.convert("RGB").tobytes(), "exposure did not change the image"
    settle = root / "settle.jsonl"
    settle.write_text(json.dumps({"method": "ui.settle", "params": {"timeoutMs": 60000}}) + "\n")
    run(["snapshot", "--demo", "--script", settle, "--size", "1200x800", "-o", root / "interface.png"], timeout=180)
    image_ok(root / "interface.png", (1200, 800))
    run(["render", fixtures / "invalid.png", "-o", root / "invalid-output.png"], expected="error")
    assert not (root / "invalid-output.png").exists()


def motion():
    project = root / "acceptance.ecproj"
    run(["--demo", "info", "--json"])
    run(["--demo", "exec", "file.saveAs", "--params", json.dumps({"path": str(project)}), "--json"])
    run(["--project", project, "render-frame", "--time", "1", "--max-side", "640", "--out", root / "frame.png"])
    image_ok(root / "frame.png")
    run(["--project", root / "missing.ecproj", "info"], expected="error")


def publish():
    native = root / "layout.designcraft"
    run(["run", "--sample", "--export", native])
    run(["run", "--in", native, "--export", root / "layout.pdf", "--export", root / "layout.png"])
    image_ok(root / "layout.png")
    pdf_ok(root / "layout.pdf")
    run(["run", "--in", root / "missing.designcraft", "--export", root / "invalid.pdf"], expected="error")


def write():
    docx = root / "document.docx"
    run(["convert", fixtures / "text.txt", docx])
    with zipfile.ZipFile(docx) as archive:
        assert b"Plervo acceptance 2026" in archive.read("word/document.xml")
    run(["info", docx])
    run(["convert", docx, root / "document.pdf"])
    pdf_ok(root / "document.pdf", "Plervo acceptance 2026")
    run(["convert", docx, root / "roundtrip.txt"])
    assert "Plervo acceptance 2026" in (root / "roundtrip.txt").read_text()
    run(["info", root / "missing.docx"], expected="error")


def slides():
    native = root / "presentation.deckcraft"
    run(["run", "--sample", "--save", native, "--export", root / "slide.png"])
    run(["info", native])
    run(["convert", native, root / "presentation.pptx"])
    with zipfile.ZipFile(root / "presentation.pptx") as archive:
        assert "ppt/presentation.xml" in archive.namelist()
        assert any(x.startswith("ppt/slides/slide") and x.endswith(".xml") for x in archive.namelist())
    run(["info", root / "presentation.pptx"])
    image_ok(root / "slide.png")
    run(["info", root / "missing.pptx"], expected="error")


def sheets():
    import openpyxl
    xlsx = root / "workbook.xlsx"
    run(["convert", fixtures / "values.csv", xlsx])
    workbook = openpyxl.load_workbook(xlsx, read_only=True, data_only=False)
    sheet = workbook.active
    assert sheet["A2"].value == "Alpha"
    assert float(sheet["B2"].value) == 2 and float(sheet["C3"].value) == 5
    workbook.close()
    run(["info", xlsx, "--json"])
    calculation = run(["eval", "SUM(2,3,5)", "--json"])
    assert "10" in calculation, calculation
    run(["convert", xlsx, root / "roundtrip.csv"])
    assert "Alpha" in (root / "roundtrip.csv").read_text()
    run(["info", root / "missing.xlsx"], expected="error")


def cad():
    native = root / "drawing.dxf"
    run(["run", "--metric", "--cmd", 'line {"points":[[0,0],[30,20],[45,0]]}', "--save", native, "--export", root / "drawing.png"])
    info = json.loads(run(["info", native]))
    assert info["entityCount"] == 2 and info["extents"] == [[0.0, 0.0], [45.0, 20.0]], info
    run(["convert", native, root / "drawing.svg"])
    run(["convert", native, root / "drawing.pdf"])
    image_ok(root / "drawing.png", line_drawing=True)
    pdf_ok(root / "drawing.pdf")
    assert "<svg" in (root / "drawing.svg").read_text()
    run(["info", root / "missing.dxf"], expected="error")


def audio():
    run(["info", fixtures / "tone.wav"])
    run(["convert", fixtures / "tone.wav", root / "tone.flac", "--bit-depth", "16"])
    run(["convert", root / "tone.flac", root / "roundtrip.wav", "--bit-depth", "16"])
    with wave.open(str(root / "roundtrip.wav"), "rb") as decoded, wave.open(str(fixtures / "tone.wav"), "rb") as source:
        assert decoded.getparams()[:4] == source.getparams()[:4]
        # This CLI intentionally applies TPDF dither on each integer export.
        # Two conversions may therefore differ by up to two 16-bit LSBs.
        original = [sample[0] for sample in struct.iter_unpack("<h", source.readframes(48000))]
        converted = [sample[0] for sample in struct.iter_unpack("<h", decoded.readframes(48000))]
        maximum = max(abs(a - b) for a, b in zip(original, converted, strict=True))
        assert maximum <= 2, ("audio error exceeds the two dither passes", maximum)
        records.append({"oracle": "wave", "check": "sample rate, channels, frames and dither error", "maximum_16bit_sample_delta": maximum})
    native = root / "session.scraft"
    run(["run", "--demo", "--save", native, "--bounce", root / "mix.wav", "--start", "0", "--end", "1"], timeout=180)
    run(["info", native])
    with wave.open(str(root / "mix.wav"), "rb") as mix:
        duration = mix.getnframes() / mix.getframerate()
        samples = mix.readframes(mix.getnframes())
        assert 0.99 <= duration <= 1.01
        assert len(samples) == mix.getnframes() * mix.getnchannels() * mix.getsampwidth()
        assert any(samples), "rendered audio is silent"
        records.append({"oracle": "wave", "check": "complete non-silent bounced PCM audio", "seconds": duration})
    run(["info", fixtures / "invalid.png"], expected="error")


def canvas():
    native = root / "design.figcraft"
    run(["convert", fixtures / "shapes.svg", native, "--no-system-fonts"])
    run(["info", native, "--json", "--no-system-fonts"])
    run(["render", native, "-o", root / "design.png", "--no-system-fonts"])
    run(["export", native, "-o", root / "design.pdf", "--no-system-fonts"])
    image_ok(root / "design.png")
    pdf_ok(root / "design.pdf")
    run(["info", fixtures / "invalid.png", "--no-system-fonts"], expected="error")


status = "failed"
error = None
try:
    assert args.product in {"photo", "vector", "video", "develop", "motion", "publish", "write", "slides", "sheets", "cad", "audio", "canvas"}
    help_text = run(["--help"])
    assert "plervo" in help_text.lower()
    version = run(["--version"])
    assert args.expected_version in version, version
    run(["__plervo_unknown_command__"], expected="error")
    globals()[args.product]()
    for name, digest in fixture_hashes.items():
        assert hashlib.sha256((fixtures / name).read_bytes()).hexdigest() == digest, name + " input changed"
    status = "passed"
except Exception as exc:
    error = repr(exc)
    raise
finally:
    report = {"product": args.product, "status": status, "source": args.source, "binary": str(binary), "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(), "scope": "Representative headless file operations and failure handling; not complete feature or native GUI acceptance", "records": records, "error": error}
    (root / "result.json").write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n")

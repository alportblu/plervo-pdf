# OCR build-input recovery

Status: 2026-10-28. The original Ocrs model download endpoint returned HTTP 403
during Windows Store and Linux candidate packaging. Compilation and source
dependency checks had passed; these packages were not released.

The two unmodified models by Robert Knight are now mirrored in a dedicated
[build-input release](https://github.com/alportblu/plervo-pdf/releases/tag/build-inputs-ocr-2026-10-28).
They were recovered from the signed/notarized Plervo PDF alpha.7 public macOS
package (DMG SHA-256 `b2058695a3dfddb6e99a1dc7aeb193236ee0a7d8beec546696a2124ddce58034`).
Both match the original pins in `ATTRIBUTION.toml` exactly:

| File | Bytes | SHA-256 |
| --- | ---: | --- |
| text-detection.rten | 2510284 | `f15cfb56bd02c4bf478a20343986504a1f01e1665c2b3a0ad66340f054b1b5ca` |
| text-recognition.rten | 9716568 | `e484866d4cce403175bd8d00b128feb08ab42e208de30e42cd9889d8f1735a6e` |

Original source: [ocrs-models](https://github.com/robertknight/ocrs-models),
[model card](https://huggingface.co/robertknight/ocrs). The original download URLs
were `https://ocrs-models.s3-accelerate.amazonaws.com/text-detection.rten` and
`https://ocrs-models.s3-accelerate.amazonaws.com/text-recognition.rten`.

The original CC-BY-SA-4.0 license texts and attribution accompany the mirrored
files. Each license text retains SHA-256
`28a9529c7d0bb4dc51f4bf5c116a3d16ef247a052f7591466768ddf563fd1cf5`.
This is redistribution without model conversion, retraining or a license change.
The separately available Hugging Face detection export has a different hash;
it was not substituted for the qualified model.

Only build-time download URLs change. `cargo xtask models` still verifies the
original SHA-256 pins and refuses any mismatch. Models remain bundled for local
OCR; the application gains no new network request. The build-input release is
a prerelease with no app installers, is not marked latest, and is ignored by
the Suite's package/version selection.

Revision history: 2026-10-28 — record the outage, unchanged-file provenance and
new download location. Exact native package validation is required before an
application release using these URLs.

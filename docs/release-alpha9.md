# Alpha.9 release qualification

Status: preparation, October 30, 2026. This document is not a publication notice.

The next Mac Apple Silicon and Linux x86_64 release updates Suite, PDF, Photo,
Video and Develop. The other nine apps retain their separately qualified alpha.8
packages: Suite resolves the latest compatible release independently for each app.
The existing public assets must not be overwritten or relabeled.

- PDF includes the merged crypto-bigint timing-advisory correction.
- Photo and Video include Tokio 1.53.3; Video and Develop include the reviewed
  TLS provider and crypto-bigint updates documented in `products/SECURITY-REVIEW.md`.
- Suite keeps macOS disk-image installation serialized through copy and cleanup.
- The Windows Store edition, MSIX 1.0.1.0, adds the verified live CAD and Video
  acquisition links. Installation and updates remain Microsoft Store operations.
  Reserved, unpublished products remain unavailable for acquisition.

The version-only source changes preserve every dependency pin and original
third-party notice byte. Their inventory input hashes are refreshed only for
Cargo.toml/Cargo.lock version fields. Product overlays remain reproducible from
pinned upstream commits.

Publication requires the protected PR checks, native candidate and installation
checks, exact tested-payload comparisons, final malware scans, and macOS signing
and notarization. Store certification is a separate step after submission.
Representative testing does not establish all-feature correctness or absence of
vulnerabilities. Existing alpha limitations, RSA advisory exceptions and the
manual installer/restart step for Suite self-updates remain documented.

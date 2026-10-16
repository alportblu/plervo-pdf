# Candidate review — 2026-10-16

These twelve applications are not approved releases. The Suite catalog disables their
install actions. Candidate artifacts are for isolated testing, including when a later
dependency gate marks their CI job failed. No signing credentials are used by the workflow.

## Dependency changes

| Affected candidates | Change | Reason |
| --- | --- | --- |
| Photo, Video, Develop, Canvas | rustls 0.23.46 | RUSTSEC-2026-0346, 0347, 0348, 0349 |
| Photo | ruzstd 0.9.1 | RUSTSEC-2026-0344, decompression memory exhaustion |
| Develop, Publish, Slides | quick-xml 0.41.0; use its current attribute API | RUSTSEC-2026-0194, 0195 |
| Video, Develop | vendor rustls-rustcrypto 0.0.2-alpha; update rustls-webpki to the patched 0.103 series | RUSTSEC-2026-0049, 0098, 0099, 0104 |

The TLS provider overlay changes its webpki dependency and imports the existing ASN.1
algorithm identifiers from their new `rustls::pki_types::alg_id` location. It does not
implement or replace cryptographic algorithms. The original crate archive checksum is
`f12052947763ab8515f753315357599e9b0b4dab3b8ba15f30f725fe6d025557`.
MIT/Apache notices and a patch explanation are retained in both vendor directories.
The resulting source and lockfile are covered by each overlay's SHA-256.

Sources: [RustSec advisories](https://rustsec.org/advisories/),
[RustCrypto provider](https://github.com/RustCrypto/rustls-rustcrypto).

## Findings that remain open

- **Video and Develop:** `rsa` 0.9.10 still triggers
  [RUSTSEC-2023-0071](https://rustsec.org/advisories/RUSTSEC-2023-0071.html).
  Develop's `crates/fetch/src/http.rs::tls_config` explicitly disables client authentication.
  Video's `crates/speech/src/models.rs::agent` supplies no client certificate to ureq;
  ureq 3.4.2 selects `with_no_client_auth()` in that case. The observed application paths
  verify server public keys; they do not load an RSA private key. This is an applicability
  assessment, **not a patched RSA implementation**. No new advisory suppression was added.
- **Maintenance:** transitive `rustybuzz`, `ttf-parser` and `paste` remain in several
  graphs. They are tracked separately from known vulnerability advisories. Updating
  the SVG/PDF/font stack needs compatibility work; the current strict dependency gate
  continues to report them. Do not remove it just to obtain a green release.
- **Video:** cargo-audit also reports GLib's old `VariantStrIter` advisory in its
  all-platform lockfile. The desktop's `muda` dependency is macOS-only; verify the
  actual shipping target graphs before deciding applicability.
- Some lockfiles retain yanked dependency versions. Review replacements and their
  regression results before public release.
- Slides and Audio currently lack the complete OS-selected multilingual interface
  required for Plervo. The other source trees have differing translation coverage;
  translation presence is not proof of completeness. Develop and Publish now select
  the first supported OS language for a new profile and preserve saved selections.

## Validation and remaining release work

Local targeted tests passed for Photo's CLI conversions, Develop's fetch/TLS and
metadata/lens code, Video's speech/download code, Publish's DOCX/IDML import,
Slides' PPTX/ISO-BMFF code, Write's dictionary parser and Motion's model parser.
These are component checks, not an all-feature or all-platform acceptance claim.
Photo's first release smoke test preceded the dependency updates and must be repeated
on the exact candidate, with its pinned fonts and release features.

The untouched Slides baseline was run in CI and failed a Rust 1.95 Clippy diagnostic
in ISO-BMFF code. Its log is preserved separately. The adapted candidate fixes that
diagnostic and must independently pass all its own checks.

Still required: clean native installs/upgrades/removals, representative GUI workflows,
malformed-file and resource-limit tests, platform permission boundaries, locale and
accessibility checks, licensing/asset gates, signed macOS notarization, approved Windows
publisher signing, Linux distribution acceptance and qualified installation through
the Suite. A clean dependency audit alone cannot establish absence of vulnerabilities.

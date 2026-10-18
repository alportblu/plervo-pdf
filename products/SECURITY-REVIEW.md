# Product security review

Release status updated October 18, 2026: all twelve applications below, plus PDF, are
published for Mac Apple Silicon and Linux x86_64 in
[alpha.4](https://github.com/alportblu/plervo-pdf/releases/tag/v0.1.0-alpha.4).
The release notes and provenance identify the exact qualified packages and remaining
alpha limits. Windows candidates are internal; SignPath approval, signing integration
and final signed-package acceptance remain pending.

The review below records candidate findings and fixes through October 18. Its historical
failures are retained; they are not claims that every later package has the same status.
Workflow candidate artifacts are for isolated testing, including when a later dependency
gate marks their CI job failed. No signing credentials are used by that workflow.

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

## Scoped applicability review and remaining boundaries

- **Video and Develop:** `rsa` 0.9.10 still has
  [RUSTSEC-2023-0071](https://rustsec.org/advisories/RUSTSEC-2023-0071.html).
  The model-download paths use server-public-key verification, explicitly omit client
  authentication, and now install a provider that refuses every client private key.
  Both regressions pass a valid synthetic PKCS#8 key that the underlying provider accepts,
  then verify the application provider rejects it. RSA itself is **not patched**.
  The reviewed exception is generated only for these two products; it does not authorize
  RSA signing/decryption or apply to the other products.
- **Font maintenance:** Photo imports SVG with resvg 0.48.1; Publish's reviewed krilla-svg
  compatibility overlay uses the same maintained HarfRust/Skrifa stack. Write and Canvas
  remove unused Krilla simple-text features. Slides notes/footers and Vector's synthetic
  live-text fixtures now shape with HarfRust. Their format/export regressions passed.
  Develop's remaining `ttf-parser` use is ab_glyph watermark rasterization, limited to
  compile-time embedded Inter and pinned craft-fonts; it does not parse user fonts.
  This maintenance-only exception is generated exclusively for Develop, pending an
  upstream rasterizer migration. No global `ttf-parser` or `rustybuzz` exception exists.
- **Compile-time macros:** the paste and proc-macro-error maintenance exceptions cover
  trusted build-time procedural macros, not shipped runtime parsers. No known
  vulnerability is being described as fixed by these exceptions.
- **Linux D-Bus transition (reviewed 2026-10-17):** RUSTSEC-2026-0370,
  0371 and 0372 are explicitly informational maintenance notices for `zvariant`,
  `zbus_names` and `zvariant_derive`. Their code moved into zbus 6; they are not
  reports of a new exploitable flaw. The pinned AccessKit/AT-SPI accessibility
  dependencies still require zbus 5 and standalone types, as do some desktop-portal
  integrations. Updating only our direct zbus dependency cannot remove that chain.
  The review accepts these three exact maintenance IDs temporarily, preserves Linux
  accessibility and portal features, and leaves all other advisories enforced.
  Remove the exceptions when those upstream integrations migrate to zbus 6.
  No runtime library, lockfile, installed permission or security mechanism changed.
  Sources: [maintainer transition](https://github.com/z-galaxy/zbus/issues/2043),
  [zvariant notice](https://github.com/RustSec/advisory-db/blob/main/crates/zvariant/RUSTSEC-2026-0370.md),
  [names notice](https://github.com/RustSec/advisory-db/blob/main/crates/zbus_names/RUSTSEC-2026-0371.md),
  [derive notice](https://github.com/RustSec/advisory-db/blob/main/crates/zvariant_derive/RUSTSEC-2026-0372.md).
- `cargo xtask products --audit-policy PRODUCT` emits the common strict policy plus only
  that product's reviewed applicability exceptions. Regression coverage prevents an
  exception from spreading to another product. CI audits this generated policy.
- **Desktop automation:** Slides requires a private 256-bit capability, caps connections,
  requests and buffers, and never automatically replays a mutation after losing its reply.
  Seven MCP and two server tests passed; the authentication test failed with its guard
  removed and passed after restoration. Photo, Vector and Write already use authenticated
  transports. The other eight products' unauthenticated loopback UI control is now disabled
  in default/public builds, with a real-socket refusal regression. Normal GUI editing and
  headless CLI/MCP remain available. Their `local-control` Cargo feature is for isolated
  development only and must not be enabled in distributed artifacts. Broader review of
  those optional transports remains open; disabling them is not an authentication fix.
- **Languages:** Slides and Audio have English-only interfaces. Their alpha release notes
  must state that limitation; complete OS-selected multilingual interfaces remain product
  work, not a completed feature. Preparation continues on that explicit alpha basis under
  the current request to make the applications available. Other source trees differ in
  translation coverage. Develop and Publish select the first supported OS language for
  new profiles and preserve saved choices; 30/11 locale tests passed respectively.

## Validation and remaining release work

The completed native matrix for PR head `4da05339d4f6f6715d09f0622975b8164be7f480`
([run 39001993064](https://github.com/alportblu/plervo-pdf/actions/runs/39001993064))
passed 29 of 36 product/platform jobs. Suite/PDF CI, internal packaging and the
separate Linux Suite acceptance run passed. These results qualify neither changed
source nor a later binary: the next matrix must validate the refreshed overlays.

The following fixes address the seven remaining native results:

- Audio retains recovery data when its original and autosave have equal filesystem
  timestamps. A deterministic regression reproduced lost unsaved audio before the
  correction. All eight autosave and six recovery-interface tests pass afterward,
  with strict engine/UI Clippy. The earlier atomic-directory reservation also
  prevents same-millisecond session collisions. A timestamp tie may now offer a
  redundant recovery copy; it must never be used as proof that unsaved work is safe
  to delete.
- Video's Windows release executable prints informational flags without opening a
  modal dialog. Its previous release build and feature tests passed, but the
  version check waited for user input until the job timed out. Native executable
  checks now have a two-minute limit so that a regression fails promptly.
- Develop's offline-original regression waits for its deliberately blocked worker
  to start before asserting the probe ran. Frames remain subject to their existing
  responsiveness assertions throughout that bounded wait.
- Sheets' Linux acceptance recognizes the canonical X11 application class as well
  as a product title. The original installed app was alive with the normal `Book1`
  title; a captured-window regression verifies both identities and rejects another
  product or a partial class match.
- Canvas' live-share fixtures accept its own invite scheme and retain legacy-input
  coverage. Its credits dialog now fits the measured table, and rename shortcuts
  are exercised with the current platform's command modifier and F2. The complete
  local UI suite passed 304 tests; native Windows/Linux confirmation remains required.

Native packages already exercise DEB/MSI install, reinstall, repair where applicable,
and removal; macOS checks inspect the copied DMG bundle and standalone CLI. Linux
opens installed apps in clean Ubuntu 24.04, Debian 13 and Fedora 44 containers on
native x86_64 CI. Installed CLI file operations passed for Photo, Vector, Video,
Develop, Motion, Publish, Write, Slides, Sheets and CAD. This checks actual installed
executables, not only pre-package build outputs. Linux requires glibc 2.39 or newer
and the declared X11/ALSA runtime dependencies; it is not a promise of compatibility
with every distribution or older Ubuntu releases.

Native Linux screenshots reviewed so far show legible interfaces for Photo, Vector,
Video, Develop, Motion, Publish, Write, Slides and CAD. The earlier corrupted text
seen under AMD64 Docker emulation on Apple Silicon did not reproduce in these native
CI screenshots. Photo reports its software image-compositor fallback on the virtual
GPU; hardware acceleration, physical audio/MIDI devices and third-party plugins
still require device-specific testing. Startup screenshots are not full interaction
acceptance.

Other corrections verified in the completed native matrix include Publish's Windows
file-identity recovery guard, Audio's explicit-target plugin fixtures, Photo's batch
launcher, isolated Write PDF scaling checks with unchanged thresholds, full-version
macOS metadata and Vector's case-insensitive Windows notice packaging. Platform
installers retain their own product identifiers and compatible document formats.

The independent products retain their existing, unmodified OFL font inputs under
their own source-repository asset rules and attribution inventories. Publish's
instructions explicitly retain its existing Latin fonts; Develop explicitly permits
its pinned Noto CJK input. Vector and Write require open, attributed font sources.
These inputs are not embedded in PDF or Suite, whose stricter asset policy remains
unchanged. The previous request for a suite-wide font exception incorrectly applied
PDF's repository-local restriction to the separate products; no owner exception,
font substitution, proprietary font extraction, or licence change is required.
Each product's native packages must retain its original font notices.

Public release still requires final native acceptance and review of the exact
artifacts, macOS Developer ID signing/notarization, Windows SignPath approval and
signature verification, and Suite installation/update acceptance against the
published checksums. No unsigned candidate is described as a signed public release.
Security checks reduce specific demonstrated risks; they do not prove the absence
of every vulnerability or malware.

## Canvas recent-file test readiness — 2026-10-18

The native Windows run reached 303 passing UI tests but failed the first-run/recent-file transition: twelve rendered frames did not guarantee that its background metadata reader had returned. The test now waits up to five seconds for the inspected metadata state before checking the same visible labels and template removal. No assertion was removed or runtime behavior changed. Eleven home-screen tests and scoped Clippy passed locally with the pinned Canvas fonts. The follow-up native feature/regression tests passed on macOS, Windows and Linux.

The follow-up macOS packaging step then exposed stale PNG names. The packager now copies the catalog's verified Plervo Canvas ICNS directly, as the other product packagers do. Shell syntax and the actual ICNS decoding passed locally; a complete native package and mounted-bundle rerun is required before signing or publication. The failed original job remains recorded.

## Motion cache test isolation — 2026-10-18

The Windows engine run passed 611 tests but failed Object Matte disk persistence while other fixtures could replace the process-wide segmentation store. The existing test guard covered selected tests, but the shared disk-cache setup and settings fixtures could also install a store. Test builds now acquire that same guard at the configuration boundary; release behavior is unchanged. The existing persistence assertions remain, and the local engine suite passed 614 tests (two existing ignored tests), with strict scoped Clippy and byte-for-byte source reproduction. Native Windows confirmation remains required.

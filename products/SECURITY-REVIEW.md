# Candidate review — 2026-10-16

These twelve applications are not approved releases. The development Suite can select each product, but enables installation only
when a qualified matching release exists. No new product has been published yet. Candidate artifacts are for isolated testing, including when a later
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

The next overlay revision addresses failures found after the first full native run:
Photo's generated batch launcher now invokes the packaged Plervo executable; native
document extensions remain compatible in the Vector, Slides and Canvas installers;
Publish declares its actual document format, and Develop no longer exports an
unrelated inherited document type. Canvas uses its own invite-link registration while
accepting legacy links, preserving another app's Windows registration. Regressions
reproduced the old executable, rejected new link, and overwritten-registration failures.
Locale keys and menu labels are aligned, with explicit upstream credit retained.
Video contains optional effect-pipeline driver failures and retains the tested CPU
effect fallback. Required GPU effects and golden-scene comparisons passed locally
on Metal; the Windows FXC failure still requires confirmation on the corrected runner.
Slides maps the existing Symbol double-arrow code to its Unicode glyph when the
named font lacks the private-use alias. A regression using an open bundled font
failed before the mapping and passed after it; all 67 text-library tests passed.
Audio's recovery tests exposed a real autosave collision: sessions with the same
name created within one millisecond could reuse a retained recovery directory.
Directory creation now reserves a new path atomically and never reopens an existing
session folder. A deterministic same-clock-tick regression reproduced the overwrite
and verifies that discarding the second session preserves the first session's audio.
These changes still require the final overlay/CI run before release.

The next native run exposed Windows-specific Publish recovery failures: reading a
second handle to its exclusively locked file is refused by
[Windows file-lock semantics](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-lockfileex).
The correction
compares volume/file identities through the safe `winapi-util` API, keeping the lock
held and rejecting a different file even with identical contents. Existing native
recovery tests and a same-marker/different-file regression cover this boundary; final
Windows confirmation remains required. Audio's plugin test fixtures also now account
for the explicit Cargo build target used by the Windows runner. Their default and
explicit-target paths passed locally; production plugin behavior is unchanged.

Audio's macOS GUI and CLI now include Apple's
[`com.apple.security.cs.disable-library-validation` entitlement](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.security.cs.disable-library-validation),
which allows a plugin host to load libraries signed by other developers. An isolated
synthetic CLAP processing test failed under the previous hardened-runtime policy and
passed with that exception. CLAP and VST3 processing then passed under both the GUI
and CLI entitlement files. The GUI retains its microphone entitlement; the offline CLI
does not request microphone access. No JIT or executable-memory exception was added.
These tests use ad-hoc signatures on disposable copies, never a Developer ID key or
the owner's installed application. They cover native plugin processing, not editor
windows, arbitrary third-party plugins, or the final notarized distribution.

All twelve macOS packagers now accept `MACOS_NOTARY_PROFILE` with an explicit
`MACOS_KEYCHAIN`, using the same policy and submission function as the existing
Suite/PDF packager. The previous product scripts ignored that profile. Isolated
mock checks verify profile routing with spaces, accepted/rejected responses, and
refusal to notarize an ad-hoc identity; shell syntax passed for all twelve scripts.
These checks do not unlock a Keychain, access a signing key, or submit anything to Apple.

The mounted Sheets candidate exposed a packaging defect: `hdiutil makehybrid -hfs`
added Finder metadata that strict code-signature verification rejected. The eight
remaining packagers using that builder now use the existing Suite/PDF `create`
workflow and verify the app inside the mounted image. A disposable original app
reproduced the old signature failure; the corrected image and a normal copied app
both passed strict verification. No metadata was stripped to conceal a bad package.
These ad-hoc packaging checks do not qualify the final Developer ID artifacts.

The Windows installed-app check now waits for the exact GUI process and reads its
exit code through [`Start-Process -Wait -PassThru`](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.management/start-process).
It records both output streams and verifies the source version instead of relying
on the shell's potentially unset `LASTEXITCODE`. Native Windows confirmation is
still required; the Sheets installer failure remains in the earlier evidence.

Video's individual-selection command now honors the existing UI flag that suppresses
linked audio selection. Deleting the current selection preserves deliberately
unselected linked clips; explicit clip arguments retain their linked-edit behavior.
Both engine regressions failed before the correction and passed afterwards, along
with the offscreen Alt-marquee/Delete workflow and undo coverage.

Write's existing offscreen focus regression reproduced a lost first keystroke after
clicking a style gallery tile. The canvas input pass now honors its pending focus
request; the regression and all 386 UI tests passed (two existing ignored cases),
including the font-size field retaining its own editing focus. Word 2003 XML format
probing also now goes through the checked I/O module and its bounded regular-file
reader, preserving the existing automation read-root restriction.

Slides' interface fallback previously stopped at an oversized installed font, leaving
Chinese/Korean text as boxes despite suitable smaller fonts being available. A local
Mac reproduced the failure. The font database now continues past faces rejected by
the interface's existing 48 MiB and non-colour limits, without changing document
fallback or caching constrained misses globally. All 23 font tests, five IME/interface
tests and two UI-font tests passed locally, with strict scoped Clippy. Installed fonts
remain runtime-only; no font asset was added or copied into a package or screenshot.

Canvas' standalone server hardening fixture now handles ZIP 9's fallible entry name.
All 22 hardening tests and strict server Clippy passed locally. CAD's MCP description,
launch target and error messages now identify Plervo CAD; its four CLI connection tests
and ten MCP tests passed. Motion's three Windows path assertions now compare resolved
paths rather than drive or separator spellings; native Windows reruns remain required.

Representative debug CLI acceptance passed for all twelve products (125 recorded
checks) using original synthetic fixtures and independent image/document/audio readers
where applicable. These reports identify their binary hashes and inputs; they do not
qualify later release binaries. Develop's settled headless interface was also visually
reviewed, followed by offscreen Photo, Audio, Portuguese Sheets, Write, Publish and Motion
examples. These visual checks opened no native window and do not qualify installed
release GUI behavior. Failed harness assumptions remain recorded separately.

The candidate workflow now repeats those representative file operations against the
actual native release CLI on each platform. The portable harness was checked locally
with Write, Sheets and Audio, and its independent PDF reader accepted the five earlier
synthetic product PDFs. Its validation-only Python dependencies have exact wheel hashes
from the maintainers' PyPI releases: [Pillow](https://pypi.org/project/pillow/) (MIT-CMU),
[pypdf](https://pypi.org/project/pypdf/) (BSD-3-Clause),
[openpyxl](https://pypi.org/project/openpyxl/) and its et-xmlfile dependency (MIT).
They are not bundled with the applications. Final native release results remain required.

A fresh RustSec database (`64f92cdc64401fab47baa10e2e11fe382aa57155`) identified three
additional ZIP advisories (RUSTSEC-2026-0367, 0368, 0369) after the previous cached audit.
The affected products now require ZIP 9.0.4 and explicitly handle its fallible member-name
API. Video and Develop retain Candle 0.9.2 via a reviewed source overlay that updates ZIP,
adapts NPZ/PTH member-name decoding, and includes a synthetic Unicode NPZ regression.
All other Candle Rust source is unchanged; upstream licence and modification notices
are preserved and packaged. Fresh dependency policy checks passed for all twelve
products. Scoped document-format and inference compatibility checks passed, including
Unicode NPZ tensor names/values, eager/lazy NPZ reading and PTH member discovery.
Final native checks remain required. No ZIP advisory suppression was added.

All twelve Linux packagers now share the existing ELF-version guard with a declared
glibc 2.39 floor matching the Ubuntu 24.04 build runner. DEB and RPM metadata use that
same value; final binaries must still pass the guard. Package maintainer defaults use
Plervo contributors rather than inventing addresses in an upstream publisher's domain.
Video's TAR layout was corrected to contain only `bin/` and `share/`, matching the
Suite's existing archive policy; a legacy standalone install script remains in source
but is not included in that archive. The candidate workflow now runs the real Suite
installer against each Linux TAR, including detection, executable checks, reinstall,
removal and preservation of documents/preferences in an isolated temporary profile.
Final archive acceptance remains required; the production archive policy was not relaxed.

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


## Licence and source review — 2026-10-16

The initial shared policy omitted dependencies already used by these upstream apps.
The reviewed versions are now explicit exceptions, not a blanket MPL allowance:
Symphonia 0.5.5 and its codec/container crates, acadrust 0.6.4, imbl 7.0.2,
imbl-sized-chunks 0.2.0, and webpki-roots 1.0.9. Registry archive SHA-256 values
were checked against each Cargo.lock. Their sources are unmodified. Each affected
product's NOTICE now identifies the exact downloadable sources, authors, checksum,
and applicable licence text; the existing Mac/Linux/Windows packagers include NOTICE.
MPL-covered source rights are preserved separately from the application's MIT/Apache terms.
See [Mozilla's distribution guidance](https://www.mozilla.org/en-US/MPL/2.0/FAQ/#q8).

The source allowlist additionally names Motion's FilmCraft codec revision, Publish's
PdfCraft import revision, and its pinned winit fork. Those immutable revisions are
recorded beside the allowlist entries and remain covered by the overlay hashes.
No organization resources, proprietary code, or signing credentials are involved.

The CI cargo-deny invocation previously passed `--config` after `check`, which stopped
the tool before it could audit the graph. It now passes the global flag before the
subcommand. Native diagnostics remain separate; a failed dependency audit still fails
the candidate job and cannot qualify a public package.

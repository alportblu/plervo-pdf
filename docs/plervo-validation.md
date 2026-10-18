# Plervo PDF — pre-distribution validation record

**Historical snapshot: October 14, 2026, internal alpha.** The pending items below describe that checkpoint, not current download availability. For the current Suite and 13-app public alpha, see the [README](../README.md) and [alpha.4 release notes](https://github.com/alportblu/plervo-pdf/releases/tag/v0.1.0-alpha.4). This record was translated into English on October 18 without treating historical tests as evidence for newer packages.

A test counts as passed only when its result identifies the corresponding commit and package. Historical PdfCraft documentation does not qualify a Plervo release.

At `87ae2456`, Mac/Linux, WASM and dependency checks passed. Windows exposed a missing wait in the test rejecting GoToR links to directories/devices: it inspected the notification before the worker responded. The correction uses the existing bounded wait and preserves assertions for the refusal message and absence of a new tab; production behavior did not change. A new Windows run was still required. Embedded-font inspection found 55 missing characters in Telugu, 205 in Simplified Chinese and one in Traditional Chinese. The other 18 translated catalogs had character mappings, which does not certify font selection, shaping or linguistic quality. System fallback remains separate. Accented Portuguese, Spanish, French and German text survived creation and independent extraction in the final Mac package; OCR remained English-only.

## Prepared automation

- `ci.yml`: native tests, static analysis and builds on Windows, macOS and Linux; dependency/license audit, WASM build and separate fuzzing.
- `candidates.yml`: manual runs or packaging-related pull requests, extra fonts pinned by commit, release-mode CLI tests, local automation regressions and internal packages. No public release is created automatically.
- Windows builds and tests use `x86_64-pc-windows-msvc`, static CRT and mandatory PE resources. Packaging receives `-SkipBuild` and uses those executables instead of rebuilding a different version after testing.
- Existing CLI tests cover editing and saving, rotation and metadata after reopening, rendering, text extraction, malformed files, invalid options, paths outside the permitted root and preservation of existing files. Full CI also covers the interface and switching among 22 languages.
- Linux x64 uses Ubuntu 24.04 as its baseline and requires glibc 2.39 or newer. Packaging checks both executables' ELF requirements before creating packages; DEB and RPM declare the same minimum. Ubuntu 22.04 and Debian 12 are not targets for these candidates.
- macOS packaging mounts and verifies the DMG; Windows packaging inspects MSI tables and conditions. These checks **do not replace installation and use on a clean system**.
- The candidate pipeline installs, repairs and removes MSI packages, and installs/removes DEB packages on disposable runners. It checks the installed CLI, OCR-model discovery and preservation of synthetic documents. The standalone CLI is checked independently of the main installation. At this checkpoint, the new steps still needed to pass CI; they did not replace native GUI testing or an upgrade between two versions.

## Practical acceptance on October 14, 2026

- CI for `3cb65eda`: 3,051 macOS tests, 3,052 Windows tests and 3,064 Linux tests passed. Totals include shared cases across platforms, not distinct features. The subsequent fixes below required another CI run.
- Signed Mac candidate `c8b89cdc`: 17 engine-operation groups and five additional groups passed. They covered page organization, text, images, forms, comments, export, encryption, redaction, synthetic signatures, print-to-PDF layouts, bounded scripts and sanitization. All 163 commands rejected unknown arguments; this is input-validation coverage, not coverage of every behavior.
- Actual Mac interface: edit/save/reopen, fields, comments, search, pages, undo/redo, incorrect/correct passwords, malformed PDFs, OCR, themes, preferences after restart and switching all 22 languages. The control channel was enabled only for testing, authenticated and restricted to the app; documents were synthetic.
- Two discovered problems were fixed: missing OCR models in standalone Mac/Linux CLIs, and stale text reappearing in the editor after a content rewrite. OCR passed in the repackaged Mac file; the text regression failed before the fix and passed afterward. Local checks for that change passed 704 tests, with seven already ignored.
- The compiled CLI test also found that the in-memory item list remained stale after metadata correction. Text/image rewrites now refresh the list. The new regression failed before the fix and covers the active list, undo/redo and save/reopen; 437 engine/automation tests passed afterward, with three already ignored. These runs overlap and do not represent 1,141 distinct tests.
- Observed limits: missing Chinese characters without system fallback; English/unaccented-Latin OCR; incomplete linguistic and right-to-left review. Physical printing, visual Office inspection, public trust of the synthetic PDF-signing certificate and native Windows/Linux GUI acceptance had not been tested.

## Acceptance matrix at this checkpoint

| Stage | Required evidence | Plervo status on October 14 |
| --- | --- | --- |
| Code and security | Passing CI for the commit; dependency/license audit; limit, authentication and file-access regressions | CI for `3cb65eda` passed on all three systems; later fixes awaited CI |
| End-to-end PDF workflow | Create, open, edit, undo/redo, save, reopen, render and extract text; preserve originals; clear malformed-file errors | Local synthetic Mac workflow passed on the previous alpha; repeat on new candidates |
| Windows x64 | Install MSI on a clean machine/VM, open a PDF, save a copy, check shortcuts, repair and uninstall; preserve documents and the default PDF app | Pending |
| macOS Apple Silicon | Install from DMG in a clean environment; open, edit and save; check Gatekeeper and final package signature; remove the app while preserving documents | Internal `c8b89cdc` candidate signed/notarized and GUI exercised; clean installation and final-package repeat pending |
| macOS Intel | Build and run on Intel; repeat installation and PDF workflow | Pending; do not advertise validated support |
| Linux x64 | Install/remove DEB on Ubuntu 24.04 and RPM with glibc 2.39+; run the app, check menu integration, open/save a PDF and validate the portable package | `c8b89cdc` packages inspected; incorrect glibc minimum corrected in packaging; new packages and native installation pending |
| Upgrade | Install the previous candidate, change preferences/language, upgrade to a newer test version and verify preservation | Pending; requires two distinct packages |
| Languages | Persistent automatic/manual selection; 22 catalogs; check glyphs, clipping, dialogs and writing direction on each system | All 22 catalogs and pt-BR persistence checked on Mac; missing Chinese glyphs; full visual/linguistic review pending |
| Distribution | Signed packages where applicable; Mac notarization; scan exact packages, record SHA-256 and publish limitations | Pending for new candidates |

Use synthetic documents and test accounts. Record the system, architecture, commit, installer hash, steps and result. Shortcut, printing, permission and interface tests require native execution; cross-compilation or CLI checks alone are insufficient. A scan with no detections does not establish the absence of vulnerabilities.

## Website and installer publication

The static website could be published on Cloudflare Pages while downloads remained unavailable. After acceptance for each platform, publish its installers on GitHub Releases and configure only real website URLs. Optional support uses `https://buymeacoffee.com/alport`, with no payment details in the code.

Source was published in `alportblu/plervo-pdf` and the website in `alportblu/plervo-site`, exclusively under the personal account. Remote app CI was still under validation at this checkpoint; its results did not yet qualify distribution. The website was hosted on Cloudflare Pages. A Developer ID Application certificate had been issued to the authorized publisher, Maintech Solutions LLC; Mac candidate `c8b89cdc` passed strict signing and notarization checks. The owner had submitted the SignPath application and confirmed its receipt; Windows approval and integration remained pending. Each final package still required the signing and native checks above. Never add certificates, passwords or tokens to the repository.

For local Mac notarization, `packaging/macos/package.sh` accepts a `MACOS_NOTARY_PROFILE` created with `xcrun notarytool store-credentials`. When `MACOS_KEYCHAIN` is set, the profile must be in that Keychain with the signing identity. This supports authentication without placing an Apple password in the packaging command or repository. A notarization profile with ad-hoc signing is rejected; publication still requires an `Accepted` result, stapling and Gatekeeper verification. The existing three-Apple-variable path remains available for CI.

## Historical findings and follow-up runs

At commit `10a9374ff9058e9f0259bcbd7f920c7912290734`, the macOS/Linux, dependency and WASM jobs in [CI 38651175379](https://github.com/alportblu/plervo-pdf/actions/runs/38651175379) passed, and the Linux job in [candidate run 38651175487](https://github.com/alportblu/plervo-pdf/actions/runs/38651175487) produced packages. Windows was still running when evidence was collected. The later Windows-configuration change required its own passing run; these results were not evidence for the new binaries or native installation.

Subsequent runs found two test failures: parallel form tests removed each other's temporary directories, and the Windows PE-resource test still expected the old copyright text. The corrections preserve both PdfCraft and Plervo credits and isolate each temporary-directory creation. The isolation regression failed before the fix; all 23 form tests and the CLI suite passed locally afterward, along with Clippy, formatting, layer/asset checks and WASM checks across 31 crates (with pre-existing unused-code warnings). The PE-resource test runs only on Windows and still required updated CI. No failure was hidden or test disabled.

At commit `c8b89cdcc6f19881cea2be836a049ef0ac94b582`, the Linux job in CI 38658294918 failed three search/selection tests while still showing “Searching… 0/2” without a text layer. The helper could exit silently after 200 iterations. It now uses the shared helper's 30-second deadline, requires no pending renders and paints results before reading the accessibility tree. The same tests passed on Mac before the change; the observed failure was on remote Linux. Another Linux run was required to validate the correction.

After the adjustment, 1,113 Mac UI-crate tests passed, with five previously ignored cases, together with formatting, Clippy with warnings treated as errors, layer/asset checks and WASM. The first local run used `PDFCRAFT_SYSTEM_FONTS=0`, which conflicts with the test requiring font fallback. It was repeated with the default CI configuration, without changing the product or publishing screenshots with system fonts.

Mac Apple Silicon package `c8b89cdc` was notarized on the evening of October 14, 2026, local time: the app, DMG and CLI ZIP received `Accepted`, with no issues in Apple's logs. App/DMG tickets were stapled and validated; strict signing, Gatekeeper, `syspolicy_check distribution` and the CLI notarization requirement passed. Seven synthetic checks on the extracted CLI passed, including malformed-PDF refusal and path confinement; ClamAV scanned the final artifacts with no detections. This does not prove the absence of vulnerabilities or first launch on a clean machine. Hashes of the internal candidates, unpublished at that checkpoint:

- DMG: `8ec03466ee72735aa61515a2e164d63d21a80402fa2cd5f1255c29459f786be4`.
- CLI ZIP: `6ee71ca539f6df439b372f81fb4622e981af797866d9add2b4a59ed771868cc0`.

Linux inspection found no ClamAV detections in the four `c8b89cdc` candidate files. Tarballs contained no escaping paths, special links or files with setuid, setgid or world-write permissions. ELF inspection found `GLIBC_2.38` and `GLIBC_2.39` requirements even though the DEB declared 2.35. The verifier rejected those binaries under the old minimum before creating the package tree. The declared baseline is now 2.39, shared by verification and DEB/RPM metadata. Seven local checks covered accepting the real ELF files under 2.39, rejecting them under 2.35 and 2.9, invalid/missing files and invalid arguments; syntax, ShellCheck and actionlint passed. Those old candidates must not be published; corrected packages must be rebuilt and inspected. This was static verification, not local Linux execution or installation.

# Plervo Suite

**13 open-source desktop apps for documents, design and media. One place to install and manage them.**

Start with **Plervo Suite**, then choose the apps you need from its catalog. The Suite installs, opens, updates and removes each app. Plervo is maintained by **AlporT**, with optional community support.

[Download Plervo Suite](https://plervo.com) · [Release notes](https://github.com/alportblu/plervo-pdf/releases) · [Support on Buy Me a Coffee](https://buymeacoffee.com/alport)

## The apps

| App | Purpose |
| --- | --- |
| Plervo PDF | PDF editing and document tools |
| Plervo Photo | Raster image editing |
| Plervo Vector | Vector illustration |
| Plervo Video | Video editing |
| Plervo Develop | RAW photo development |
| Plervo Motion | Motion graphics |
| Plervo Publish | Page layout |
| Plervo Write | Word processing |
| Plervo Slides | Presentations |
| Plervo Sheets | Spreadsheets |
| Plervo CAD | Technical drawing |
| Plervo Audio | Audio editing |
| Plervo Canvas | Interface design |

This repository contains the Suite, PDF, and the pinned source catalog and overlays for the other twelve apps. See [product sources and build instructions](products/README.md).

## Downloads and platform support

**Public downloads are experimental alpha releases, not stable software.** The Suite selects the latest qualified release separately for each app; app and Suite version numbers can differ.

| Platform | Availability |
| --- | --- |
| macOS, Apple Silicon | Suite and all 13 apps. Signed by Maintech Solutions LLC and notarized by Apple. |
| Linux, x86_64 | Suite and all 13 apps. DEB/RPM installers; glibc 2.39 or newer. |
| Windows | [Plervo PDF is available in Microsoft Store](https://apps.microsoft.com/detail/9PBWKBZRBJNR). Suite and other Windows apps remain internal candidates; they are not public signed downloads. |

1. Download the Suite for your computer from [plervo.com](https://plervo.com).
2. Install and open **Plervo Suite**.
3. Select an app in the catalog and choose **Install**.

Linux app packages were exercised on Ubuntu 24.04, Debian 13 and Fedora 44; the Suite also passed Ubuntu 26.04 acceptance. Linux Mint 22 shares the Ubuntu 24.04 base but has not been tested separately. macOS Intel, Linux ARM and native Wayland sessions are not qualified.

The alpha has known limits. Final Mac GUI acceptance, installation on a clean Mac and broader hardware coverage remain pending. Starting with Suite alpha.8, an automatic check at startup and every six hours while open shows available Suite and app updates. The Suite update button downloads the appropriate Mac DMG or Linux DEB/RPM; install that package and reopen Suite. Checks can be switched to manual in Settings. Existing alpha.5 users need to install this newer Suite once before they receive these notices. Check the [release notes](https://github.com/alportblu/plervo-pdf/releases) for available packages and their validation and limitations.

## Languages

The Suite follows the **operating system's display language** on a new profile and offers English, Portuguese and Spanish. A saved manual choice takes priority.

PDF retains the upstream's 22 interface language catalogs. Choose **Edit → Preferences → Interface language** to select a language or use automatic system detection. Your choice is saved; switching languages preserves open documents and automation commands. New Plervo screens and actions must use the same translation system.

Translation coverage varies between apps, with English fallback for untranslated text. **Slides and Audio are English-only in this alpha.** Full linguistic review, script/font coverage and right-to-left layout remain ongoing work. See [localization](docs/localization.md) and [language limitations](docs/localization-parity.md).

## Security and privacy

The Suite verifies official release origins, package versions and SHA-256 checksums before installation. Mac packages also require the expected publisher signature and Gatekeeper assessment. Linux installs each app for the current user under `~/.local/share/plervo-suite/apps/plervo-<product>/current` (respecting `XDG_DATA_HOME`), without an administrator password. Its installer rejects links, unsafe paths and oversized archives, and replaces the verified payload atomically. Removing an app preserves documents and preferences.

Automation is opt-in. Donations open an external support page only when you choose the link; the apps contain no payment SDK or financial credentials. Support is always optional.

Read the [security notes](SECURITY-PLERVO.md) or [report a vulnerability privately](https://github.com/alportblu/plervo-pdf/security). Tests, package scans and notarization do not guarantee the absence of vulnerabilities.

## Development

Rust 1.95. System dependencies are documented in [.github/workflows/ci.yml](.github/workflows/ci.yml).

```sh
cargo build --locked -p plervo-suite -p plervo-pdf -p plervo-pdf-cli -j 2
cargo run -p plervo-suite
cargo run -p plervo-suite -- --status
cargo run -p plervo-pdf
cargo run -p plervo-pdf-cli -- --help
cargo test --locked -p pdfcraft-ui-egui --lib control::transport_tests -j 2
cargo xtask assets
```

Internal PDF crates retain their `pdfcraft-*` names to support upstream tracking and future fixes. Plervo has separate public branding, executables and settings. It does not automatically migrate or modify PdfCraft/PrintCraft data.

Candidate workflows produce internal artifacts; they do not publish releases automatically. Native acceptance and macOS/Windows publisher signing are required before public distribution.

The [Microsoft Store build](docs/microsoft-store.md) uses MSIX and Store-managed updates. It does not require the Suite or a separately purchased Windows certificate. An internal test package is not a certified public download.

## License and credits

Plervo Suite adapts installation components from [ArtCraft Launcher 0.4.1](docs/suite-provenance.md). Plervo PDF is based on [PdfCraft 0.8.0](https://github.com/storytold/pdfcraft/tree/v0.8.0). The other apps' upstream sources are listed in the [product catalog](products/README.md). Original authors' credits and license notices are preserved.

MIT OR Apache-2.0, as described in [LICENSE-MIT](LICENSE-MIT), [LICENSE-APACHE](LICENSE-APACHE) and [NOTICE](NOTICE). Third-party assets are documented in [ATTRIBUTION.md](ATTRIBUTION.md).

Plervo is independent and is not affiliated with Adobe or the ArtCraft team. This distribution includes no Adobe or ArtCraft brand artwork.

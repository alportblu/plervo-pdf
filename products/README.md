# Plervo applications

These twelve independent forks join Plervo PDF in the **13-app Plervo Suite**.
All thirteen apps have published packages for Mac Apple Silicon and Linux x86_64.
The [release notes](https://github.com/alportblu/plervo-pdf/releases) identify the
qualified version of each app; the Suite selects releases independently per app.
Windows candidates remain internal while publisher signing and final signed-package
acceptance are pending. Start with [Plervo Suite](https://plervo.com) to install apps.

This directory records reproducible source inputs and candidate-build requirements.
Passing a compile check does not qualify an app, installer, or platform for release.
Current findings and remaining gates are recorded in [SECURITY-REVIEW.md](SECURITY-REVIEW.md).

The development catalog targets alpha.5: visible optional Support actions in the
application headers, localized through each app's existing language catalogs.
All support actions point to https://buymeacoffee.com/alport and require a click.
Earlier alpha.4 packages remain unchanged. New packages are published separately
after native qualification, signing where required, and installation checks.

| Product | Purpose | Public upstream |
| --- | --- | --- |
| Plervo Photo | Raster image editing | storytold/photocraft |
| Plervo Vector | Vector illustration | storytold/vectorcraft |
| Plervo Video | Video editing | storytold/filmcraft |
| Plervo Develop | RAW photo development | storytold/lightcraft |
| Plervo Motion | Motion graphics | storytold/effectcraft |
| Plervo Publish | Page layout | storytold/designcraft |
| Plervo Write | Word processing | storytold/wordcraft |
| Plervo Slides | Presentations | storytold/deckcraft |
| Plervo Sheets | Spreadsheets | storytold/gridcraft |
| Plervo CAD | Technical drawing | storytold/cadcraft |
| Plervo Audio | Audio editing | storytold/soundcraft |
| Plervo Canvas | Interface design | storytold/figcraft |

## Reproduce a candidate

`sources.json` records immutable source and font commits, the source-overlay SHA-256,
the original icon inventory, and separate Windows installer identities. There are
no credentials or signing keys in this catalog.

1. Check out the product's public upstream at its exact `commit` in a clean directory.
2. From this repository, run `cargo xtask products --prepare photo /absolute/clean-checkout`
   (replace `photo` with the desired product ID).
3. Use the resulting checkout's native build and packaging commands. The
   `Plervo product candidates` workflow applies the same overlay on macOS, Linux,
   and Windows, then builds internal artifacts with provenance and a qualification warning.

The preparation command refuses dirty checkouts, unknown products, mismatched
source/patch/icon hashes, traversal, and linked asset destinations. It stages the
prepared snapshot for upstream asset-inventory gates, without creating a commit. It never
downloads, installs, signs, or publishes an application. A candidate using the
`0.1.0-alpha.5` version is not necessarily the qualified public package. The release's
provenance and checksum manifests identify the exact published artifacts.

## Development validation

The Suite manages all 13 canonical product identities, with release metadata,
platform, hash and signing checks kept per product. Buttons become installable only when a
matching public release exists for the current platform.
The local Suite management gate passed 103 tests; native product and installer acceptance
are separate requirements.

For a focused native rerun, manually dispatch `Plervo product candidates` with a
`product` choice. The default and pull-request runs still validate all twelve
products; an unknown selection fails instead of silently producing an empty matrix.

The candidate workflow additionally exercises installed DEB/MSI binaries, Linux
reinstallation, Windows repair, and removal on disposable runners. Clean Ubuntu 24.04,
Debian 13 and Fedora 44 containers test DEB/RPM installation, reinstallation and removal, recording
the exact container digest alongside the installed executable checks. macOS candidates
are copied out of their DMGs and checked alongside the separately extracted CLI.
The clean Linux containers also open the installed GUI in a fresh, unprivileged Xvfb
session and retain a screenshot for visual review. This catches runtime libraries
missing from package metadata without using the runner's already-provisioned desktop.
The native Linux runner repeats file-operation acceptance with the installed DEB
CLI, so packaging changes are exercised beyond executable version/help checks.
The actual Suite detector must recognize each mounted Mac bundle's exact release
version, including its alpha suffix.
Each Linux TAR candidate is also installed, detected, reinstalled and removed by the
Suite's real manager in a temporary profile. The helper checks both installed executable
versions, CLI help and preservation of sentinel documents and preferences. It is a CI
example, not a public application or an alternative production release source.
The same workflow runs representative native release CLI operations with original
synthetic files: conversions, native-format round trips, document/audio rendering,
spreadsheet values and expected failures. Independent Pillow, pypdf, openpyxl and
Python standard-library readers inspect the outputs. Their validation-only environment
uses exact versions and wheel hashes in `acceptance-requirements.txt`; none of these
Python packages is included in a Plervo application. Reports retain binary/input hashes.
These internal checks do not establish publisher signing, notarization, clean-machine
Gatekeeper acceptance, or a complete GUI workflow. Installation evidence is retained
separately from the packages, including when a subsequent gate fails.

Video's macOS qualification runner requires the GPU effect stage. Direct stage
comparisons report an unavailable optional stage on other adapters; composited-output
and explicit CPU-fallback comparisons still run. The previously recorded untouched
Slides baseline can be repeated through the manual workflow input; normal candidate
runs validate the adapted source without recompiling that unchanged baseline.

Generate the product-scoped dependency policy with
`cargo xtask products --audit-policy PRODUCT`; the CI uses the same policy. Eight desktop
candidates disable the unauthenticated UI control transport by default. Their development-only
`local-control` feature must not be enabled in a public build; headless CLI/MCP still works.

## Release requirements

- Preserve upstream notices and asset licences; review all new dependency advisories.
- Verify native operations with synthetic documents, damaged-input handling, and
  filesystem/network boundaries. Keep automation opt-in and authenticated.
- Verify UI, OS-language selection, manual-language persistence, and missing-glyph behavior.
- Test clean install, upgrade, coexistence, and uninstall without removing documents.
- macOS: verify Developer ID signature, notarization, and clean-machine launch.
- Windows: require the selected SignPath publisher-signing approval and installer checks.
- Linux: verify the declared glibc minimum on supported distributions, DEB/RPM,
  and installation through the Suite.
- Integrate each qualified product into the Suite before advertising its download.

The Windows packages in this workflow are unsigned and the Mac packages are not
notarized. They must not be promoted automatically to public releases.

For an approved macOS candidate, the product packager accepts the existing
`MACOS_SIGN_IDENTITY`, `MACOS_KEYCHAIN` and `MACOS_NOTARY_PROFILE` settings used by
the Suite. The same Developer ID Application identity can sign each product, but
each application and its distribution artifacts require their own verification
and notarization. No signing credentials are stored in this catalog or workflow.

## Identity and attribution

The code retains the upstream MIT/Apache notices. Native document formats and
internal engine crate names remain compatible; the Plervo application identity,
settings locations, links, installers, and artwork are independent.
Original Plervo artwork is in `assets/products`, with hashes and licence records
in `ATTRIBUTION.toml`. The suite's original green identity is shared, with a
different symbol and accent color for each function. Optional support points to
https://buymeacoffee.com/alport.

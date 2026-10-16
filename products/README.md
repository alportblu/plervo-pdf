# Plervo application candidates

These twelve independent forks are **internal candidates**, not public downloads.
The currently published Suite still distributes the qualified PDF releases only.
Passing a compile check does not qualify an app, installer, or platform for release.
Current findings and remaining gates are recorded in [SECURITY-REVIEW.md](SECURITY-REVIEW.md).

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
downloads, installs, signs, or publishes an application. Internal candidates use
`0.1.0-alpha.4` as a development version; that version is not a public release promise.

## Development validation

The development Suite manages all 13 canonical product identities, with release metadata,
platform, hash and signing checks kept per product. Buttons become installable only when a
matching public release exists. The published Suite has not changed during preparation.
The local Suite management gate passed 103 tests; native product and installer acceptance
are separate requirements.

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

## Identity and attribution

The code retains the upstream MIT/Apache notices. Native document formats and
internal engine crate names remain compatible; the Plervo application identity,
settings locations, links, installers, and artwork are independent.
Original Plervo artwork is in `assets/products`, with hashes and licence records
in `ATTRIBUTION.toml`. The suite's original green identity is shared, with a
different symbol and accent color for each function. Optional support points to
https://buymeacoffee.com/alport.

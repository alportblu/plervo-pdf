# Distribution notices

`THIRD_PARTY_NOTICES.txt` is a reviewed, frozen inventory for the PDF/Suite workspace.
It covers the union of native targets and optional features, including build dependencies;
it is deliberately broader than the libraries linked into any single executable. It does
not cover the separately built sibling application repositories.

The initial inventory was collected with cargo-about 0.9.2, `generate --workspace
--all-features --locked --fail --format json`, ignoring development-only dependencies,
for macOS arm64/x64, Windows x64 and Linux arm64/x64. Original license texts missing
from published crate archives were retrieved from the exact `.cargo_vcs_info.json`
revision. Compound notices, bundled font licenses, Adobe's non-visual CMap data license,
and the existing asset notices were included separately. The manifest records source
URLs, hashes, package versions and the five upstream crates that only supply a license
declaration. Their published authors and declared standard license are preserved without
inventing a copyright year. Original project notices and dual licensing remain intact.

`cargo xtask licenses` checks the notice digest and all reviewed dependency inputs
(UTF-8 text with CRLF normalized to LF for Windows checkouts).
Packaging fails if dependencies or notices have changed. For an update, regenerate the
inventory, inspect original upstream texts (a scanner's generic SPDX text is not a
replacement for an upstream copyright notice), preserve additional NOTICE/asset terms,
then update the text and manifest together after review. Do not simply rehash stale text.

This distribution inventory is technical evidence, not a blanket legal opinion or an
assertion that every third-party component uses the project's MIT/Apache license.

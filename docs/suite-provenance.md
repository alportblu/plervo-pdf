# Plervo Suite implementation and provenance

Status: experimental alpha implementation. Publication and artifact status are recorded in the [release notes](https://github.com/alportblu/plervo-pdf/releases). Updated 2026-10-15.

The install engine is adapted from the MIT/Apache-2.0 source of [storytold/craft-launcher](https://github.com/storytold/craft-launcher/tree/0f295d4b9c90d15c2e38e36c322bdbc0fa943a54), version 0.4.1, commit 0f295d4b9c90d15c2e38e36c322bdbc0fa943a54. Original authors retain their copyright. The root MIT and Apache licences apply to these derived modules. No upstream trademark icon or screenshot is bundled. The upstream product names appear only in preserved synthetic regression fixtures and attribution.

The installed universal launcher was inspected read-only with REA: SHA-256 3b18b3163b3a57b07e12dc7078def0316dfd52a189104e37f6ea572d11766141, version 0.4.1, signed by Learning Machines LLC (DJ6XS33FX8). The extracted ARM64 analysis copy is 32526e4bebf43bba7a8776c7abaacfe6adc8bcabefc077db60fe5f436c3a287e. REA inventoried Mach-O dependencies, metadata, imports and exports; it located the signer-verification and filename-validation symbols. Deep Ghidra analysis exceeded its 330-second limit twice (inventory, then one targeted procedure); no successful decompilation is claimed. Reconstruction is grounded in the matching open source, supported by the packaged artifact inventory. Private evidence is stored outside this repository.

## Scope and intentional changes

- Original Plervo native UI, with English, Portuguese and Spanish. Auto is the default and resolves the operating system's preferred UI languages. User overrides persist separately from the PDF preferences.
- Only Plervo PDF is in the shipped catalogue; identifiers and credentials never target an upstream app or an organization account. The Suite and PDF can coexist with ArtCraft/PdfCraft.
- No ArtCraft catalogue, screenshots service, telemetry, login item, tray daemon, required GitHub credentials, control listener or background updater. Public release checks use the personal alportblu/plervo-pdf repository. Optional donation opens buymeacoffee.com/alport only after a click.
- Reuse of platform selection, bounded HTTP transfer, SHA-256 verification, retries, atomic replacement/rollback and native installer code. The public installation path is restricted to qualified macOS ARM64 packages until Windows/Linux acceptance and signer checks are ready.
- Mac installations require the expected bundle identifier/version, Maintech team 52527829PM, strict signature validation and Gatekeeper assessment before replacement. Full pre-release versions must match exactly. A failed move to Trash leaves the app installed, never permanently deletes it. Staging uses a unique temporary directory; occupied unrecognized destinations are preserved. Existing installations are revalidated before launch/removal/update; removal moves only the app to Trash and leaves documents/preferences intact.
- No self-update replacement yet: the Suite manages PDF updates, while a new Suite version is obtained from the official site. This limitation is explicit in release notes.
- `plervo-suite --check`, `--status`, `--install-pdf`, `--open-pdf`, `--uninstall-pdf` exercise the same engine as the GUI. Mutating commands and the GUI use an OS file lock. Never pass arbitrary source URLs or destinations.

## Validation

Core/native unit tests preserve upstream security cases: corrupt/resumed downloads, cancellation, rollback, signatures and version mismatch. Plervo adds catalogue confinement, missing/invalid checksums, limits and path checks. GUI tests check all three languages and cancellation of uninstall. The screenshot example renders the actual UI without global keyboard/mouse input.

Linux helper execution retries only `ExecutableFileBusy` before a program starts, within the original timeout. A deterministic Linux regression holds a write handle briefly, then releases it; this failed before the fix. A permanently busy executable remains refused. Permissions failures and failures after a helper starts are not retried.

The PDF locale regression was observed failing with shell Portuguese ahead of display English. Auto now prioritizes the OS display language on macOS/Windows; Linux still follows locale environment settings. Local testing does not establish native Windows/Linux release readiness.

# Video and CAD in Microsoft Store

These MSIX packages reuse the pinned public source and Plervo overlays in
`products/sources.json`. `prepare.py` applies the additional Store overlay and
records the source, patch, lockfile, font and original license hashes. Video's
TLS certificate dependencies are updated for RUSTSEC-2026-0382/0383. No new
advisory exception is introduced. CAD uses the existing reviewed source unchanged.

The Store identities in `apps.json` were read from the personal publisher's
Partner Center account on 2026-10-21. Reserving an identity is not certification
or public availability. The apps are independent of Adobe, Autodesk and ArtCraft.
Their source licenses remain **MIT OR Apache-2.0**, with separate notices for
fonts and other dependencies. Original MIT/Apache texts must remain byte-identical
to the pinned upstream, and the packages carry NOTICE and asset/vendor notices.

Run `.github/workflows/apps-store.yml` to qualify both apps. It checks source
formatting, boundaries, asset attribution, dependency policies, full Windows
workspace lint/tests and release CLI operations with independently decoded output.
It then installs the exact MSIX payload on a disposable Windows runner, verifies
file hashes, renders a saved project/drawing through the installed CLI, opens and
captures the installed GUI in EN/PT/ES, checks for default TCP listeners, performs
a higher-version MSIX update, and uninstalls without changing user documents or
the default file handler. Screenshots still need a human/agent visual review;
this is not a claim of all-feature, hardware or Microsoft Store delivery testing.

The desktop `local-control` feature is excluded. Video's optional Whisper and
neural-voice downloads are also excluded from these default-feature packages.
CLI/MCP remains opt-in. The only manifest capability is `runFullTrust`, required
for these local desktop applications. Updates are delivered by Microsoft Store;
these apps do not contain an application self-updater. The public CLI entry is
visible in Start, matching the accepted PDF packaging configuration.

`package.ps1 -Product video -ValidationOnly -ManifestOnly` checks SDK semantics
before compilation. Temporary inert probes are never installed or uploaded.
Real validation packages use Microsoft's package-scoped unsigned testing OID;
no trusted root, Developer Mode or SmartScreen changes are made. **Never publish
validation-only packages.** The actual-identity package is emitted only after
native acceptance and compared file-for-file with the tested payload, excluding
the identity manifest. Both remain internal artifacts until final review,
malware scan, protected-branch merge and Partner Center submission.

Store tiles are resized from the existing attributed Plervo product icon.
`Licenses/STORE-ASSETS.json` records each derived tile, its source, hashes,
authorship, license and transformation. Native screenshots contain the app's own
client area and original synthetic documents; OS title bars and desktop artwork
are excluded. Listing publication must preserve the corresponding attribution.

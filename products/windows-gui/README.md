# Internal Windows GUI diagnostics

This manual workflow opens only the exact native-passed MSI packages in
`candidates.json`, on a disposable GitHub Windows runner. It verifies the native
job, artifact source and SHA-256, and the MSI's own pinned SHA-256 before installing.
The download token is scoped to the fetch step; the GUI step receives no GitHub token.

Each app must expose a responsive native window, remain alive for a capture of its
client area, and close normally. Installation is removed afterward, including on
failure. Failures are collected per app so the remaining diagnostics still run.
Screenshots require visual review; startup does not establish complete feature,
physical hardware, upgrade, publisher-signature or public-release acceptance.
Nothing is published and consumer trust settings are not changed.

The initial pinned set contains twelve apps. Motion is added only after its exact
corrected Windows candidate passes native qualification. Existing failed runs
remain recorded. Reproduction uses the pinned workflow/source commit and artifact
digests, not a mutable latest download.

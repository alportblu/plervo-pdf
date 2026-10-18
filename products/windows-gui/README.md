# Internal Windows GUI diagnostics

This workflow opens only the exact native-passed MSI packages in
`candidates.json`, on a separate disposable GitHub Windows runner per app. It verifies the native
job, artifact source and SHA-256, and the MSI's own pinned SHA-256 before installing.
The download token is scoped to the fetch step; the GUI step receives no GitHub token.

Each app must expose a responsive native window, remain alive for a capture of its
client area, and close normally. Installation is removed afterward, including on
failure. Failures are collected per app so the remaining diagnostics still run.
Screenshots require visual review; startup does not establish complete feature,
physical hardware, upgrade, publisher-signature or public-release acceptance.
Nothing is published and consumer trust settings are not changed.

Window positioning uses asynchronous cross-thread requests after a bounded
responsiveness check. Each phase is logged, and the summary is saved after each
app. The GUI step has its own timeout so partial diagnostics can still upload
before the job's overall deadline. MSI starts directly with an argument list,
without shell activation or a recursive service-process kill. Earlier failed or
stalled sequential runs are retained; isolated success alone does not prove
coexistence or explain a previous installation failure.

The initial pinned set contains twelve apps. Motion is added only after its exact
corrected Windows candidate passes native qualification. Existing failed runs
remain recorded. Reproduction uses the pinned workflow/source commit and artifact
digests, not a mutable latest download.

The Windows matrix repeats the exact pinned batch IDs so a separate Ubuntu
selector cannot hold up these diagnostics. A follow-up batch must update both
that list and `candidates.json`; each runner still validates its pinned candidate.

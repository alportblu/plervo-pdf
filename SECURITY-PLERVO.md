# Plervo security notes

Updated October 20, 2026. Plervo Suite and all 13 apps are available as **experimental alpha** releases for Mac Apple Silicon and Linux x86_64. SignPath Foundation declined the project; standalone PDF is now being prepared for Microsoft Store distribution. Certification and publication are pending. Exact package evidence and limitations are recorded in the [release notes](https://github.com/alportblu/plervo-pdf/releases).

Report suspected vulnerabilities through [GitHub's private security reporting channel](https://github.com/alportblu/plervo-pdf/security). Do not include personal documents, credentials or private keys in public issues.

## PDF hardening

The PDF fork starts from PdfCraft v0.8.0, commit `9931329b9fae0b05373b4dc3ac7c58541f0891a4`. Its initial security changes include:

- `rustls` 0.23.45 → 0.23.46, addressing the four RustSec advisories identified in the October 14, 2026 audit.
- Local UI automation remains opt-in (`--control`), binds to `127.0.0.1`, uses a random token per session and writes an owner-only control file.
- At most 16 active connections; each slot is returned when its handler finishes, including thread-creation failures.
- Authentication must finish within five seconds. Blank lines and slowly sent bytes do not extend that deadline.
- Authenticated requests are limited to 1 MiB and a 30-second read deadline; the pre-authentication limit is 4 KiB.
- Writes have a five-second timeout. A call that receives no UI response within 30 seconds closes the connection.
- The automation queue is capped at 32 requests; saturation returns an error without blocking the interface.
- Separate application identity, settings directory, MSI UpgradeCode and URLs. The fork does not migrate PrintCraft/PdfCraft data.
- Optional support opens a fixed HTTPS link only after a user action. No payment widget or secret is embedded in the app.
- AppImage packaging does not automatically download or execute a tool without a verified version and hash.
- Project workflows use read-only default permissions and actions pinned by commit SHA. Candidate builds do not publish releases.

Other apps have separate dependency and automation reviews in [products/SECURITY-REVIEW.md](products/SECURITY-REVIEW.md). A PDF-specific mitigation is not evidence that every app uses the same implementation.

## October 20 certificate-validation updates

The Store preparation CI rejected the previous lockfile for `RUSTSEC-2026-0382`
and `RUSTSEC-2026-0383`, concerning distrusted emSign certificate intermediates.
The root lockfile now uses `rustls-webpki` 0.103.16 and `webpki-root-certs` 1.0.10,
the upstream fixed versions. See the [webpki correction](https://github.com/rustls/webpki/pull/546)
and [root-certificate correction](https://github.com/rustls/webpki-roots/pull/133).
No advisory exception was added. The Store candidate must be rebuilt from this
lockfile and pass the dependency gate. Previously published binaries and the
separate product source trees do not inherit a lockfile change retroactively.

## RSA: remaining advisory and preserved mitigation

The recorded `RUSTSEC-2023-0071` advisory affects private-key RSA operations in the `rsa` library; the database consulted for the release review did not list a fix. The PDF code uses that library only for public verification (`crates/sign/src/rsa_pad.rs`, `ec512.rs`). Native private-key operations use `aws-lc-rs` or system providers; the browser build refuses the private RSA path. The documented exception remains in `deny.toml`, without hiding the finding from raw `cargo audit` output.

This does not establish the absence of other flaws. Future dependency or signing changes must reassess this mitigation. Do not import `rsa::RsaPrivateKey` or replace the native providers with private-key operations from that library. Other reviewed maintenance exceptions are recorded in `deny.toml` and the product security review.

## Suite installation boundaries

The Suite accepts release metadata and packages from the official personal repository. It checks product identity, platform, version and SHA-256 before installation. Mac installation additionally checks the expected publisher signature and Gatekeeper assessment. Linux extraction is bounded and rejects links and unsafe paths; installation replaces the verified per-user payload atomically. Documents and preferences stay outside the managed payload and are preserved during removal. See [Suite implementation and provenance](docs/suite-provenance.md).

The Suite reads public release metadata at startup and on request. Installing, updating or removing an app requires a user action. The Suite itself is updated manually; no automatic self-replacement is implemented. PDF's separate version checker is user-initiated and does not execute installers.

## Validation before distribution

1. Test changes, cryptographic operations and open/edit/save/reopen flows with synthetic PDFs.
2. Run a current dependency audit, review its findings and scan the final artifacts.
3. Test installation, upgrade, removal and data preservation for each declared system and architecture; document remaining alpha limits.
4. macOS: use the authorized Developer ID, hardened runtime, notarization and stapling. Apple Development/Apple Distribution certificates do not replace Developer ID for this channel.
5. Windows: require publisher Authenticode signing and actual MSI acceptance. Linux: tie packages and checksums to the official release and review release integrity.
6. Publish artifact SHA-256 checksums, release notes with limitations and a private security-reporting channel before enabling downloads.
7. Only then update website download links. Do not remove quarantine, disable Gatekeeper or instruct users to ignore platform protections.

A successful scan, test run or notarization is evidence about the exact artifact checked, not a guarantee that it has no vulnerabilities.

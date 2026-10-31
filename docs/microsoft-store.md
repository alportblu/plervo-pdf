# Plervo PDF on the Microsoft Store

Status verified in Partner Center: October 31, 2026. These Windows x64 packages
are published; certification of a later update is a separate status.

| App | Public Store package |
| --- | --- |
| [Plervo PDF](https://apps.microsoft.com/detail/9PBWKBZRBJNR) | 1.0.8.0, including the merged cryptographic dependency security fixes |
| [Plervo Suite](https://apps.microsoft.com/detail/9P5P5QD5GDK1) | 1.0.1.0, with PDF, CAD and Video in its Store catalog |
| [Plervo CAD](https://apps.microsoft.com/detail/9P5CZZSKKG1Q) | 1.0.8.0 |
| [Plervo Video](https://apps.microsoft.com/detail/9NRKLM46BCDG) | 1.0.8.0 |

Suite 1.0.1.0 is certified and published. Its catalog offers PDF, CAD and Video
through Microsoft Store; each app can also be installed from its own Store page. The other ten applications are being prepared
for separate packages and certification. macOS and Linux distribution stays separate.

Microsoft's [new developer registration](https://learn.microsoft.com/en-us/windows/apps/publish/partner-center/open-a-developer-account) is free for individuals and companies when started at `https://storedeveloper.microsoft.com/`. The [Store signs MSIX submissions](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/code-signing-options); EXE/MSI distribution still requires the publisher's own certificate. Account verification and app certification are separate gates. There is no promised approval date.

## Package

- x64 desktop app, Windows 10 version 2004 or later; `runFullTrust` is required for the native desktop process. No administrator elevation, service, startup task or default automation listener is declared.
- PDF association is offered without changing the user's default PDF handler.
- A fresh profile follows the OS language. Existing interface catalogs and manual language preferences are preserved.
- The `microsoft-store` Cargo feature routes **Help → Check for updates** to Microsoft Store. A saved preference cannot switch it back to GitHub distribution.
- The package includes the app, CLI, verified OCR models and license notices. The CLI's MCP server and app's authenticated loopback control require explicit invocation.
- Registered `plervo-pdf.exe` and `plervo-pdf-cli.exe` execution aliases start the installed apps from a terminal without accessing protected WindowsApps payload paths directly.
- Each executable has its own application entry because the SDK permits only one execution-alias extension per application. Both entries are visible in Start; the CLI declares console/multiple-instance activation and is intended for terminal use. Hiding the CLI with `AppListEntry="none"` passed local SDK and native validation but was rejected by Partner Center because it requires a separate `HeadlessAppBypass` waiver. This package uses the normal visible entry instead.
- Store package version `1.0.8.0` uses the application's experimental `0.1.0-alpha.8` base plus the merged security corrections. The build records its exact Git commit. The major number is required by the Store; it is not a stable-quality claim. Store package revisions end in zero.

## Build and acceptance

Use Windows SDK `makeappx.exe`, Rust 1.95 and the pinned craft-fonts input specified in `.github/workflows/pdf-store.yml`. Copy the three identity values verbatim from the reserved app's **Product identity** page in Partner Center. Never infer them from the app's display name or a company name.

```powershell
packaging/windows/package-store.ps1 `
  -IdentityName $env:STORE_IDENTITY_NAME `
  -Publisher $env:STORE_PUBLISHER `
  -PublisherDisplayName $env:STORE_PUBLISHER_DISPLAY_NAME
```

Normal packaging builds the Store feature, validates the manifest through the SDK and writes a SHA-256 inventory beside the MSIX. The real-identity package is an upload candidate until Microsoft certifies it; the JSON metadata deliberately does not claim certification.

Before compilation, CI invokes `-ValidationOnly -ManifestOnly` to check the same manifest and assets with the Windows SDK. This uses inert, empty executable placeholders only to satisfy file references. The temporary probe is never installed, uploaded or treated as an app test, and is removed even on failure. The real package still undergoes SDK validation and the full native lifecycle below.

The workflow first builds a separate **validation-only** identity in Microsoft's [unsigned testing namespace](https://learn.microsoft.com/en-us/windows/msix/package/unsigned-package). Only the disposable Windows runner installs this package with `-AllowUnsigned`. No root certificate is installed and no SmartScreen or developer-mode policy is changed. Do not distribute this test package or these installation instructions to customers.

Acceptance checks install/file hashes, installed CLI save/reopen and OCR availability, native GUI screenshots in English/Portuguese/Spanish, an MSIX upgrade preserving synthetic preferences, and removal preserving a synthetic document and the default PDF handler. This is not a claim that Store acquisition, all Windows versions, ARM devices or every feature have been tested. Microsoft certification and an installation of the final Store-delivered build remain required.

## Submission

1. Complete account verification and reserve **Plervo PDF**. The owner supplies identity documents and accepts account agreements.
2. Dispatch the workflow with the actual Partner Center identity. Download the successful run's candidate and verify its recorded SHA-256.
3. Create a free desktop app submission in Productivity. Upload the real-identity MSIX, never the validation-only package.
4. Use the localized draft in `packaging/windows/store/listing.json`, native app screenshots from the successful acceptance run, and the [Store edition privacy notice](microsoft-store-privacy.md). Review age ratings and declarations against actual behavior; do not invent certifications or endorsements.
5. Submit for certification. Add the Store download link to the website only after the published listing can be opened and installed.

The listing explicitly identifies the preview and its OCR/language limitations. Donations remain optional and unlock no functionality. Restricted-capability notes explain why a native PDF editor needs `runFullTrust`.

Revision history: October 20, 2026 — initial Store distribution preparation following the owner's choice of this route after SignPath Foundation declined the project.

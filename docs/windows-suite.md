# Windows Suite candidate

`Plervo Windows Suite candidate` builds an **internal, unsigned** x64 MSI and portable ZIP on a disposable Windows 2025 runner. The Suite has its own PE icon, version information and per-monitor DPI manifest. The MSI installs to Program Files and adds a Start Menu shortcut; it does not register document associations, add a service, launch the app after setup or remove per-user settings. Its current wizard is English. The Suite's language setting remains independent and follows the system when set to Auto.

The native gate inspects the compiled MSI, rejects per-user scope overrides, installs the exact binary, checks the CLI, captures the native Suite window, deletes/restores the installed executable through MSI repair, and uninstalls while preserving synthetic preferences, documents and the default PDF handler. A failed GUI capture fails the job. This is not signed-package or clean consumer-device acceptance.

Windows uses eframe's existing wgpu renderer rather than requiring an OpenGL 2+ vendor driver. The initial native candidate installed successfully but could not open with the runner's stock OpenGL 1.1 driver; the native capture remains a required regression check. Mac/Linux keep their existing renderer.

```powershell
dotnet tool install --global wix --version 5.0.2
packaging/windows/package-suite.ps1
# Only on a disposable GitHub Windows runner:
packaging/windows/test-suite-install.ps1 -Path <internal-msi>
```

Output stays in `dist/windows-suite-internal/` and includes source commit, binary/package hashes and an explicit `public_qualification: false`. The workflow has read-only repository permissions, uses no signing secrets and never uploads a release. The existing Windows download gate remains closed.

## Windows distribution

SignPath Foundation declined the initial application. No paid signing service is configured.
The external Windows MSI remains an internal candidate; it must not be advertised as
Store-signed or installed by the Store edition.

# Microsoft Store distribution option

PDF is available at https://apps.microsoft.com/detail/9PBWKBZRBJNR. Its Store edition
uses Microsoft's install/update channel. The Mac/Linux Suite remains a separate
download and package manager.

The Suite Store edition is built with `--features microsoft-store` and packaged by
`packaging/windows/package-suite-store.ps1`. Its own reserved identity is
`RicardoBonetti.PlervoSuite`, Store ID `9P5P5QD5GDK1`; a reservation is not approval.
The independent MSIX workflow exercises the package before any manual submission.

This edition lists and opens the current user's Store-signed Plervo packages using
Windows package APIs. It requires an exact known product identity, the publisher from
Partner Center, a Store signature and healthy registration. It does not execute paths
from the registry or an external download feed. Only a published product gets an
acquisition link. Initially that is Plervo PDF; the Video/CAD identities are reserved,
not published. Unpublished catalog entries are omitted unless already installed from
the Store. `My apps` displays every matching installed product.

Acquisition opens the product's Microsoft Store page with the user's explicit action;
the Store handles consent, download and updates. The updates button opens the Store's
updates page, and removal opens Windows Settings. No GitHub polling, package downloading,
sideloading, certificate installation, service or startup entry is enabled by this
edition. Language follows Windows with English/Portuguese/Spanish overrides. Support
opens the optional Buy Me a Coffee page in the user's browser; it unlocks no features.

The [Microsoft Store policies](https://learn.microsoft.com/windows/apps/publish/store-policies)
(10.1.5, version 7.20) require apps acquired through this flow to be listed and acquired
through the Store. This is a publisher app manager, not an external storefront under
10.1.6. Microsoft still decides whether this initial edition provides sufficient value.
Do not claim acceptance until the Partner Center submission has passed certification.

The native gate checks the test-namespace MSIX, exact payload hashes, registration,
EN/PT/ES rendering, local inventory, invalid/unpublished actions, reinstallation and
removal, preservation of synthetic documents/preferences and the PDF default handler.
The upload payload must match the tested payload except for the manifest identity.
Unsigned test packages are installed only on disposable runners using Microsoft's
package-scoped test namespace. A Windows Server runner does not prove consumer Store
acquisition or launch of a Store-delivered PDF; those limitations remain explicit in
its evidence and certification notes.

Headless commands: `--status`, `--store-link pdf`, `--install pdf`, `--open pdf`,
`--check-updates`, `--manage`. The Store binary does not dispatch the external-edition
installation/removal commands. Mac/Linux builds and existing releases are unchanged.

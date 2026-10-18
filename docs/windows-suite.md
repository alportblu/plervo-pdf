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

## SignPath completion

Approval for the owner's SignPath Foundation application has not been confirmed. Use the existing application; do not submit another or switch to a paid provider.

After approval, the project owner supplies the SignPath organization ID, project slug, signing-policy slug and artifact-configuration slug through the provider/GitHub settings. The API token belongs in a scoped GitHub Actions secret, never chat, source or logs. Follow the [official GitHub integration](https://docs.signpath.io/trusted-build-systems/github) and the [official action inputs](https://github.com/SignPath/github-action-submit-signing-request/blob/main/action.yml); pin the reviewed action commit when the actual integration is configured.

The artifact configuration must cover the executable inside the MSI/portable archive as well as the installer itself. Verify the resulting Authenticode chain, timestamp and approved publisher identity, then rerun acceptance on the exact signed files. Pin the approved publisher in the Suite's Windows installer path before enabling Windows downloads. A successful unsigned candidate or a valid unrelated publisher signature is not sufficient. No certificate is added to a consumer trust store and SmartScreen is not disabled.

# Suite and dependency update — October 23, 2026

This describes the alpha.8 source candidate. Publication and package qualification are
tracked separately in the [release notes](https://github.com/alportblu/plervo-pdf/releases).
It is not a claim that every package below is already available.

## Finding updates

Suite checks the official GitHub release catalog at startup and every six hours while
running on supported macOS/Linux systems. Settings can switch the check to manual.
The request discloses ordinary connection metadata, such as the IP address and application
user agent, to GitHub. It does not send documents, document names or installed-app lists.
No account or credential is required by default. If an operator explicitly configures the
existing optional `PLERVO_SUITE_GITHUB_TOKEN` environment variable, that token is sent only
to GitHub's API for authenticated requests. There is no background service when Suite is closed.

Each app has its own release selection and installed-version comparison. My Apps lists
all detected installations, keeps them usable offline, and marks newer available versions.
The Suite banner offers an explicit download action; it neither silently replaces the running
Suite nor closes documents. Install the downloaded package and reopen Suite. Mac ARM and
Intel asset matching are distinct. Linux offers the package type identified from os-release;
unknown distributions are not guessed. Only official, checksummed product assets are offered.
The existing per-app installer still verifies packages and preserves the previous installation
until replacement is ready. Stable Suite releases do not switch to prerelease channels.

Alpha.5 cannot acquire this new behavior retroactively: its users must first install alpha.8
manually. `plervo-suite --check-updates` exposes the same read-only discovery as JSON.

## Dependency corrections

| Applications | Previous dependency | Updated dependency | Advisory |
| --- | --- | --- | --- |
| PDF and Write | ctutils 0.4.2 / 0.4.3 | 0.4.4 | [RUSTSEC-2026-0385](https://rustsec.org/advisories/RUSTSEC-2026-0385.html) |
| Photo, Video, Develop, Canvas | rustls-webpki 0.103.15 | 0.103.16 | [RUSTSEC-2026-0382](https://rustsec.org/advisories/RUSTSEC-2026-0382.html) |
| Photo, Video | webpki-root-certs 1.0.9 | 1.0.10 | [RUSTSEC-2026-0383](https://rustsec.org/advisories/RUSTSEC-2026-0383.html) |
| Develop, Canvas | webpki-roots 1.0.9 | 1.0.10 | [RUSTSEC-2026-0381](https://rustsec.org/advisories/RUSTSEC-2026-0381.html) |

No new advisory suppression is introduced. The existing, documented RSA applicability
exceptions for Video/Develop remain; RSA itself is not patched. See the
[product security review](../products/SECURITY-REVIEW.md) for their narrow scope.

All twelve additional product candidates include reviewed dependency notices with original
copyright texts and exact source links. Their frozen inventories cover platform-specific,
optional and build dependencies conservatively. Packaging refuses changed dependency inputs
until notices are reviewed again. Code and asset terms remain separate; this does not relicense
third-party components as MIT. Existing packages are not overwritten.

## PHP2Shell exposure check

The [PHP project's October 21 security notice](https://www.php.net/archive/2026.php)
identifies CVE-2026-109618 and CVE-2026-109619. Affected PHP web deployments need their
vendor's fix; upstream fixed branches are 8.5.12, 8.4.27, 8.3.36 and 8.2.35.

Plervo's site is generated static HTML/CSS/JavaScript deployed to Cloudflare Pages. Its
reviewed deployment source (plervo-site commit 917c745), tracked deployment configuration,
and the thirteen desktop source inventories contain no PHP runtime/application or Composer
manifest. The local Plervo tooling also had no PHP/FPM executable or running PHP web handler.
This vulnerability therefore has no identified entry point in the reviewed Plervo scope.
No exploit was run against production. This finding does not assess unrelated servers,
company systems, or every other vulnerability class.

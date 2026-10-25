# Cryptographic dependency maintenance — 2026-10-25

Scope: source correction. Previously published installers are unchanged and need
a separately versioned, qualified release.

The scheduled [CI run on October 25](https://github.com/alportblu/plervo-pdf/actions/runs/40176584548)
failed its advisory check for `crypto-bigint 0.7.5`. Licenses, dependency sources and
bans passed. Windows, macOS, Linux, WASM and the scheduled mutation-fuzz job also
passed. The October 24 run reported the same advisory on the same source commit.

[RUSTSEC-2026-0387](https://github.com/RustSec/advisory-db/blob/main/crates/crypto-bigint/RUSTSEC-2026-0387.md)
describes timing and microarchitectural leakage when affected integer operations
process secrets. Exploitability depends on the algorithm, compiler, CPU and an
attacker's ability to observe the side channel. A dependency alert is not evidence
that an attack occurred.

## Correction and scope

- Raise PDF's direct `crypto-bigint` requirement and lock to `0.7.9`. The advisory
  lists `0.7.6` as patched, but upstream subsequently withdrew `0.7.6`–`0.7.8`;
  [the upstream changelog](https://github.com/RustCrypto/crypto-bigint/blob/v0.7.9/CHANGELOG.md)
  records the additional arithmetic corrections in the currently available release.
- Update the resolved `ctutils` dependency from `0.4.4` to `0.4.5`, including its
  additional constant-time hardening. No other locked package changes.
- Refresh the frozen distribution inventory after comparing both dependencies'
  original MIT and Apache license files byte-for-byte. All 634 inventory entries
  remain, with unchanged original license-text hashes.
- Keep advisory checks and their existing documented exceptions unchanged. No new
  ignore entry or weakened CI gate is introduced.

The PDF signing module uses this dependency directly and through RustCrypto's
elliptic-curve stack. Imported P-256/P-384 private keys and internally generated
P-256 identities use that stack for signing. Native RSA private-key operations
use `aws-lc-rs`; external Keychain/CNG identities delegate signing to their
providers. The existing verification-only RSA advisory exception does **not**
justify ignoring this new elliptic-curve dependency finding.

The standalone Suite launcher does not include `crypto-bigint` in its normal
dependency graph. Opening a PDF app from Suite does not update that PDF's binary.
Video and Develop have a separate, older `crypto-bigint 0.5.5` dependency in their
reviewed inventories. This workspace update does not modify or clear those
products; their dependency paths need separate compatibility and security review.

## Validation and delivery limits

The original advisory failure was reproduced locally. The updated dependency
audit passes with the same policy; the independent `clipboard-win 5.4.1` yanked
version warning remains and was not the cause of the failed advisory job.
Existing signature/PKCS#12/PDF-signing regressions pass locally: 110 passed,
zero failed and two pre-existing opt-in tests ignored (a local-file diagnostic and
a temporary macOS Keychain test). Scoped Clippy
also passes. Full required CI remains a merge gate. These checks do not measure
side-channel resistance on every CPU.

No published release asset is replaced, and no Store or macOS update is claimed
by this source change. New installers must be built, qualified and distributed
before installed copies receive the correction. There is no claim of zero
vulnerabilities or independent security certification.

## Revision history

- 2026-10-25: identify the nightly failure, review upstream fixes and affected PDF
  signing paths, and prepare the dependency/inventory correction.

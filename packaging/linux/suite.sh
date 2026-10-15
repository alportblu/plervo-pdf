#!/usr/bin/env bash
# Package the Suite itself as DEB, RPM and a portable FHS tarball.
set -euo pipefail
. "$(dirname "${BASH_SOURCE[0]}")/../env.sh"
[ "${1:-}" = --skip-build ] || (cd "$ROOT" && cargo build --release --locked -p plervo-suite)
[ "$(uname -m)" = x86_64 ] || { echo 'Linux Suite is qualified only on x86_64.' >&2; exit 2; }
export PLERVO_MIN_GLIBC=2.39 NFPM_ARCH=amd64
export PDFCRAFT_MAINTAINER='Plervo contributors'
export RELEASE="1${PDFCRAFT_BUILD_LABEL:+.$PDFCRAFT_BUILD_LABEL}"
name="plervo-suite-$VERSION${PDFCRAFT_BUILD_LABEL:+-$PDFCRAFT_BUILD_LABEL}-linux-x86_64"
work="$(mktemp -d "$CARGO_TARGET_DIR/suite-package.XXXXXX")"
trap 'rm -rf "$work"' EXIT
stage="$work/$name"
bash "$ROOT/packaging/linux/verify-glibc.sh" "$PLERVO_MIN_GLIBC" "$CARGO_TARGET_DIR/release/plervo-suite"
install -Dm755 "$CARGO_TARGET_DIR/release/plervo-suite" "$stage/bin/plervo-suite"
strip "$stage/bin/plervo-suite"
install -Dm644 "$ROOT/packaging/linux/com.plervo.suite.desktop" "$stage/share/applications/com.plervo.suite.desktop"
install -Dm644 "$ROOT/assets/suite/com.plervo.suite.svg" "$stage/share/icons/hicolor/scalable/apps/com.plervo.suite.svg"
mkdir -p "$stage/share/doc/plervo-suite"
copy_docs "$stage/share/doc/plervo-suite"
desktop-file-validate "$stage/share/applications/com.plervo.suite.desktop"
sed "s|\${STAGE}|$stage|g" "$ROOT/packaging/linux/suite-nfpm.yaml" >"$work/nfpm.yaml"
for format in deb rpm; do
  (cd "$ROOT" && nfpm package -f "$work/nfpm.yaml" -p "$format" -t "$DIST/$name.$format")
done
tar -C "$work" -czf "$DIST/$name.tar.gz" "$name"
echo "Packaged Plervo Suite $VERSION for Linux x86_64"

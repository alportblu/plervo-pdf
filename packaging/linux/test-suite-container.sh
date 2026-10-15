#!/usr/bin/env bash
# Runs only in the disposable test container. It never uses the host desktop or home.
set -euo pipefail
[ "${PLERVO_ACCEPTANCE:-}" = 1 ] && [ -f /.dockerenv ] || { echo 'Disposable container required' >&2; exit 2; }
packages=/packages
out=/evidence
if command -v apt-get >/dev/null; then
  apt-get update -qq
  DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends ca-certificates xvfb xauth dbus-x11 jq libgl1-mesa-dri libvulkan1 mesa-vulkan-drivers libx11-xcb1
  apt-get install -y --no-install-recommends "$packages"/plervo-suite-*.deb
else
  dnf install -y ca-certificates xorg-x11-server-Xvfb xorg-x11-xauth dbus-x11 jq mesa-dri-drivers mesa-vulkan-drivers vulkan-loader shadow-utils procps-ng
  dnf install -y "$packages"/plervo-suite-*.rpm
fi
useradd --create-home --shell /bin/bash plervo
plervo_uid="$(id -u plervo)"
runtime="/run/user/$plervo_uid"
mkdir -p "$out" "$runtime"
chown plervo:plervo "$out" "$runtime"
chmod 700 "$runtime"
/usr/bin/plervo-suite --version
test -f /usr/share/applications/com.plervo.suite.desktop
test -f /usr/share/icons/hicolor/scalable/apps/com.plervo.suite.svg
runuser -u plervo -- env PLERVO_ACCEPTANCE=1 PLERVO_PUBLIC_ACCEPTANCE="${PLERVO_PUBLIC_ACCEPTANCE:-0}" \
  PLERVO_EXPECTED_VERSION="${PLERVO_EXPECTED_VERSION:-}" XDG_RUNTIME_DIR="$runtime" LIBGL_ALWAYS_SOFTWARE=1 PDFCRAFT_SYSTEM_FONTS=0 \
  xvfb-run -a -s '-screen 0 1280x900x24 -nolisten tcp' bash /test/test-suite-user.sh "$packages" "$out"
if command -v apt-get >/dev/null; then
  apt-get remove -y plervo-suite
else
  dnf remove -y plervo-suite
fi
test ! -e /usr/bin/plervo-suite
test -s /home/plervo/Documents/acceptance.pdf
test -s /home/plervo/.config/plervo-preserve.txt
printf '%s\n' 'Suite install/remove, user-level PDF install/update/remove, native Suite/PDF rendering, locale, OCR and documents preserved.' >"$out/result.txt"

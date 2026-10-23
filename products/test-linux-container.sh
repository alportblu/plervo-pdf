#!/usr/bin/env bash
# Package acceptance inside an isolated CI container, never on a user's installation.
set -euo pipefail
[ "${PLERVO_ACCEPTANCE:-}" = 1 ] && [ -f /.dockerenv ] || { echo 'Disposable container required' >&2; exit 2; }
case "${PRODUCT:-}" in
  photo|vector|video|develop|motion|publish|write|slides|sheets|cad|audio|canvas) ;;
  *) echo 'Unknown product identity' >&2; exit 2 ;;
esac
name="plervo-$PRODUCT"
app="/usr/bin/$name"
cli="/usr/bin/$name-cli"
[ ! -e "$app" ] && [ ! -e "$cli" ]
mkdir -p /evidence
if command -v apt-get >/dev/null; then
  packages=(/packages/"$name"-*.deb)
  [ "${#packages[@]}" = 1 ] && [ -f "${packages[0]}" ]
  # Minimal images exclude documentation; retain this package's notices for verification.
  printf 'path-include=/usr/share/doc/%s\npath-include=/usr/share/doc/%s/*\n' "$name" "$name" \
    > /etc/dpkg/dpkg.cfg.d/zz-plervo-acceptance
  apt-get update -qq
  apt-get install -y --no-install-recommends "${packages[0]}"
  apt-get install -y --reinstall --no-install-recommends "${packages[0]}"
else
  packages=(/packages/"$name"-*.rpm)
  [ "${#packages[@]}" = 1 ] && [ -f "${packages[0]}" ]
  dnf install -y "${packages[0]}"
  dnf reinstall -y "${packages[0]}"
  # Fedora minimal images omit the comparator used by the notice check below.
  dnf install -y diffutils
fi
"$app" --version > /evidence/app-version.txt
"$cli" --version > /evidence/cli-version.txt
"$cli" --help > /evidence/cli-help.txt
test -f "/usr/share/applications/com.plervo.$PRODUCT.desktop"
cmp "/usr/share/doc/$name/THIRD_PARTY_NOTICES.txt" "/test/licenses/$PRODUCT/THIRD_PARTY_NOTICES.txt"
test -s /evidence/app-version.txt
test -s /evidence/cli-help.txt
# Resolve the product's runtime dependencies before adding display-only test tools.
# A build runner already has development libraries and can hide broken package metadata.
if command -v apt-get >/dev/null; then
  DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
    xvfb xauth dbus-x11 x11-utils imagemagick libgl1-mesa-dri mesa-vulkan-drivers
else
  dnf install -y xorg-x11-server-Xvfb xorg-x11-xauth dbus-x11 xwininfo ImageMagick \
    mesa-dri-drivers mesa-vulkan-drivers shadow-utils
fi
useradd --create-home --shell /bin/bash plervo
plervo_uid="$(id -u plervo)"
runtime="/run/user/$plervo_uid"
mkdir -p "$runtime"
chown plervo:plervo /evidence "$runtime"
chmod 700 "$runtime"
runuser -u plervo -- env PLERVO_ACCEPTANCE=1 PRODUCT="$PRODUCT" XDG_RUNTIME_DIR="$runtime" \
  LIBGL_ALWAYS_SOFTWARE=1 LP_NUM_THREADS=2 LANG=C.UTF-8 \
  xvfb-run -a -s '-screen 0 1280x900x24 -nolisten tcp' bash /test/test-linux-user.sh
if command -v apt-get >/dev/null; then
  apt-get remove -y "$name"
else
  dnf remove -y "$name"
fi
[ ! -e "$app" ] && [ ! -e "$cli" ]
printf '%s\n' 'Package installation, reinstall, executable checks, unprivileged virtual-desktop first launch and removal passed. Full GUI and feature acceptance remain separate gates.' > /evidence/result.txt

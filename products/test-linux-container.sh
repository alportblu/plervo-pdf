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
  apt-get update -qq
  apt-get install -y --no-install-recommends "${packages[0]}"
  apt-get install -y --reinstall --no-install-recommends "${packages[0]}"
else
  packages=(/packages/"$name"-*.rpm)
  [ "${#packages[@]}" = 1 ] && [ -f "${packages[0]}" ]
  dnf install -y "${packages[0]}"
  dnf reinstall -y "${packages[0]}"
fi
"$app" --version > /evidence/app-version.txt
"$cli" --version > /evidence/cli-version.txt
"$cli" --help > /evidence/cli-help.txt
test -f "/usr/share/applications/com.plervo.$PRODUCT.desktop"
test -s /evidence/app-version.txt
test -s /evidence/cli-help.txt
if command -v apt-get >/dev/null; then
  apt-get remove -y "$name"
else
  dnf remove -y "$name"
fi
[ ! -e "$app" ] && [ ! -e "$cli" ]
printf '%s\n' 'Package installation, reinstall, installed executable checks and removal passed. GUI and feature acceptance are separate gates.' > /evidence/result.txt

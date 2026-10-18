#!/usr/bin/env bash
# A fresh, unprivileged account on a disposable Xvfb display, never a user desktop.
set -euo pipefail
[ "${PLERVO_ACCEPTANCE:-}" = 1 ] && [ -f /.dockerenv ] && [ "$(id -u)" != 0 ] || exit 2
case "${PRODUCT:-}" in
  photo|vector|video|develop|motion|publish|write|slides|sheets|cad|audio|canvas) ;;
  *) exit 2 ;;
esac
out=/evidence
title="Plervo ${PRODUCT^}"
[ "$PRODUCT" != cad ] || title='Plervo CAD'
"/usr/bin/plervo-$PRODUCT" > "$out/gui-startup.log" 2>&1 &
app=$!
cleanup() { kill "$app" 2>/dev/null || true; wait "$app" 2>/dev/null || true; }
trap cleanup EXIT
for attempt in $(seq 1 90); do
  kill -0 "$app" 2>/dev/null || { cat "$out/gui-startup.log"; exit 1; }
  xwininfo -root -tree > "$out/window-tree.txt"
  # Document editors can replace the title with the document's name (e.g. Book1).
  # Their X11 application class remains the canonical product identity.
  if grep -Fq "\"com.plervo.$PRODUCT\"" "$out/window-tree.txt" || grep -Fq "$title" "$out/window-tree.txt"; then
    sleep 5
    kill -0 "$app"
    if command -v import >/dev/null; then
      import -window root "$out/first-launch.png"
    else
      magick import -window root "$out/first-launch.png"
    fi
    test -s "$out/first-launch.png"
    kill -0 "$app"
    printf '%s\n' 'Installed GUI remained alive and exposed its product window on a fresh virtual desktop. Screenshot requires visual review; this is not full feature acceptance.' > "$out/gui-result.txt"
    exit 0
  fi
  sleep 1
done
echo 'Expected application window did not appear within 90 seconds' >&2
exit 1

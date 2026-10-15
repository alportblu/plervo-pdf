#!/usr/bin/env bash
# Install and remove our DEB only on a disposable GitHub runner.
set -euo pipefail
if [ "${GITHUB_ACTIONS:-}" != true ] || [ "${RUNNER_OS:-}" != Linux ]; then
  echo 'This acceptance test is restricted to disposable Linux GitHub runners.' >&2
  exit 2
fi
if dpkg-query -W -f='${Status}' plervo-pdf 2>/dev/null | grep -q 'install ok installed'; then
  echo 'Refusing to replace an existing Plervo installation.' >&2
  exit 2
fi
work="$(mktemp -d "$RUNNER_TEMP/plervo-install.XXXXXX")"
mkdir -p dist/validation
printf '%s\n' 'preserve user document' >"$work/preserve.txt"
cleanup() { sudo dpkg --remove plervo-pdf; }
trap cleanup EXIT
sudo apt-get install -y --no-install-recommends "$(realpath "$1")"
/usr/bin/plervo-pdf-cli --version
/usr/bin/plervo-pdf-cli run ocr_status | jq -e '.available == true'
cat >"$work/steps.json" <<'JSON'
[{"tool":"doc_create","args":{"from":"text","text":"Plervo installation test"}},
 {"tool":"doc_save","args":{"doc":1,"path":"saved.pdf"}}]
JSON
/usr/bin/plervo-pdf-cli run --root "$work" --script "$work/steps.json"
/usr/bin/plervo-pdf-cli info "$work/saved.pdf" | jq -e '.pages == 1'
test -f /usr/share/applications/com.plervo.pdf.desktop
cleanup
trap - EXIT
test ! -e /usr/bin/plervo-pdf
test ! -e /usr/bin/plervo-pdf-cli
test -s "$work/saved.pdf"
test "$(cat "$work/preserve.txt")" = 'preserve user document'
# Test the standalone CLI after uninstall so system-wide models cannot mask a missing bundle.
archives=(dist/release/plervo-pdf-cli-*.tar.gz)
[ "${#archives[@]}" = 1 ]
mkdir "$work/standalone"
tar -xzf "${archives[0]}" -C "$work/standalone"
standalone=("$work"/standalone/*/plervo-pdf-cli)
[ "${#standalone[@]}" = 1 ]
env -u PDFCRAFT_MODELS "${standalone[0]}" run ocr_status | jq -e '.available == true'
printf '%s\n' 'DEB install, installed and standalone CLI OCR discovery, save/reopen, desktop entry, uninstall and document preservation passed.' >dist/validation/linux-install-result.txt

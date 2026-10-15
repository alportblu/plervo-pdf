#!/usr/bin/env bash
set -euo pipefail
packages="$1"
out="$2"
mkdir -p "$HOME/Documents" "$HOME/.config"
printf '%s\n' 'keep settings' >"$HOME/.config/plervo-preserve.txt"
archive=("$packages"/plervo-pdf-*-linux-x86_64.tar.gz)
[ "${#archive[@]}" = 1 ]
helper=/validation/linux_acceptance
binary="$($helper "${archive[0]}")"
cli="$(dirname "$binary")/plervo-pdf-cli"
"$cli" --version
"$cli" run ocr_status | jq -e '.available == true'
cat >"$out/create.json" <<'JSON'
[{"tool":"doc_create","args":{"from":"text","text":"Plervo Linux acceptance"}}, {"tool":"doc_save","args":{"doc":1,"path":"acceptance.pdf"}}]
JSON
"$cli" run --root "$HOME/Documents" --script "$out/create.json"
"$cli" info "$HOME/Documents/acceptance.pdf" | jq -e '.pages == 1'
# Update through the same manager with the verified candidate in its download cache.
"$helper" "${archive[0]}"
for language in en_US.UTF-8 pt_BR.UTF-8 es_ES.UTF-8; do
  LANG="$language" LANGUAGE="${language%%.*}" timeout 60 plervo-suite --native-smoke-test "$out/suite-$language.png"
done
# The PDF's existing opt-in control channel exercises the actual installed GUI.
"$binary" --control "$out/pdf-control.json" "$HOME/Documents/acceptance.pdf" >"$out/pdf.log" 2>&1 &
pdf_pid=$!
trap 'kill "$pdf_pid" 2>/dev/null || true' EXIT
for _attempt in $(seq 1 60); do
  [ -s "$out/pdf-control.json" ] && break
  sleep 1
done
"$cli" ui --control "$out/pdf-control.json" inspect >"$out/pdf-inspect.json"
"$cli" ui --control "$out/pdf-control.json" screenshot --out "$out/pdf.png"
if plervo-suite --uninstall-pdf >"$out/refuse-running.txt" 2>&1; then
  echo 'Uninstall must refuse a running PDF.' >&2
  exit 1
fi
kill "$pdf_pid"
wait "$pdf_pid" || true
trap - EXIT
rm -f "$out/pdf-control.json"
plervo-suite --uninstall-pdf
plervo-suite --status | jq -e '. == null'
test -s "$HOME/Documents/acceptance.pdf"
test "$(cat "$HOME/.config/plervo-preserve.txt")" = 'keep settings'
# Loopback control credentials are never included in the uploaded evidence.

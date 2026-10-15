#!/usr/bin/env bash
# Refuse binaries that require a newer glibc than the package declares. Does not execute them.
set -euo pipefail
if [ "$#" -lt 2 ] || [[ ! "$1" =~ ^[0-9]+\.[0-9]+$ ]]; then
  echo "usage: verify-glibc.sh MAX_MAJOR.MINOR ELF..." >&2
  exit 2
fi
ceiling="$1"
shift
for binary in "$@"; do
  headers="$(LC_ALL=C "${CROSS_COMPILE:-}objdump" -p "$binary")"
  if ! printf '%s\n' "$headers" | LC_ALL=C awk -v ceiling="$ceiling" '
    BEGIN { split(ceiling, cap, /[.]/) }
    $NF ~ /^GLIBC_/ {
      seen = 1
      if ($NF !~ /^GLIBC_[0-9]+\.[0-9]+(\.[0-9]+)?$/) {
        print "unsupported glibc requirement: " $NF > "/dev/stderr"
        bad = 1
        next
      }
      split(substr($NF, 7), version, /[.]/)
      if (version[1] > cap[1] || (version[1] == cap[1] &&
          (version[2] > cap[2] || (version[2] == cap[2] && version[3] > 0)))) {
        print "requires " $NF ", package baseline is " ceiling > "/dev/stderr"
        bad = 1
      }
    }
    END {
      if (!seen) print "no glibc version requirements found" > "/dev/stderr"
      exit (bad || !seen)
    }
  '; then
    echo "error: incompatible or unreadable Linux runtime requirements: $binary" >&2
    exit 1
  fi
done

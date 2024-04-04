#!/bin/bash
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"
cd "$SCRIPT_DIR/.."

HUMANODE_PEER_PATH="$(realpath -s "$HUMANODE_PEER_PATH")"
export HUMANODE_PEER_PATH

# Run all zombienet tests.
for TESTFILE in "$SCRIPT_DIR"/*/*.zndsl; do
  printf "=> Running test %s\n" "$TESTFILE" >&2
  zombienet -p native test "$TESTFILE"
  printf "=> Test %s passed\n" "$TESTFILE" >&2
done

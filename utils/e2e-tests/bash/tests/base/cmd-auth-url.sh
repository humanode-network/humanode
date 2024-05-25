#!/bin/bash
set -euo pipefail

# Make temporary test directory.
TEMPDIR="$(mktemp -d)"
trap 'rm -rf "$TEMPDIR"' EXIT

OUTPUT="$("$HUMANODE_PEER_PATH" bioauth auth-url --webapp-url https://example.com --rpc-url https://localhost:9933)"
EXPECTED="https://example.com/open?url=https%3A%2F%2Flocalhost%3A9933"

if [[ "$OUTPUT" != "$EXPECTED" ]]; then
  printf "Output did not match:\n  expected: %s\n  actual:   %s\n" "$EXPECTED" "$OUTPUT"
  exit 1
fi

printf "Test succeeded\n" >&2

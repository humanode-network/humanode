#!/bin/bash
set -euo pipefail

# Make temporary test directory.
TEMPDIR="$(mktemp -d)"
trap 'rm -rf "$TEMPDIR"' EXIT

GENERATE_OUTPUT="$("$HUMANODE_PEER_PATH" bioauth key generate)"

# Look for "Secret phrase" in the output. Fail the test if absent.
grep -q "Secret phrase:" <<<"$GENERATE_OUTPUT"

INSPECT_OUTPUT="$("$HUMANODE_PEER_PATH" bioauth key inspect --suri "custom resemble extend detect expand ready battle never deputy argue right tent")"
EXPECTED_INSPECT_OUTPUT="$(
  cat <<EOF
Secret phrase:       custom resemble extend detect expand ready battle never deputy argue right tent
  Network ID:        substrate
  Secret seed:       0x5cfec9f10108959aa5bb6b5966a0ba8df2c2f5c1181acf91dcdf9ec227f17226
  Public key (hex):  0xeeaf49c07a5c2af27b081042338ad41b7991f4aa078e2f7a6ddfd8266b8e985c
  Account ID:        0xeeaf49c07a5c2af27b081042338ad41b7991f4aa078e2f7a6ddfd8266b8e985c
  Public key (SS58): 5HTfKp6LcTcDh5ep2gszAHcKKC1YUExzvpphnJ7NTtbFBwBn
  SS58 Address:      5HTfKp6LcTcDh5ep2gszAHcKKC1YUExzvpphnJ7NTtbFBwBn
EOF
)"

if [[ "$INSPECT_OUTPUT" != "$EXPECTED_INSPECT_OUTPUT" ]]; then
  printf "Output did not match:\n\n  expected: %s\n\n  actual:   %s\n" "$INSPECT_OUTPUT" "$EXPECTED_INSPECT_OUTPUT"
  exit 1
fi

printf "Test succeeded\n" >&2

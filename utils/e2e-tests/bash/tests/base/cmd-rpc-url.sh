#!/bin/bash
set -euo pipefail

# Make temporary test directory.
TEMPDIR="$(mktemp -d)"
trap 'rm -rf "$TEMPDIR"' EXIT

assert_eq() {
  local ACTUAL="$1"
  local EXPECTED="$2"
  if [[ "$ACTUAL" != "$EXPECTED" ]]; then
    printf "Output did not match:\n expected: %s\n actual: %s\n" "$EXPECTED" "$ACTUAL"
    exit 1
  fi
}

# An explicit URL is printed unchanged. Resolution must not require a webapp URL.
OUTPUT="$("$HUMANODE_PEER_PATH" rpc-url --rpc-url https://localhost:9933)"
assert_eq "$OUTPUT" "https://localhost:9933"

# Localhost fallback uses the same port and scheme preference the node uses.
OUTPUT="$(
  "$HUMANODE_PEER_PATH" rpc-url \
    --chain dev \
    --tmp \
    --rpc-port 9944 \
    --rpc-url-scheme-preference ws
)"
assert_eq "$OUTPUT" "ws://localhost:9944"

OUTPUT="$(
  "$HUMANODE_PEER_PATH" rpc-url \
    --chain dev \
    --tmp \
    --rpc-port 9933 \
    --rpc-url-scheme-preference http
)"
assert_eq "$OUTPUT" "http://localhost:9933"

# An explicitly unset URL is a failure, so callers never parse an error as a URL.
if "$HUMANODE_PEER_PATH" rpc-url --chain dev --tmp --rpc-url-unset >"$TEMPDIR/unset.stdout" 2>"$TEMPDIR/unset.stderr"; then
  printf "rpc-url --rpc-url-unset was expected to fail\n"
  exit 1
fi
if [[ -s "$TEMPDIR/unset.stdout" ]]; then
  printf "rpc-url --rpc-url-unset wrote to stdout:\n%s\n" "$(cat "$TEMPDIR/unset.stdout")"
  exit 1
fi

printf "Test succeeded\n" >&2

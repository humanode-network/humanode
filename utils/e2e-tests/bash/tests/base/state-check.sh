#!/bin/bash
set -euo pipefail

# Make temporary test directory.
TEMPDIR="$(mktemp -d)"
trap 'rm -rf "$TEMPDIR"' EXIT

# Set up command.
COMMAND_TO_RUN=("$HUMANODE_PEER_PATH" --dev --base-path "$TEMPDIR")

# Set up timeout.
TIME_IN_SEC=20

run_with_timeout() {
  ANCHOR="$SECONDS"
  if timeout "$@"; then
    printf "App run lasted for %d seconds and was terminated successfully (which is bad because we expect it to keep running)\n" "$(("$SECONDS" - "$ANCHOR"))" >&2
    exit
  else
    EXITCODE="$?"
    if [[ "$EXITCODE" -ne 124 ]]; then
      printf "App run lasted for %d seconds and was terminated with error code %d\n" "$(("$SECONDS" - "$ANCHOR"))" "$EXITCODE" >&2
      exit
    else
      printf "App run lasted for %d seconds and was terminated by timeout\n" "$TIME_IN_SEC" >&2
    fi
  fi
}

# Run with empty state, then 2nd time with non-empty state.
for RUN in {1..2}; do
  printf "Run %d started\n" "$RUN" >&2
  run_with_timeout "$TIME_IN_SEC" "${COMMAND_TO_RUN[@]}"
done

printf "Test succeeded\n" >&2
